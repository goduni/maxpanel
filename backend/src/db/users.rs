use sqlx::PgPool;
use uuid::Uuid;

use crate::models::User;

pub async fn create(pool: &PgPool, email: &str, password_hash: &str, name: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"INSERT INTO users (email, password_hash, name)
           VALUES ($1, $2, $3)
           RETURNING id, email, password_hash, name, created_at, updated_at"#,
        email,
        password_hash,
        name,
    )
    .fetch_one(pool)
    .await
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT id, email, password_hash, name, created_at, updated_at FROM users WHERE email = $1",
        email,
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT id, email, password_hash, name, created_at, updated_at FROM users WHERE id = $1",
        id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_name(pool: &PgPool, id: Uuid, name: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"UPDATE users SET name = $2 WHERE id = $1
           RETURNING id, email, password_hash, name, created_at, updated_at"#,
        id,
        name,
    )
    .fetch_one(pool)
    .await
}

pub async fn update_password(pool: &PgPool, id: Uuid, password_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE users SET password_hash = $2 WHERE id = $1",
        id,
        password_hash,
    )
    .execute(pool)
    .await?;
    Ok(())
}
