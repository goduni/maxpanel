use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::errors::AppError;
use crate::models::Event;

pub async fn list_for_bot(
    pool: &PgPool,
    bot_id: Uuid,
    direction: Option<&str>,
    cursor: Option<(DateTime<Utc>, Uuid)>,
    limit: i64,
) -> Result<Vec<Event>, AppError> {
    let events = db::events::list_for_bot_filtered(pool, bot_id, direction, cursor, limit).await?;
    Ok(events)
}

pub async fn list_for_bot_chat(
    pool: &PgPool,
    bot_id: Uuid,
    chat_id: i64,
    direction: Option<&str>,
    cursor: Option<(DateTime<Utc>, Uuid)>,
    limit: i64,
) -> Result<Vec<Event>, AppError> {
    let events = db::events::list_for_bot_chat_filtered(pool, bot_id, chat_id, direction, cursor, limit).await?;
    Ok(events)
}

pub async fn get_event(
    pool: &PgPool,
    event_id: Uuid,
    bot_id: Uuid,
    created_at_hint: Option<DateTime<Utc>>,
) -> Result<Event, AppError> {
    db::events::find_by_id(pool, event_id, bot_id, created_at_hint)
        .await?
        .ok_or(AppError::NotFound)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_cursor_roundtrip() {
        let dt = Utc::now();
        let id = Uuid::new_v4();
        let cursor = encode_cursor(&dt, &id);
        let (decoded_dt, decoded_id) = decode_cursor(&cursor).unwrap();
        assert_eq!(decoded_id, id);
        // DateTime roundtrip through RFC3339 may lose sub-nanosecond precision
        assert!((dt - decoded_dt).num_milliseconds().abs() < 1);
    }

    #[test]
    fn event_cursor_invalid() {
        assert!(decode_cursor("not-valid-base64!!!").is_err());
        assert!(decode_cursor("").is_err());
    }

}

/// Decode a cursor string into (DateTime, Uuid).
pub fn decode_cursor(cursor: &str) -> Result<(DateTime<Utc>, Uuid), AppError> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| AppError::BadRequest("Invalid cursor".into()))?;
    let s = String::from_utf8(bytes)
        .map_err(|_| AppError::BadRequest("Invalid cursor".into()))?;
    let parts: Vec<&str> = s.splitn(2, ',').collect();
    if parts.len() != 2 {
        return Err(AppError::BadRequest("Invalid cursor".into()));
    }
    let dt = parts[0]
        .parse::<DateTime<Utc>>()
        .map_err(|_| AppError::BadRequest("Invalid cursor".into()))?;
    let id = parts[1]
        .parse::<Uuid>()
        .map_err(|_| AppError::BadRequest("Invalid cursor".into()))?;
    Ok((dt, id))
}

/// Encode a (DateTime, Uuid) into a cursor string.
pub fn encode_cursor(dt: &DateTime<Utc>, id: &Uuid) -> String {
    use base64::Engine;
    let raw = format!("{},{}", dt.to_rfc3339(), id);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw.as_bytes())
}
