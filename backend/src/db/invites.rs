use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{InviteRow, OrgRole};

pub async fn create(
    pool: &PgPool,
    org_id: Uuid,
    email: &str,
    role: OrgRole,
    token_hash: &str,
    invited_by: Uuid,
    expires_at: DateTime<Utc>,
) -> Result<InviteRow, sqlx::Error> {
    sqlx::query_as!(
        InviteRow,
        r#"INSERT INTO invites (organization_id, email, role, token_hash, invited_by, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, organization_id, email, role as "role: OrgRole",
                     token_hash, invited_by, expires_at, accepted_at, revoked_at, created_at"#,
        org_id,
        email,
        role as OrgRole,
        token_hash,
        invited_by,
        expires_at,
    )
    .fetch_one(pool)
    .await
}

pub async fn find_by_token_hash(pool: &PgPool, token_hash: &str) -> Result<Option<InviteRow>, sqlx::Error> {
    sqlx::query_as!(
        InviteRow,
        r#"SELECT id, organization_id, email, role as "role: OrgRole",
                  token_hash, invited_by, expires_at, accepted_at, revoked_at, created_at
           FROM invites
           WHERE token_hash = $1
             AND accepted_at IS NULL
             AND revoked_at IS NULL
             AND expires_at > now()"#,
        token_hash,
    )
    .fetch_optional(pool)
    .await
}

pub async fn revoke(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!("UPDATE invites SET revoked_at = now() WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_pending_for_org(pool: &PgPool, org_id: Uuid) -> Result<Vec<InviteRow>, sqlx::Error> {
    sqlx::query_as!(
        InviteRow,
        r#"SELECT id, organization_id, email, role as "role: OrgRole",
                  token_hash, invited_by, expires_at, accepted_at, revoked_at, created_at
           FROM invites
           WHERE organization_id = $1
             AND accepted_at IS NULL
             AND revoked_at IS NULL
             AND expires_at > now()
           ORDER BY created_at DESC"#,
        org_id,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_for_email(pool: &PgPool, email: &str) -> Result<Vec<InviteRow>, sqlx::Error> {
    sqlx::query_as!(
        InviteRow,
        r#"SELECT id, organization_id, email, role as "role: OrgRole",
                  token_hash, invited_by, expires_at, accepted_at, revoked_at, created_at
           FROM invites
           WHERE email = $1
             AND accepted_at IS NULL
             AND revoked_at IS NULL
             AND expires_at > now()
           ORDER BY created_at DESC"#,
        email,
    )
    .fetch_all(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<InviteRow>, sqlx::Error> {
    sqlx::query_as!(
        InviteRow,
        r#"SELECT id, organization_id, email, role as "role: OrgRole",
                  token_hash, invited_by, expires_at, accepted_at, revoked_at, created_at
           FROM invites WHERE id = $1"#,
        id,
    )
    .fetch_optional(pool)
    .await
}
