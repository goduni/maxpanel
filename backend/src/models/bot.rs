use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Database row — never implement Serialize on this
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BotRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub access_token_enc: Vec<u8>,
    pub access_token_nonce: Vec<u8>,
    pub key_version: i32,
    pub event_mode: EventMode,
    pub webhook_secret: Option<Uuid>,
    pub webhook_url: Option<String>,
    pub polling_marker: Option<i64>,
    pub is_active: bool,
    pub history_limit: i32,
    pub max_bot_id: Option<i64>,
    pub max_bot_info: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API response — no sensitive fields
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BotResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    #[schema(example = "660e8400-e29b-41d4-a716-446655440000")]
    pub project_id: Uuid,
    #[schema(example = "My Bot")]
    pub name: String,
    pub event_mode: EventMode,
    #[schema(example = true)]
    pub is_active: bool,
    #[schema(example = 100)]
    pub history_limit: i32,
    #[schema(example = 12345)]
    pub max_bot_id: Option<i64>,
    /// Opaque bot info returned by the Max API
    pub max_bot_info: Option<serde_json::Value>,
    #[schema(example = "2026-03-14T12:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2026-03-14T12:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

impl From<BotRow> for BotResponse {
    fn from(b: BotRow) -> Self {
        Self {
            id: b.id,
            project_id: b.project_id,
            name: b.name,
            event_mode: b.event_mode,
            is_active: b.is_active,
            history_limit: b.history_limit,
            max_bot_id: b.max_bot_id,
            max_bot_info: b.max_bot_info,
            created_at: b.created_at,
            updated_at: b.updated_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "event_mode", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum EventMode {
    Webhook,
    Polling,
}
