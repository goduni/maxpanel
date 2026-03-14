use axum::extract::{Json, State};
use serde::Deserialize;
use utoipa::ToSchema;

#[allow(unused_imports)]
use uuid::Uuid; // used in OpenAPI params annotation

use crate::db;
use crate::errors::AppError;
use crate::extractors::api_key_auth::ApiKeyAuth;
use crate::services::{bots as bot_svc, ingestion, max_api};
use crate::app_state::AppState;

#[allow(unused_imports)]
use crate::errors::ErrorResponse;

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct GatewayRequest {
    /// HTTP method (GET, POST, PUT, PATCH, DELETE).
    pub method: String,
    /// API path (e.g. "/messages").
    pub path: String,
    /// Optional JSON request body.
    pub body: Option<serde_json::Value>,
}

#[utoipa::path(
    post,
    path = "/api/bots/{bot_id}/gateway",
    params(("bot_id" = Uuid, Path, description = "Bot ID")),
    request_body = GatewayRequest,
    responses(
        (status = 200, description = "Max API response", body = serde_json::Value),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Invalid API key", body = ErrorResponse),
        (status = 422, description = "Max API error", body = ErrorResponse),
    ),
    security(("api_key" = [])),
    tag = "Gateway"
)]
pub async fn gateway(
    State(state): State<AppState>,
    auth: ApiKeyAuth,
    Json(req): Json<GatewayRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let valid_methods = ["GET", "POST", "PUT", "PATCH", "DELETE"];
    if !valid_methods.contains(&req.method.to_uppercase().as_str()) {
        return Err(AppError::BadRequest("method must be GET, POST, PUT, PATCH, or DELETE".into()));
    }

    if !state.rate_limiter.check(
        &format!("gateway:{}", auth.bot_id),
        60.0,
        10.0,
    ) {
        return Err(AppError::RateLimited);
    }

    let bot = db::bots::find_by_id(&state.db, auth.bot_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if !bot.is_active {
        return Err(AppError::BadRequest("Bot is not active".into()));
    }

    let token = bot_svc::decrypt_bot_token(&state.config, &bot)?;

    tracing::info!(
        target: "audit",
        bot_id = %auth.bot_id,
        api_key_id = %auth.api_key.id,
        method = %req.method,
        path = %req.path,
        "gateway proxy call"
    );

    let request_body_for_event = req.body.clone();
    let (status, response_body) = max_api::proxy_call_raw(
        &state.http_client,
        &state.config,
        &token,
        &req.method,
        &req.path,
        req.body,
    )
    .await?;

    // Save outbound event (fire-and-forget)
    let pool = state.db.clone();
    let bot_id = auth.bot_id;
    let max_bot_id = bot.max_bot_id;
    let method = req.method;
    let path = req.path;
    let resp_clone = response_body.clone();
    tokio::spawn(async move {
        if let Err(e) = ingestion::ingest_outbound(
            &pool,
            bot_id,
            max_bot_id,
            "gateway",
            &method,
            &path,
            status,
            request_body_for_event,
            Some(resp_clone),
            None,
        )
        .await
        {
            tracing::error!("Failed to save outbound gateway event: {e}");
        }
    });

    if (200..300).contains(&status) {
        Ok(Json(response_body))
    } else {
        Err(AppError::MaxApiError {
            status,
            body: response_body,
        })
    }
}
