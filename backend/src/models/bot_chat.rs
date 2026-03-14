use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, ToSchema, sqlx::FromRow)]
pub struct BotChat {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub bot_id: Uuid,
    #[schema(example = 100500)]
    pub chat_id: i64,
    /// Chat type: "dialog", "chat", or "channel"
    #[schema(example = "chat")]
    pub chat_type: Option<String>,
    #[schema(example = "My Chat")]
    pub title: Option<String>,
    pub icon_url: Option<String>,
    #[schema(example = 5)]
    pub participants: Option<i32>,
    #[schema(example = "2026-03-14T12:00:00Z")]
    pub last_event_at: Option<DateTime<Utc>>,
    #[schema(example = "2026-03-14T12:00:00Z")]
    pub synced_at: DateTime<Utc>,
}
