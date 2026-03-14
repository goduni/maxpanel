use sqlx::PgPool;
use uuid::Uuid;
use crate::models::api_key::ApiKeyRow;

pub async fn create(
    pool: &PgPool,
    bot_id: Uuid,
    name: &str,
    key_hash: &str,
    key_prefix: &str,
) -> Result<ApiKeyRow, sqlx::Error> {
    sqlx::query_as!(
        ApiKeyRow,
        r#"INSERT INTO bot_api_keys (bot_id, name, key_hash, key_prefix)
           VALUES ($1, $2, $3, $4)
           RETURNING id, bot_id, name, key_hash, key_prefix, created_at, last_used_at, is_active"#,
        bot_id,
        name,
        key_hash,
        key_prefix,
    )
    .fetch_one(pool)
    .await
}

pub async fn list_for_bot(pool: &PgPool, bot_id: Uuid) -> Result<Vec<ApiKeyRow>, sqlx::Error> {
    sqlx::query_as!(
        ApiKeyRow,
        r#"SELECT id, bot_id, name, key_hash, key_prefix, created_at, last_used_at, is_active
           FROM bot_api_keys
           WHERE bot_id = $1 AND is_active = true
           ORDER BY created_at DESC"#,
        bot_id,
    )
    .fetch_all(pool)
    .await
}

pub async fn find_by_prefix(
    pool: &PgPool,
    bot_id: Uuid,
    key_prefix: &str,
) -> Result<Vec<ApiKeyRow>, sqlx::Error> {
    sqlx::query_as!(
        ApiKeyRow,
        r#"SELECT id, bot_id, name, key_hash, key_prefix, created_at, last_used_at, is_active
           FROM bot_api_keys
           WHERE bot_id = $1 AND key_prefix = $2 AND is_active = true"#,
        bot_id,
        key_prefix,
    )
    .fetch_all(pool)
    .await
}

pub async fn count_for_bot(pool: &PgPool, bot_id: Uuid) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM bot_api_keys WHERE bot_id = $1 AND is_active = true"#,
        bot_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete(pool: &PgPool, key_id: Uuid, bot_id: Uuid) -> Result<bool, sqlx::Error> {
    let result: sqlx::postgres::PgQueryResult = sqlx::query!(
        r#"DELETE FROM bot_api_keys WHERE id = $1 AND bot_id = $2"#,
        key_id,
        bot_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_last_used(pool: &PgPool, key_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE bot_api_keys SET last_used_at = now() WHERE id = $1"#,
        key_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}
