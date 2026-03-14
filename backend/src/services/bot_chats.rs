use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::db;
use crate::db::bot_chats::UpsertChat;
use crate::errors::AppError;
use crate::models::BotChat;
use crate::services::max_api;

/// Extract display name from dialog_with_user for dialog-type chats.
fn extract_dialog_user_name(chat: &serde_json::Value) -> Option<String> {
    let user = chat.get("dialog_with_user")?;
    let first = user.get("first_name").and_then(|v| v.as_str());
    let last = user.get("last_name").and_then(|v| v.as_str());
    let name: Vec<&str> = [first, last].into_iter().flatten().collect();
    if name.is_empty() {
        // Fallback to deprecated "name" field
        return user.get("name").and_then(|v| v.as_str()).map(|s| truncate_name(s));
    }
    Some(truncate_name(&name.join(" ")))
}

fn truncate_name(s: &str) -> String {
    s.chars().take(255).collect()
}

/// Sync chats from Max API for a bot. Walks all pages.
/// Returns the number of chats synced.
pub async fn sync_chats(
    pool: &PgPool,
    config: &Config,
    http_client: &reqwest::Client,
    bot_id: Uuid,
    access_token: &str,
) -> Result<i64, AppError> {
    let page_size: i64 = 100;
    let mut marker: Option<i64> = None;
    let mut total_synced: i64 = 0;

    loop {
        let response = max_api::get_chats(http_client, config, access_token, page_size, marker).await?;

        let chats = response
            .get("chats")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        if chats.is_empty() {
            break;
        }

        let upserts: Vec<UpsertChat> = chats
            .iter()
            .filter_map(|c| {
                let chat_id = c.get("chat_id").and_then(|v| v.as_i64())?;
                Some(UpsertChat {
                    bot_id,
                    chat_id,
                    chat_type: c.get("type").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    title: c.get("title").and_then(|v| v.as_str()).map(|s| s.to_string())
                        .or_else(|| extract_dialog_user_name(c)),
                    icon_url: c.get("icon")
                        .and_then(|v| v.get("url"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    participants: c.get("participants_count").and_then(|v| v.as_i64()).map(|v| v as i32),
                    last_event_at: None,
                })
            })
            .collect();

        total_synced += upserts.len() as i64;
        db::bot_chats::batch_upsert_from_sync(pool, &upserts).await?;

        // Check for next page
        marker = response.get("marker").and_then(|v| v.as_i64());
        if marker.is_none() || chats.len() < page_size as usize {
            break;
        }
    }

    tracing::info!(
        target: "audit",
        bot_id = %bot_id,
        total_synced = total_synced,
        "chats synced from Max API"
    );

    Ok(total_synced)
}

/// List chats for a bot from the bot_chats table.
pub async fn list_chats(
    pool: &PgPool,
    bot_id: Uuid,
    cursor: Option<(chrono::DateTime<chrono::Utc>, i64)>,
    limit: i64,
    search: Option<&str>,
) -> Result<Vec<BotChat>, AppError> {
    let chats = db::bot_chats::list_for_bot(pool, bot_id, cursor, limit, search).await?;
    Ok(chats)
}

/// Encode a (DateTime, chat_id) into a cursor string.
pub fn encode_chat_cursor(time: &chrono::DateTime<chrono::Utc>, chat_id: i64) -> String {
    use base64::Engine;
    let raw = format!("{},{}", time.to_rfc3339(), chat_id);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw.as_bytes())
}

/// Decode a cursor string into (DateTime, chat_id).
pub fn decode_chat_cursor(cursor: &str) -> Result<(chrono::DateTime<chrono::Utc>, i64), AppError> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| AppError::BadRequest("Invalid cursor".into()))?;
    let raw = String::from_utf8(bytes)
        .map_err(|_| AppError::BadRequest("Invalid cursor".into()))?;
    let parts: Vec<&str> = raw.splitn(2, ',').collect();
    if parts.len() != 2 {
        return Err(AppError::BadRequest("Invalid cursor".into()));
    }
    let time = chrono::DateTime::parse_from_rfc3339(parts[0])
        .map_err(|_| AppError::BadRequest("Invalid cursor".into()))?
        .with_timezone(&chrono::Utc);
    let chat_id: i64 = parts[1]
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid cursor".into()))?;
    Ok((time, chat_id))
}

// --- Chat History Sync & Proxy ---

use crate::db::events::{self, NewHistoryEvent};

/// Convert a Max API message object into a NewHistoryEvent.
/// `max_bot_id` is used to determine direction (inbound vs outbound).
fn message_to_history_event(
    bot_id: Uuid,
    max_bot_id: Option<i64>,
    message: &serde_json::Value,
) -> Option<NewHistoryEvent> {
    let mid = message.get("body")
        .and_then(|b| b.get("mid"))
        .and_then(|v| v.as_str())?;

    let timestamp = message.get("timestamp")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let chat_id = message.get("recipient")
        .and_then(|r| r.get("chat_id"))
        .and_then(|v| v.as_i64());

    let sender_id = message.get("sender")
        .and_then(|s| s.get("user_id"))
        .and_then(|v| v.as_i64());

    // Determine direction: if sender matches bot, it's outbound
    let direction = match (sender_id, max_bot_id) {
        (Some(sid), Some(bid)) if sid == bid => "outbound",
        _ => "inbound",
    };

    // Deterministic hash of mid for deduplication.
    // FNV-1a: simple, fast, deterministic across compilations.
    let mid_hash = {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in mid.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash as i64
    };

    // Derive created_at from message timestamp for correct partition placement
    let created_at = chrono::DateTime::from_timestamp_millis(timestamp)
        .unwrap_or_else(chrono::Utc::now);

    // Match the webhook payload structure: { "message": {...}, "timestamp": ... }
    let raw_payload = serde_json::json!({
        "message": message,
        "timestamp": timestamp,
    });

    Some(NewHistoryEvent {
        bot_id,
        max_update_id: Some(mid_hash),
        update_type: "message_created".to_string(),
        chat_id,
        sender_id,
        timestamp,
        raw_payload,
        direction: direction.to_string(),
        source: "history_sync".to_string(),
        created_at,
    })
}

/// Sync up to `limit` messages for a single chat from Max API.
/// Returns the number of messages synced.
pub async fn sync_chat_history(
    pool: &PgPool,
    config: &Config,
    http_client: &reqwest::Client,
    bot_id: Uuid,
    max_bot_id: Option<i64>,
    chat_id: i64,
    access_token: &str,
    limit: i32,
) -> Result<i64, AppError> {
    debug_assert!(limit > 0, "sync_chat_history called with non-positive limit");

    let mut total_synced: i64 = 0;
    let mut total_seen: i64 = 0;
    let mut to_timestamp: Option<i64> = None;
    let batch_size = 100i32; // Max API maximum

    while total_seen < limit as i64 {
        let fetch_count = batch_size.min((limit as i64 - total_seen) as i32);
        let resp = max_api::get_messages(
            http_client, config, access_token, chat_id, to_timestamp, fetch_count,
        ).await?;

        // Take ownership of the messages array to avoid cloning
        let messages: Vec<serde_json::Value> = match resp {
            serde_json::Value::Object(mut obj) => {
                match obj.remove("messages") {
                    Some(serde_json::Value::Array(a)) if !a.is_empty() => a,
                    _ => break,
                }
            }
            _ => break,
        };

        let new_events: Vec<NewHistoryEvent> = messages.iter()
            .filter_map(|m| message_to_history_event(bot_id, max_bot_id, m))
            .collect();

        if new_events.is_empty() {
            break;
        }

        // Update to_timestamp for next page: use oldest message timestamp minus 1ms
        // to avoid re-fetching messages with the exact same timestamp on the boundary.
        to_timestamp = messages.last()
            .and_then(|m| m.get("timestamp"))
            .and_then(|v| v.as_i64())
            .map(|ts| ts - 1);

        total_seen += messages.len() as i64;
        let inserted = events::batch_insert_history(pool, &new_events).await?;
        total_synced += inserted.len() as i64;

        // If we got fewer than requested, no more messages
        if (messages.len() as i32) < fetch_count {
            break;
        }

        // Rate limit: 100ms delay between batches to stay under 30 rps
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    Ok(total_synced)
}

/// Proxy message history from Max API without storing.
/// Returns raw messages JSON from Max API.
pub async fn proxy_chat_history(
    config: &Config,
    http_client: &reqwest::Client,
    access_token: &str,
    chat_id: i64,
    to: Option<i64>,
    count: i32,
) -> Result<serde_json::Value, AppError> {
    max_api::get_messages(http_client, config, access_token, chat_id, to, count.clamp(1, 100)).await
}
