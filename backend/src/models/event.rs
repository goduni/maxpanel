use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, ToSchema)]
pub struct Event {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    #[schema(example = "660e8400-e29b-41d4-a716-446655440000")]
    pub bot_id: Uuid,
    #[schema(example = 98765)]
    pub max_update_id: Option<i64>,
    #[schema(example = "message_created")]
    pub update_type: String,
    #[schema(example = 100500)]
    pub chat_id: Option<i64>,
    #[schema(example = 42)]
    pub sender_id: Option<i64>,
    #[schema(example = 1710410400)]
    pub timestamp: i64,
    /// Raw update payload from the Max API
    pub raw_payload: serde_json::Value,
    #[schema(example = "2026-03-14T12:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "inbound")]
    pub direction: String,
    #[schema(example = "webhook")]
    pub source: String,
}

