use axum::extract::{Json, State};
use serde::Deserialize;
use utoipa::ToSchema;
#[allow(unused_imports)]
use uuid::Uuid; // used in OpenAPI params annotation

use crate::db;
use crate::errors::AppError;
use crate::extractors::api_key_auth::ApiKeyAuth;
use crate::services::ingestion;
use crate::app_state::AppState;

#[allow(unused_imports)]
use crate::errors::ErrorResponse;

const MAX_EVENTS_PER_REQUEST: usize = 100;

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct IngestOutgoingRequest {
    pub events: Vec<OutgoingEventPayload>,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct OutgoingEventPayload {
    /// HTTP method used (e.g. "POST").
    pub method: String,
    /// API path (e.g. "/messages").
    pub path: String,
    /// HTTP status code from the Max API.
    pub status_code: u16,
    /// Optional request body sent to the API.
    pub request_body: Option<serde_json::Value>,
    /// Optional response body received from the API.
    pub response_body: Option<serde_json::Value>,
    /// Optional Unix timestamp in milliseconds. Defaults to now.
    pub timestamp: Option<i64>,
}

#[utoipa::path(
    post,
    path = "/api/bots/{bot_id}/outgoing-events",
    params(("bot_id" = Uuid, Path, description = "Bot ID")),
    request_body = IngestOutgoingRequest,
    responses(
        (status = 200, description = "Events ingested", body = crate::handlers::common::OkResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Invalid API key", body = ErrorResponse),
        (status = 429, description = "Rate limited", body = ErrorResponse),
    ),
    security(("api_key" = [])),
    tag = "Ingestion"
)]
pub async fn ingest_outgoing(
    State(state): State<AppState>,
    auth: ApiKeyAuth,
    Json(req): Json<IngestOutgoingRequest>,
) -> Result<Json<crate::handlers::common::OkResponse>, AppError> {
    if req.events.is_empty() {
        return Err(AppError::BadRequest("No events provided".into()));
    }
    if req.events.len() > MAX_EVENTS_PER_REQUEST {
        return Err(AppError::BadRequest(format!(
            "Maximum {} events per request",
            MAX_EVENTS_PER_REQUEST
        )));
    }

    for (i, event) in req.events.iter().enumerate() {
        if !(100..=599).contains(&event.status_code) {
            return Err(AppError::BadRequest(format!(
                "events[{}].status_code must be between 100 and 599", i
            )));
        }
    }

    // Rate limit per bot
    if !state.rate_limiter.check(
        &format!("ingestion:{}", auth.bot_id),
        60.0,
        10.0,
    ) {
        return Err(AppError::RateLimited);
    }

    // Load bot for max_bot_id
    let bot = db::bots::find_by_id(&state.db, auth.bot_id)
        .await?
        .ok_or(AppError::NotFound)?;

    tracing::info!(
        target: "audit",
        bot_id = %auth.bot_id,
        api_key_id = %auth.api_key.id,
        event_count = req.events.len(),
        "outgoing events ingested"
    );

    let new_events: Vec<_> = req
        .events
        .iter()
        .map(|event| {
            ingestion::build_outbound_event(
                auth.bot_id,
                bot.max_bot_id,
                "ingestion_api",
                &event.method,
                &event.path,
                event.status_code,
                event.request_body.clone(),
                event.response_body.clone(),
                event.timestamp,
            )
        })
        .collect();

    ingestion::ingest_outbound_batch(&state.db, auth.bot_id, new_events).await?;

    Ok(Json(crate::handlers::common::OkResponse { ok: true }))
}
