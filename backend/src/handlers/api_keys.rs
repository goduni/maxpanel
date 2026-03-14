use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::db;
use crate::errors::AppError;
#[allow(unused_imports)]
use crate::errors::ErrorResponse; // used in OpenAPI schema annotations
use crate::extractors::BotAuthContext;
use crate::handlers::common::OkResponse;
use crate::models::api_key::{ApiKeyCreateResponse, ApiKeyResponse};
use crate::services::api_keys::{self, MAX_KEYS_PER_BOT};

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/api/bots/{bot_id}/api-keys",
    params(("bot_id" = Uuid, Path, description = "Bot ID")),
    request_body = CreateApiKeyRequest,
    responses(
        (status = 201, description = "API key created", body = ApiKeyCreateResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 409, description = "Max keys limit reached", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "API Keys"
)]
pub async fn create_api_key(
    State(state): State<AppState>,
    ctx: BotAuthContext,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<ApiKeyCreateResponse>), AppError> {
    if !ctx.effective_role.can_send_api() {
        return Err(AppError::Forbidden);
    }

    let name = req.name.trim().to_string();
    if name.is_empty() || name.len() > 100 {
        return Err(AppError::BadRequest("Name must be 1-100 characters".into()));
    }

    let count = db::api_keys::count_for_bot(&state.db, ctx.auth_row.bot_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    if count >= MAX_KEYS_PER_BOT {
        return Err(AppError::Conflict(format!(
            "Maximum {} API keys per bot",
            MAX_KEYS_PER_BOT
        )));
    }

    let (key, prefix) = api_keys::generate_api_key();
    let hash = api_keys::hash_api_key(&state.config.bot_api_key_hmac_secret, &key);

    let row = db::api_keys::create(&state.db, ctx.auth_row.bot_id, &name, &hash, &prefix)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    tracing::info!(
        target: "audit",
        actor_id = %ctx.user_id,
        bot_id = %ctx.auth_row.bot_id,
        key_id = %row.id,
        "api key created"
    );

    Ok((
        StatusCode::CREATED,
        Json(ApiKeyCreateResponse {
            id: row.id,
            name: row.name,
            key,
            key_prefix: row.key_prefix,
            created_at: row.created_at,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/bots/{bot_id}/api-keys",
    params(("bot_id" = Uuid, Path, description = "Bot ID")),
    responses(
        (status = 200, description = "List of API keys", body = Vec<ApiKeyResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "API Keys"
)]
pub async fn list_api_keys(
    State(state): State<AppState>,
    ctx: BotAuthContext,
) -> Result<Json<Vec<ApiKeyResponse>>, AppError> {
    if !ctx.effective_role.can_read() {
        return Err(AppError::Forbidden);
    }

    let rows = db::api_keys::list_for_bot(&state.db, ctx.auth_row.bot_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let keys: Vec<ApiKeyResponse> = rows.into_iter().map(ApiKeyResponse::from).collect();
    Ok(Json(keys))
}

#[utoipa::path(
    delete,
    path = "/api/bots/{bot_id}/api-keys/{key_id}",
    params(
        ("bot_id" = Uuid, Path, description = "Bot ID"),
        ("key_id" = Uuid, Path, description = "API key ID"),
    ),
    responses(
        (status = 200, description = "Key deleted", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Key not found", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "API Keys"
)]
pub async fn delete_api_key(
    State(state): State<AppState>,
    ctx: BotAuthContext,
    Path((_bot_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<OkResponse>, AppError> {
    if !ctx.effective_role.can_send_api() {
        return Err(AppError::Forbidden);
    }

    let deleted = db::api_keys::delete(&state.db, key_id, ctx.auth_row.bot_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if !deleted {
        return Err(AppError::NotFound);
    }

    tracing::info!(
        target: "audit",
        actor_id = %ctx.user_id,
        bot_id = %ctx.auth_row.bot_id,
        key_id = %key_id,
        "api key deleted"
    );

    Ok(Json(OkResponse { ok: true }))
}
