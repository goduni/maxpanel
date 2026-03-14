use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize, ToSchema)]
pub struct Organization {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    #[schema(example = "My Organization")]
    pub name: String,
    #[schema(example = "my-org")]
    pub slug: String,
    #[schema(example = "2026-03-14T12:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2026-03-14T12:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, ToSchema)]
pub struct OrganizationMember {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub organization_id: Uuid,
    #[schema(example = "660e8400-e29b-41d4-a716-446655440000")]
    pub user_id: Uuid,
    pub role: OrgRole,
    #[schema(example = "2026-03-14T12:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "John Doe")]
    pub user_name: Option<String>,
    #[schema(example = "user@example.com")]
    pub user_email: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "org_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum OrgRole {
    Owner,
    Admin,
    Member,
}

impl OrgRole {
    pub fn privilege_level(&self) -> u8 {
        match self {
            OrgRole::Owner => 3,
            OrgRole::Admin => 2,
            OrgRole::Member => 1,
        }
    }

    pub fn can_assign(&self, target: OrgRole) -> bool {
        self.privilege_level() > target.privilege_level()
    }
}
