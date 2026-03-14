use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, decode, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::db;
use crate::errors::AppError;
use crate::models::{AuthTokens, LoginResponse, User, UserResponse};
use crate::services::crypto;

const JWT_ISSUER: &str = "maxpanel";
const JWT_AUDIENCE: &str = "maxpanel";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub aud: String,
}

pub async fn register(
    pool: &PgPool,
    config: &Config,
    email: &str,
    password: &str,
    name: &str,
) -> Result<LoginResponse, AppError> {
    let email = email.trim().to_lowercase();
    let existing = db::users::find_by_email(pool, &email).await?;
    if existing.is_some() {
        return Err(AppError::Conflict("Registration failed".into()));
    }

    let password_hash = hash_password(password).await?;
    let user = db::users::create(pool, &email, &password_hash, name).await?;

    let tokens = generate_tokens(pool, config, &user).await?;

    Ok(LoginResponse {
        user: user.into(),
        tokens,
    })
}

pub async fn login(
    pool: &PgPool,
    config: &Config,
    email: &str,
    password: &str,
) -> Result<LoginResponse, AppError> {
    let email = email.trim().to_lowercase();
    let user = db::users::find_by_email(pool, &email)
        .await?
        .ok_or_else(|| {
            tracing::warn!(target: "audit", email = %email, "login failed: unknown email");
            AppError::Unauthorized
        })?;

    if let Err(e) = verify_password(password, &user.password_hash).await {
        tracing::warn!(target: "audit", email = %email, "login failed: invalid credentials");
        return Err(e);
    }

    let tokens = generate_tokens(pool, config, &user).await?;

    Ok(LoginResponse {
        user: user.into(),
        tokens,
    })
}

pub async fn refresh(
    pool: &PgPool,
    config: &Config,
    refresh_token: &str,
) -> Result<AuthTokens, AppError> {
    let token_hash = crypto::hash_token(&config.refresh_token_hmac_secret, refresh_token);

    // Atomic DELETE RETURNING avoids TOCTOU race between SELECT and DELETE.
    let row = sqlx::query!(
        r#"DELETE FROM refresh_tokens WHERE token_hash = $1
           RETURNING id, user_id, family_id, expires_at"#,
        token_hash,
    )
    .fetch_optional(pool)
    .await?;

    let row = match row {
        Some(r) => r,
        None => {
            tracing::warn!(target: "audit", "refresh token not found — possible token reuse attack");
            return Err(AppError::Unauthorized);
        }
    };

    if row.expires_at < Utc::now() {
        return Err(AppError::Unauthorized);
    }

    let user = db::users::find_by_id(pool, row.user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Create new token in same family
    let (access_token, new_refresh_token) = create_token_pair(pool, config, &user, Some(row.family_id)).await?;

    Ok(AuthTokens {
        access_token,
        refresh_token: new_refresh_token,
    })
}

pub async fn logout(pool: &PgPool, config: &Config, refresh_token: &str) -> Result<(), AppError> {
    let token_hash = crypto::hash_token(&config.refresh_token_hmac_secret, refresh_token);
    sqlx::query!("DELETE FROM refresh_tokens WHERE token_hash = $1", token_hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn logout_all(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query!("DELETE FROM refresh_tokens WHERE user_id = $1", user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn change_password(
    pool: &PgPool,
    user_id: Uuid,
    current_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    let user = db::users::find_by_id(pool, user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_password(current_password, &user.password_hash).await?;

    let new_hash = hash_password(new_password).await?;
    db::users::update_password(pool, user_id, &new_hash).await?;

    // Invalidate all sessions
    sqlx::query!("DELETE FROM refresh_tokens WHERE user_id = $1", user_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Get user profile through service layer.
pub async fn get_profile(pool: &PgPool, user_id: Uuid) -> Result<UserResponse, AppError> {
    let user = db::users::find_by_id(pool, user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(user.into())
}

/// Update user profile through service layer.
pub async fn update_profile(pool: &PgPool, user_id: Uuid, name: &str) -> Result<UserResponse, AppError> {
    let user = db::users::update_name(pool, user_id, name).await?;
    Ok(user.into())
}

pub fn verify_jwt(config: &Config, token: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_issuer(&[JWT_ISSUER]);
    validation.set_audience(&[JWT_AUDIENCE]);

    let token_data = decode::<Claims>(
        token,
        &config.jwt_decoding_key,
        &validation,
    )
    .map_err(|_| AppError::Unauthorized)?;

    Ok(token_data.claims)
}

async fn generate_tokens(pool: &PgPool, config: &Config, user: &User) -> Result<AuthTokens, AppError> {
    let (access_token, refresh_token) = create_token_pair(pool, config, user, None).await?;
    Ok(AuthTokens {
        access_token,
        refresh_token,
    })
}

async fn create_token_pair(
    pool: &PgPool,
    config: &Config,
    user: &User,
    family_id: Option<Uuid>,
) -> Result<(String, String), AppError> {
    let now = Utc::now();

    // Access token
    let claims = Claims {
        sub: user.id,
        iat: now.timestamp() as usize,
        exp: (now + Duration::seconds(config.jwt_access_ttl_secs as i64)).timestamp() as usize,
        iss: JWT_ISSUER.to_string(),
        aud: JWT_AUDIENCE.to_string(),
    };
    let access_token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &config.jwt_encoding_key,
    )
    .map_err(|e| AppError::Internal(e.into()))?;

    // Refresh token
    let refresh_token_raw = Uuid::new_v4().to_string();
    let token_hash = crypto::hash_token(&config.refresh_token_hmac_secret, &refresh_token_raw);
    let family = family_id.unwrap_or_else(Uuid::new_v4);
    let expires_at = now + Duration::days(config.refresh_token_ttl_days as i64);

    sqlx::query!(
        r#"INSERT INTO refresh_tokens (user_id, token_hash, family_id, expires_at)
           VALUES ($1, $2, $3, $4)"#,
        user.id,
        token_hash,
        family,
        expires_at,
    )
    .execute(pool)
    .await?;

    Ok((access_token, refresh_token_raw))
}

async fn hash_password(password: &str) -> Result<String, AppError> {
    let password = password.to_string();
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Password hashing failed: {}", e)))?;
        Ok(hash.to_string())
    })
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("spawn_blocking failed: {}", e)))?
}

async fn verify_password(password: &str, hash: &str) -> Result<(), AppError> {
    let password = password.to_string();
    let hash = hash.to_string();
    tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&hash)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid password hash: {}", e)))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| AppError::Unauthorized)?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!("spawn_blocking failed: {}", e)))?
}
