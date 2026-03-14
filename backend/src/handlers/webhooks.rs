use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::handlers::common::OkResponse;
use crate::services::ingestion;

const WEBHOOK_RATE_LIMIT_MAX: f64 = 60.0;
const WEBHOOK_RATE_LIMIT_REFILL: f64 = 10.0;

#[allow(unused_imports)]
use crate::errors::ErrorResponse; // used in OpenAPI schema annotations

/// Handle an incoming webhook from the Max API.
///
/// Authentication is via an unguessable UUID secret embedded in the webhook URL
/// (e.g. `/webhooks/{secret}`). The Max API does not support request signature
/// verification, so the secret URL is the sole authentication mechanism.
/// The secret is generated server-side with `Uuid::new_v4()` and never exposed
/// to end-users.
#[utoipa::path(
    post,
    path = "/webhooks/{webhook_secret}",
    params(("webhook_secret" = Uuid, Path, description = "Bot webhook secret (UUID)")),
    request_body(content = serde_json::Value, description = "One or more Max API update objects"),
    responses(
        (status = 200, description = "Webhook processed", body = OkResponse),
        (status = 404, description = "Unknown webhook secret", body = ErrorResponse),
        (status = 429, description = "Too many requests. Retry-After header indicates wait time", body = ErrorResponse),
    ),
    tag = "Webhooks"
)]
pub async fn handle_webhook(
    State(state): State<AppState>,
    Path(webhook_secret): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<OkResponse>, AppError> {
    let bot = crate::db::bots::find_by_webhook_secret(&state.db, webhook_secret)
        .await?
        .ok_or(AppError::NotFound)?;

    if !state.rate_limiter.check(&format!("webhook:{}", bot.id), WEBHOOK_RATE_LIMIT_MAX, WEBHOOK_RATE_LIMIT_REFILL) {
        return Err(AppError::RateLimited);
    }

    tracing::debug!(
        target: "audit",
        bot_id = %bot.id,
        "webhook received"
    );

    // Body can be a single update or an array; cap at 100 updates per request
    const MAX_UPDATES_PER_WEBHOOK: usize = 100;
    let updates: Vec<serde_json::Value> = match body {
        serde_json::Value::Array(arr) => arr.into_iter().take(MAX_UPDATES_PER_WEBHOOK).collect(),
        other => vec![other],
    };

    ingestion::ingest_updates(&state.db, bot.id, updates, "webhook").await?;

    Ok(Json(OkResponse { ok: true }))
}
