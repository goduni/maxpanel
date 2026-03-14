use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use super::OrgRole;

/// Database row — never Serialize directly
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InviteRow {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub email: String,
    pub role: OrgRole,
    pub token_hash: String,
    pub invited_by: Uuid,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// API response — no token_hash
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InviteResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    #[schema(example = "invitee@example.com")]
    pub email: String,
    pub role: OrgRole,
    #[schema(example = "660e8400-e29b-41d4-a716-446655440000")]
    pub invited_by: Uuid,
    #[schema(example = "2026-03-21T12:00:00Z")]
    pub expires_at: DateTime<Utc>,
    #[schema(example = "2026-03-14T12:00:00Z")]
    pub created_at: DateTime<Utc>,
}

impl From<InviteRow> for InviteResponse {
    fn from(i: InviteRow) -> Self {
        Self {
            id: i.id,
            email: i.email,
            role: i.role,
            invited_by: i.invited_by,
            expires_at: i.expires_at,
            created_at: i.created_at,
        }
    }
}
