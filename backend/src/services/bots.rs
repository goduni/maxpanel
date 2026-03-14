use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::db;
use crate::errors::AppError;
use crate::models::{BotResponse, BotRow, EventMode, OrgRole};
use crate::services::{crypto, organizations};

pub async fn create(
    pool: &PgPool,
    config: &Config,
    http_client: &reqwest::Client,
    user_id: Uuid,
    org_slug: &str,
    project_slug: &str,
    name: &str,
    access_token: &str,
    event_mode: EventMode,
) -> Result<BotResponse, AppError> {
    let (org, org_role) = organizations::resolve_org(pool, user_id, org_slug).await?;
    let project = db::projects::find_by_slug(pool, org.id, project_slug)
        .await?
        .ok_or(AppError::NotFound)?;

    if org_role.privilege_level() < OrgRole::Admin.privilege_level() {
        let member = db::projects::get_member_role(pool, project.id, user_id).await?;
        if member.is_none() {
            return Err(AppError::NotFound);
        }
    }

    crate::services::projects::require_project_admin_with_org_role(pool, user_id, &project, org_role).await?;

    // Verify token by calling getMyInfo
    let raw_bot_info = crate::services::max_api::get_my_info(http_client, config, access_token).await?;
    let max_bot_id = raw_bot_info.get("user_id").and_then(|v| v.as_i64());

    // Filter to known safe fields to avoid storing unexpected data
    let bot_info = serde_json::json!({
        "user_id": raw_bot_info.get("user_id"),
        "name": raw_bot_info.get("name"),
        "username": raw_bot_info.get("username"),
        "is_bot": raw_bot_info.get("is_bot"),
    });

    let webhook_secret = match event_mode {
        EventMode::Webhook => Some(Uuid::new_v4()),
        EventMode::Polling => None,
    };

    let webhook_url = webhook_secret.map(|secret| {
        format!("{}/webhooks/{}", config.webhook_base_url, secret)
    });

    // Use a transaction: insert with placeholder encryption, then re-encrypt
    // with the real bot ID for proper HKDF key derivation.
    let mut tx = pool.begin().await?;

    // Insert with placeholder — we need the bot ID for HKDF derivation
    let placeholder_enc: Vec<u8> = vec![];
    let placeholder_nonce: Vec<u8> = vec![];

    let bot = db::bots::create_in_tx(
        &mut *tx,
        project.id,
        name,
        &placeholder_enc,
        &placeholder_nonce,
        event_mode,
        webhook_secret,
        webhook_url.as_deref(),
        max_bot_id,
        Some(&bot_info),
    )
    .await?;

    // Now encrypt with the real bot ID
    let (ciphertext, nonce) = crypto::encrypt_token(
        &config.bot_token_encryption_key,
        bot.id,
        bot.key_version,
        access_token,
    );
    sqlx::query!(
        "UPDATE bots SET access_token_enc = $2, access_token_nonce = $3 WHERE id = $1",
        bot.id,
        &ciphertext,
        &nonce,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // Subscribe webhook if needed (outside transaction — external API call)
    if event_mode == EventMode::Webhook && let Some(ref url) = webhook_url {
        let _ = crate::services::max_api::subscribe_webhook(
            http_client, config, access_token, url,
        )
        .await;
    }

    tracing::info!(
        target: "audit",
        actor_id = %user_id,
        bot_id = %bot.id,
        project_id = %project.id,
        event_mode = ?event_mode,
        "bot created"
    );

    Ok(bot.into())
}

pub async fn list_for_project(
    pool: &PgPool,
    user_id: Uuid,
    org_slug: &str,
    project_slug: &str,
    limit: i64,
    offset: i64,
) -> Result<(Vec<BotResponse>, i64), AppError> {
    let (org, org_role) = organizations::resolve_org(pool, user_id, org_slug).await?;
    let project = db::projects::find_by_slug(pool, org.id, project_slug)
        .await?
        .ok_or(AppError::NotFound)?;

    if org_role.privilege_level() < OrgRole::Admin.privilege_level() {
        let member = db::projects::get_member_role(pool, project.id, user_id).await?;
        if member.is_none() {
            return Err(AppError::NotFound);
        }
    }

    // Run both queries concurrently
    let (bots_result, total_result) = tokio::join!(
        db::bots::list_for_project(pool, project.id, limit, offset),
        db::bots::count_for_project(pool, project.id),
    );
    let bot_rows = bots_result?;
    let total = total_result?;
    let responses: Vec<BotResponse> = bot_rows.into_iter().map(Into::into).collect();
    Ok((responses, total))
}

pub async fn get_by_id(
    pool: &PgPool,
    user_id: Uuid,
    org_slug: &str,
    project_slug: &str,
    bot_id: Uuid,
) -> Result<BotResponse, AppError> {
    let (org, org_role) = organizations::resolve_org(pool, user_id, org_slug).await?;
    let project = db::projects::find_by_slug(pool, org.id, project_slug)
        .await?
        .ok_or(AppError::NotFound)?;

    if org_role.privilege_level() < OrgRole::Admin.privilege_level() {
        let member = db::projects::get_member_role(pool, project.id, user_id).await?;
        if member.is_none() {
            return Err(AppError::NotFound);
        }
    }

    let bot = db::bots::find_by_id_for_response(pool, bot_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if bot.project_id != project.id {
        return Err(AppError::NotFound);
    }

    Ok(bot.into())
}

pub async fn update_name(
    pool: &PgPool,
    bot_id: Uuid,
    name: &str,
) -> Result<BotResponse, AppError> {
    let bot = db::bots::update_name(pool, bot_id, name).await?;
    Ok(bot.into())
}

pub async fn set_active(
    pool: &PgPool,
    bot_id: Uuid,
    active: bool,
) -> Result<(), AppError> {
    db::bots::set_active(pool, bot_id, active).await?;
    Ok(())
}

pub async fn delete_by_id(
    pool: &PgPool,
    config: &Config,
    http_client: &reqwest::Client,
    user_id: Uuid,
    org_slug: &str,
    project_slug: &str,
    bot_id: Uuid,
) -> Result<(), AppError> {
    // Auth check + single fetch
    let (org, _org_role) = organizations::resolve_org(pool, user_id, org_slug).await?;
    let project = db::projects::find_by_slug(pool, org.id, project_slug)
        .await?
        .ok_or(AppError::NotFound)?;
    let bot = db::bots::find_by_id(pool, bot_id)
        .await?
        .ok_or(AppError::NotFound)?;
    if bot.project_id != project.id {
        return Err(AppError::NotFound);
    }
    delete(pool, config, http_client, &bot).await
}

pub async fn delete(
    pool: &PgPool,
    config: &Config,
    http_client: &reqwest::Client,
    bot: &BotRow,
) -> Result<(), AppError> {
    // Unsubscribe webhook if applicable
    if bot.event_mode == EventMode::Webhook && let Ok(token) = decrypt_bot_token(config, bot) {
        let _ = crate::services::max_api::unsubscribe_webhook(
            http_client, config, &token,
        )
        .await;
    }

    tracing::info!(
        target: "audit",
        bot_id = %bot.id,
        project_id = %bot.project_id,
        "bot deleted"
    );

    db::bots::delete(pool, bot.id).await?;
    Ok(())
}

pub fn decrypt_bot_token(config: &Config, bot: &BotRow) -> Result<String, AppError> {
    crypto::decrypt_token(
        &config.bot_token_encryption_key,
        bot.id,
        bot.key_version,
        &bot.access_token_enc,
        &bot.access_token_nonce,
    )
    .map_err(AppError::Internal)
}

/// Decrypt bot token using BotPollingContext, avoiding fetching the full BotRow.
pub fn decrypt_bot_token_for_polling(config: &Config, bot_id: Uuid, ctx: &crate::db::bots::BotPollingContext) -> Result<String, AppError> {
    crypto::decrypt_token(
        &config.bot_token_encryption_key,
        bot_id,
        ctx.key_version,
        &ctx.access_token_enc,
        &ctx.access_token_nonce,
    )
    .map_err(AppError::Internal)
}

/// Decrypt bot token using BotAuthRow data, avoiding a redundant DB lookup.
pub fn decrypt_bot_token_from_auth(config: &Config, auth_row: &crate::db::bots::BotAuthRow) -> Result<String, AppError> {
    crypto::decrypt_token(
        &config.bot_token_encryption_key,
        auth_row.bot_id,
        auth_row.key_version,
        &auth_row.access_token_enc,
        &auth_row.access_token_nonce,
    )
    .map_err(AppError::Internal)
}
