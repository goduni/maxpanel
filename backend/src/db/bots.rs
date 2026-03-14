use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{BotRow, EventMode};

pub async fn create_in_tx(
    tx: &mut sqlx::PgConnection,
    project_id: Uuid,
    name: &str,
    access_token_enc: &[u8],
    access_token_nonce: &[u8],
    event_mode: EventMode,
    webhook_secret: Option<Uuid>,
    webhook_url: Option<&str>,
    max_bot_id: Option<i64>,
    max_bot_info: Option<&serde_json::Value>,
) -> Result<BotRow, sqlx::Error> {
    sqlx::query_as!(
        BotRow,
        r#"INSERT INTO bots (project_id, name, access_token_enc, access_token_nonce, event_mode,
                             webhook_secret, webhook_url, max_bot_id, max_bot_info)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING id, project_id, name, access_token_enc, access_token_nonce,
                     key_version, event_mode as "event_mode: EventMode",
                     webhook_secret, webhook_url, polling_marker, is_active,
                     history_limit, max_bot_id, max_bot_info, created_at, updated_at"#,
        project_id,
        name,
        access_token_enc,
        access_token_nonce,
        event_mode as EventMode,
        webhook_secret,
        webhook_url,
        max_bot_id,
        max_bot_info,
    )
    .fetch_one(&mut *tx)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<BotRow>, sqlx::Error> {
    sqlx::query_as!(
        BotRow,
        r#"SELECT id, project_id, name, access_token_enc, access_token_nonce,
                  key_version, event_mode as "event_mode: EventMode",
                  webhook_secret, webhook_url, polling_marker, is_active,
                  history_limit, max_bot_id, max_bot_info, created_at, updated_at
           FROM bots WHERE id = $1"#,
        id,
    )
    .fetch_optional(pool)
    .await
}

/// Lightweight find_by_id that returns BotListRow (no encrypted fields).
/// Use this when you only need to build a BotResponse.
pub async fn find_by_id_for_response(pool: &PgPool, id: Uuid) -> Result<Option<BotListRow>, sqlx::Error> {
    sqlx::query_as::<_, BotListRow>(
        r#"SELECT id, project_id, name, event_mode,
                  is_active, history_limit, max_bot_id, max_bot_info, created_at, updated_at
           FROM bots WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_webhook_secret(pool: &PgPool, secret: Uuid) -> Result<Option<BotRow>, sqlx::Error> {
    sqlx::query_as!(
        BotRow,
        r#"SELECT id, project_id, name, access_token_enc, access_token_nonce,
                  key_version, event_mode as "event_mode: EventMode",
                  webhook_secret, webhook_url, polling_marker, is_active,
                  history_limit, max_bot_id, max_bot_info, created_at, updated_at
           FROM bots WHERE webhook_secret = $1 AND is_active = true"#,
        secret,
    )
    .fetch_optional(pool)
    .await
}

/// Lightweight row for listing bots — omits encrypted token fields.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BotListRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub event_mode: EventMode,
    pub is_active: bool,
    pub history_limit: i32,
    pub max_bot_id: Option<i64>,
    pub max_bot_info: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<BotListRow> for crate::models::BotResponse {
    fn from(b: BotListRow) -> Self {
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

pub async fn list_for_project(pool: &PgPool, project_id: Uuid, limit: i64, offset: i64) -> Result<Vec<BotListRow>, sqlx::Error> {
    sqlx::query_as!(
        BotListRow,
        r#"SELECT id, project_id, name, event_mode as "event_mode: EventMode",
                  is_active, history_limit, max_bot_id, max_bot_info, created_at, updated_at
           FROM bots WHERE project_id = $1
           ORDER BY created_at DESC
           LIMIT $2 OFFSET $3"#,
        project_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
}

pub async fn count_for_project(pool: &PgPool, project_id: Uuid) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM bots WHERE project_id = $1"#,
        project_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update_name(pool: &PgPool, id: Uuid, name: &str) -> Result<BotRow, sqlx::Error> {
    sqlx::query_as!(
        BotRow,
        r#"UPDATE bots SET name = $2 WHERE id = $1
           RETURNING id, project_id, name, access_token_enc, access_token_nonce,
                     key_version, event_mode as "event_mode: EventMode",
                     webhook_secret, webhook_url, polling_marker, is_active,
                     history_limit, max_bot_id, max_bot_info, created_at, updated_at"#,
        id,
        name,
    )
    .fetch_one(pool)
    .await
}

pub async fn set_active(pool: &PgPool, id: Uuid, active: bool) -> Result<(), sqlx::Error> {
    sqlx::query!("UPDATE bots SET is_active = $2 WHERE id = $1", id, active)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_polling_marker(pool: &PgPool, id: Uuid, marker: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("UPDATE bots SET polling_marker = $2 WHERE id = $1", id, marker)
        .execute(pool)
        .await?;
    Ok(())
}

/// Update bot fields in a single query with RETURNING. Returns None if bot not found or wrong project.
pub async fn update_bot_fields(
    pool: &PgPool,
    id: Uuid,
    project_id: Uuid,
    name: Option<&str>,
    history_limit: Option<i32>,
) -> Result<Option<BotListRow>, sqlx::Error> {
    sqlx::query_as!(
        BotListRow,
        r#"UPDATE bots
           SET name = COALESCE($3, name),
               history_limit = COALESCE($4, history_limit),
               updated_at = now()
           WHERE id = $1 AND project_id = $2
           RETURNING id, project_id, name, event_mode as "event_mode: EventMode",
                     is_active, history_limit, max_bot_id, max_bot_info,
                     created_at, updated_at"#,
        id,
        project_id,
        name,
        history_limit,
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_active_polling_ids(pool: &PgPool) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        r#"SELECT id FROM bots WHERE is_active = true AND event_mode = 'polling'"#,
    )
    .fetch_all(pool)
    .await
}

#[derive(sqlx::FromRow)]
pub struct BotPollingContext {
    pub is_active: bool,
    pub access_token_enc: Vec<u8>,
    pub access_token_nonce: Vec<u8>,
    pub key_version: i32,
    pub polling_marker: Option<i64>,
}

pub async fn find_polling_context(pool: &PgPool, id: Uuid) -> Result<Option<BotPollingContext>, sqlx::Error> {
    sqlx::query_as::<_, BotPollingContext>(
        "SELECT is_active, access_token_enc, access_token_nonce, key_version, polling_marker FROM bots WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    // Events must be deleted first (no FK cascade on partitioned table).
    // Wrapped in a transaction for atomicity.
    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM events WHERE bot_id = $1", id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM bots WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// Single JOIN query for BotAuthContext — resolves bot → project → org → membership
pub async fn resolve_bot_auth(
    pool: &PgPool,
    bot_id: Uuid,
    user_id: Uuid,
) -> Result<Option<BotAuthRow>, sqlx::Error> {
    sqlx::query_as!(
        BotAuthRow,
        r#"SELECT
            b.id as bot_id, b.project_id, b.name as bot_name,
            b.access_token_enc, b.access_token_nonce, b.key_version,
            b.event_mode as "event_mode: EventMode",
            b.webhook_secret, b.webhook_url, b.polling_marker,
            b.is_active, b.history_limit, b.max_bot_id, b.max_bot_info,
            b.created_at as bot_created_at, b.updated_at as bot_updated_at,
            p.organization_id, p.name as project_name, p.slug as project_slug,
            o.id as org_id, o.name as org_name, o.slug as org_slug,
            om.role as "org_role: Option<crate::models::OrgRole>",
            pm.role as "proj_role: Option<crate::models::ProjectRole>"
           FROM bots b
           JOIN projects p ON p.id = b.project_id
           JOIN organizations o ON o.id = p.organization_id
           LEFT JOIN organization_members om ON om.organization_id = o.id AND om.user_id = $2
           LEFT JOIN project_members pm ON pm.project_id = p.id AND pm.user_id = $2
           WHERE b.id = $1"#,
        bot_id,
        user_id,
    )
    .fetch_optional(pool)
    .await
}

#[derive(Debug, sqlx::FromRow)]
pub struct BotAuthRow {
    pub bot_id: Uuid,
    pub project_id: Uuid,
    pub bot_name: String,
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
    pub bot_created_at: chrono::DateTime<chrono::Utc>,
    pub bot_updated_at: chrono::DateTime<chrono::Utc>,
    pub organization_id: Uuid,
    pub project_name: String,
    pub project_slug: String,
    pub org_id: Uuid,
    pub org_name: String,
    pub org_slug: String,
    pub org_role: Option<crate::models::OrgRole>,
    pub proj_role: Option<crate::models::ProjectRole>,
}
