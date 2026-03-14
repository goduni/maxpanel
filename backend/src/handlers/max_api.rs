use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use utoipa::ToSchema;
#[allow(unused_imports)]
use uuid::Uuid; // used in OpenAPI schema annotations

use crate::app_state::AppState;
use crate::errors::AppError;
#[allow(unused_imports)]
use crate::errors::ErrorResponse; // used in OpenAPI schema annotations
use crate::extractors::BotAuthContext;
use crate::services::{bots as bot_svc, ingestion, max_api};

/// Proxy request to the Max messenger API.
///
/// Allows calling any Max API method through the bot's access token.
/// The `method` field specifies the HTTP method, `path` specifies the
/// Max API endpoint path (e.g. `/messages`, `/chats`), and `body`
/// contains the optional JSON payload.
///
/// See the [Max API documentation](https://dev.max.ru/docs-api) for
/// available endpoints and their request/response formats.
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RawProxyRequest {
    /// HTTP method (GET, POST, PUT, PATCH, DELETE)
    #[schema(example = "GET")]
    pub method: String,
    /// Max API path (e.g. /messages, /chats, /chats/{chat_id}/members)
    #[schema(example = "/chats")]
    pub path: String,
    /// Optional JSON request body
    pub body: Option<serde_json::Value>,
}

#[utoipa::path(
    post,
    path = "/api/bots/{bot_id}/max",
    params(("bot_id" = Uuid, Path, description = "Bot ID")),
    request_body = RawProxyRequest,
    responses(
        (status = 200, description = "Proxied response from Max API", body = serde_json::Value, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden — requires project editor or org admin", body = ErrorResponse),
        (status = 422, description = "Max API returned a client error", body = ErrorResponse),
        (status = 502, description = "Max API returned a server error", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Max API"
)]
pub async fn raw_proxy(
    State(state): State<AppState>,
    ctx: BotAuthContext,
    Json(req): Json<RawProxyRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !ctx.effective_role.can_send_api() {
        return Err(AppError::Forbidden);
    }

    let token = bot_svc::decrypt_bot_token_from_auth(&state.config, &ctx.auth_row)?;

    tracing::info!(
        target: "audit",
        actor_id = %ctx.user_id,
        bot_id = %ctx.auth_row.bot_id,
        method = %req.method,
        path = %req.path,
        "max API proxy call"
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
    let bot_id = ctx.auth_row.bot_id;
    let max_bot_id = ctx.auth_row.max_bot_id;
    let method = req.method;
    let path = req.path;
    let resp_body_clone = response_body.clone();
    tokio::spawn(async move {
        if let Err(e) = ingestion::ingest_outbound(
            &pool, bot_id, max_bot_id, "proxy",
            &method, &path, status,
            request_body_for_event, Some(resp_body_clone), None,
        ).await {
            tracing::error!("Failed to save outbound proxy event: {e}");
        }
    });

    if (200..300).contains(&status) {
        Ok(Json(response_body))
    } else {
        Err(AppError::MaxApiError { status, body: response_body })
    }
}
