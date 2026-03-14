use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::db::bot_chats::EventChat;
use crate::db::events::{self, NewEvent};
use crate::errors::AppError;
use crate::services::classification::{classify_outbound, extract_chat_id_outbound};

/// Shared ingestion path for both webhook and polling.
/// Extracts metadata from raw Max updates and batch-inserts them.
/// Takes owned Vec to avoid unnecessary clones.
pub async fn ingest_updates(
    pool: &PgPool,
    bot_id: Uuid,
    updates: Vec<serde_json::Value>,
    source: &str,
) -> Result<Vec<Uuid>, AppError> {
    if updates.is_empty() {
        return Ok(vec![]);
    }

    let new_events: Vec<NewEvent> = updates
        .into_iter()
        .map(|u| extract_event(bot_id, u, "inbound", source))
        .collect();

    // Batch upsert bot_chats for events with chat_id
    let chat_events: Vec<_> = new_events.iter()
        .filter_map(|e| {
            let cid = e.chat_id?;
            let title = extract_sender_name_for_dialog(&e.raw_payload, e.chat_type.as_deref());
            Some(EventChat { bot_id, chat_id: cid, chat_type: e.chat_type.clone(), title })
        })
        .collect();
    if !chat_events.is_empty() {
        if let Err(e) = db::bot_chats::batch_upsert_from_events(pool, &chat_events).await {
            tracing::warn!(error = %e, "failed to upsert bot_chats from events");
        }
    }

    let ids = events::batch_insert(pool, &new_events).await?;
    Ok(ids)
}

/// Transactional variant for polling worker (events + marker update in same tx).
pub async fn ingest_updates_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    bot_id: Uuid,
    updates: Vec<serde_json::Value>,
    source: &str,
) -> Result<Vec<Uuid>, AppError> {
    if updates.is_empty() {
        return Ok(vec![]);
    }

    let new_events: Vec<NewEvent> = updates
        .into_iter()
        .map(|u| extract_event(bot_id, u, "inbound", source))
        .collect();

    // Batch upsert bot_chats for events with chat_id (within transaction)
    let chat_events: Vec<_> = new_events.iter()
        .filter_map(|e| {
            let cid = e.chat_id?;
            let title = extract_sender_name_for_dialog(&e.raw_payload, e.chat_type.as_deref());
            Some(EventChat { bot_id, chat_id: cid, chat_type: e.chat_type.clone(), title })
        })
        .collect();
    if !chat_events.is_empty() {
        if let Err(e) = db::bot_chats::batch_upsert_from_events_tx(tx, &chat_events).await {
            tracing::warn!(error = %e, "failed to upsert bot_chats from events");
        }
    }

    // NOTE: batch_insert and batch_insert_tx share identical QueryBuilder logic.
    // This duplication is a known sqlx ergonomic limitation: sqlx's Executor trait
    // doesn't unify &PgPool and &mut Transaction easily with compile-time checked queries.
    let ids = events::batch_insert_tx(tx, &new_events).await?;
    Ok(ids)
}

fn extract_event(bot_id: Uuid, update: serde_json::Value, direction: &str, source: &str) -> NewEvent {
    let update_id = update.get("update_id").and_then(|v| v.as_i64());

    // Determine update type from the update object
    let update_type = detect_update_type(&update).to_string();

    let recipient = update
        .get("message")
        .or_else(|| update.get("callback"))
        .and_then(|m| m.get("recipient"));

    let chat_id = recipient
        .and_then(|r| r.get("chat_id"))
        .and_then(|v| v.as_i64())
        // Fallback: direct chat_id on message object
        .or_else(|| {
            update
                .get("message")
                .or_else(|| update.get("callback"))
                .and_then(|m| m.get("chat_id"))
                .and_then(|v| v.as_i64())
        });

    let chat_type = recipient
        .and_then(|r| r.get("chat_type"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let sender_id = update
        .get("message")
        .or_else(|| update.get("callback"))
        .and_then(|m| m.get("sender"))
        .and_then(|s| s.get("user_id"))
        .and_then(|v| v.as_i64());

    let timestamp = update
        .get("timestamp")
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

    NewEvent {
        bot_id,
        max_update_id: update_id,
        update_type,
        chat_id,
        chat_type,
        sender_id,
        timestamp,
        raw_payload: update,
        direction: direction.to_string(),
        source: source.to_string(),
    }
}

/// Extract sender first_name + last_name as chat title for dialog-type chats.
/// For group chats and channels, returns None (they have their own title).
fn extract_sender_name_for_dialog(payload: &serde_json::Value, chat_type: Option<&str>) -> Option<String> {
    if chat_type != Some("dialog") {
        return None;
    }
    let sender = payload
        .get("message")
        .or_else(|| payload.get("message_created"))
        .and_then(|m| {
            // Handle both direct message and nested message_created.message
            m.get("sender").or_else(|| m.get("message").and_then(|inner| inner.get("sender")))
        })?;
    let first = sender.get("first_name").and_then(|v| v.as_str());
    let last = sender.get("last_name").and_then(|v| v.as_str());
    let parts: Vec<&str> = [first, last].into_iter().flatten().collect();
    if parts.is_empty() {
        // Fallback to deprecated "name" field
        return sender.get("name").and_then(|v| v.as_str()).map(|s| s.chars().take(255).collect());
    }
    let name: String = parts.join(" ").chars().take(255).collect();
    Some(name)
}

/// Build a NewEvent for an outbound API call (no DB interaction).
pub fn build_outbound_event(
    bot_id: Uuid,
    max_bot_id: Option<i64>,
    source: &str,
    method: &str,
    path: &str,
    status_code: u16,
    request_body: Option<serde_json::Value>,
    response_body: Option<serde_json::Value>,
    timestamp: Option<i64>,
) -> NewEvent {
    let update_type = classify_outbound(method, path).to_string();
    let chat_id = extract_chat_id_outbound(path, request_body.as_ref(), response_body.as_ref());
    let ts = timestamp.unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

    let raw_payload = serde_json::json!({
        "method": method,
        "path": path,
        "status_code": status_code,
        "request_body": request_body,
        "response_body": response_body,
    });

    NewEvent {
        bot_id,
        max_update_id: None,
        update_type,
        chat_id,
        chat_type: None,
        sender_id: max_bot_id,
        timestamp: ts,
        raw_payload,
        direction: "outbound".to_string(),
        source: source.to_string(),
    }
}

/// Ingest a single outbound event (convenience wrapper for proxy/gateway).
pub async fn ingest_outbound(
    pool: &PgPool,
    bot_id: Uuid,
    max_bot_id: Option<i64>,
    source: &str,
    method: &str,
    path: &str,
    status_code: u16,
    request_body: Option<serde_json::Value>,
    response_body: Option<serde_json::Value>,
    timestamp: Option<i64>,
) -> Result<Vec<Uuid>, AppError> {
    let event = build_outbound_event(bot_id, max_bot_id, source, method, path, status_code, request_body, response_body, timestamp);
    ingest_outbound_batch(pool, bot_id, vec![event]).await
}

/// Batch ingest multiple outbound events (for ingestion API).
pub async fn ingest_outbound_batch(
    pool: &PgPool,
    bot_id: Uuid,
    new_events: Vec<NewEvent>,
) -> Result<Vec<Uuid>, AppError> {
    // Upsert bot_chats for events with chat_id
    let chat_events: Vec<_> = new_events.iter()
        .filter_map(|e| e.chat_id.map(|cid| EventChat { bot_id, chat_id: cid, chat_type: None, title: None }))
        .collect();
    if !chat_events.is_empty() {
        if let Err(e) = db::bot_chats::batch_upsert_from_events(pool, &chat_events).await {
            tracing::warn!(error = %e, "failed to upsert bot_chats from outbound events");
        }
    }

    let ids = events::batch_insert(pool, &new_events).await?;
    Ok(ids)
}

const TYPE_FIELDS: &[&str] = &[
    "message_created",
    "message_callback",
    "message_edited",
    "message_removed",
    "bot_added",
    "bot_removed",
    "user_added",
    "user_removed",
    "bot_started",
    "chat_title_changed",
    "message_construction_request",
    "message_construction_result",
    "message_chat_created",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_known_field_presence() {
        let update = serde_json::json!({ "message_created": {} });
        assert_eq!(detect_update_type(&update), "message_created");
    }

    #[test]
    fn detect_explicit_update_type_field() {
        let update = serde_json::json!({ "update_type": "bot_started" });
        assert_eq!(detect_update_type(&update), "bot_started");
    }

    #[test]
    fn detect_explicit_field_takes_priority() {
        let update = serde_json::json!({ "update_type": "message_created", "bot_started": {} });
        assert_eq!(detect_update_type(&update), "message_created");
    }

    #[test]
    fn detect_unknown_returns_unknown() {
        let update = serde_json::json!({ "some_new_field": 123 });
        assert_eq!(detect_update_type(&update), "unknown");
    }

    #[test]
    fn detect_unknown_explicit_type_returns_unknown() {
        let update = serde_json::json!({ "update_type": "brand_new_type" });
        assert_eq!(detect_update_type(&update), "unknown");
    }
}

fn detect_update_type(update: &serde_json::Value) -> &'static str {
    // Try explicit update_type field first (if the sender provides it)
    if let Some(t) = update.get("update_type").and_then(|v| v.as_str()) {
        for &field in TYPE_FIELDS {
            if t == field {
                return field;
            }
        }
    }

    // Fallback: detect by field presence
    for &field in TYPE_FIELDS {
        if update.get(field).is_some() {
            return field;
        }
    }

    "unknown"
}
