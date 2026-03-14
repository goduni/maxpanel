use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
#[allow(unused_imports)]
use crate::errors::ErrorResponse; // used in OpenAPI schema annotations
use crate::extractors::BotAuthContext;
use crate::handlers::common::CursorPaginationInfo;
use crate::models::Event;
use crate::services::events as event_svc;

fn validate_direction(direction: &Option<String>) -> Result<(), AppError> {
    if let Some(dir) = direction {
        if dir != "inbound" && dir != "outbound" {
            return Err(AppError::BadRequest("direction must be 'inbound' or 'outbound'".into()));
        }
    }
    Ok(())
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct EventsQuery {
    /// Opaque cursor for pagination. Pass the `next_cursor` value from a previous response.
    pub cursor: Option<String>,
    /// Maximum number of items to return. Default: 50, max: 200.
    #[schema(minimum = 1, maximum = 200, example = 50)]
    pub limit: Option<i64>,
    /// Filter by event direction. Valid values: "inbound", "outbound". Omit to return all events.
    #[schema(example = "inbound")]
    pub direction: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct EventListResponse {
    pub data: Vec<Event>,
    pub pagination: CursorPaginationInfo,
}


#[utoipa::path(
    get,
    path = "/api/bots/{bot_id}/events",
    params(
        ("bot_id" = Uuid, Path, description = "Bot ID"),
        EventsQuery,
    ),
    responses(
        (status = 200, description = "List of events with cursor pagination", body = EventListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Events"
)]
pub async fn list_events(
    State(state): State<AppState>,
    ctx: BotAuthContext,
    Query(q): Query<EventsQuery>,
) -> Result<Json<EventListResponse>, AppError> {
    if !ctx.effective_role.can_read() {
        return Err(AppError::Forbidden);
    }

    validate_direction(&q.direction)?;

    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let cursor = match &q.cursor {
        Some(c) => Some(event_svc::decode_cursor(c)?),
        None => None,
    };

    let mut events = event_svc::list_for_bot(&state.db, ctx.auth_row.bot_id, q.direction.as_deref(), cursor, limit + 1).await?;
    let has_more = events.len() as i64 > limit;
    if has_more { events.pop(); }
    let next_cursor = if has_more {
        events.last().map(|e| event_svc::encode_cursor(&e.created_at, &e.id))
    } else {
        None
    };

    Ok(Json(EventListResponse {
        data: events,
        pagination: CursorPaginationInfo { next_cursor, has_more },
    }))
}

/// Optional query parameter to hint the event's creation time, enabling partition pruning.
#[derive(Deserialize, IntoParams, ToSchema)]
pub struct EventHintQuery {
    /// If provided, narrows the query to a +/- 1 day window around this timestamp for faster lookup. Format: ISO 8601 (e.g. 2026-03-14T12:00:00Z).
    #[schema(example = "2026-03-14T12:00:00Z")]
    pub created_at: Option<DateTime<Utc>>,
}

#[utoipa::path(
    get,
    path = "/api/bots/{bot_id}/events/{event_id}",
    params(
        ("bot_id" = Uuid, Path, description = "Bot ID"),
        ("event_id" = Uuid, Path, description = "Event ID"),
        EventHintQuery,
    ),
    responses(
        (status = 200, description = "Event details", body = Event),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Events"
)]
pub async fn get_event(
    State(state): State<AppState>,
    ctx: BotAuthContext,
    Path((_bot_id, event_id)): Path<(Uuid, Uuid)>,
    Query(hint): Query<EventHintQuery>,
) -> Result<Json<Event>, AppError> {
    if !ctx.effective_role.can_read() {
        return Err(AppError::Forbidden);
    }

    let event = event_svc::get_event(&state.db, event_id, ctx.auth_row.bot_id, hint.created_at).await?;
    Ok(Json(event))
}

#[utoipa::path(
    get,
    path = "/api/bots/{bot_id}/chats/{chat_id}/events",
    params(
        ("bot_id" = Uuid, Path, description = "Bot ID"),
        ("chat_id" = i64, Path, description = "Chat ID"),
        EventsQuery,
    ),
    responses(
        (status = 200, description = "Chat events with cursor pagination", body = EventListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Events"
)]
pub async fn list_chat_events(
    State(state): State<AppState>,
    ctx: BotAuthContext,
    Path((_bot_id, chat_id)): Path<(Uuid, i64)>,
    Query(q): Query<EventsQuery>,
) -> Result<Json<EventListResponse>, AppError> {
    if !ctx.effective_role.can_read() {
        return Err(AppError::Forbidden);
    }

    validate_direction(&q.direction)?;

    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let cursor = match &q.cursor {
        Some(c) => Some(event_svc::decode_cursor(c)?),
        None => None,
    };

    let mut events = event_svc::list_for_bot_chat(&state.db, ctx.auth_row.bot_id, chat_id, q.direction.as_deref(), cursor, limit + 1).await?;
    let has_more = events.len() as i64 > limit;
    if has_more { events.pop(); }
    let next_cursor = if has_more {
        events.last().map(|e| event_svc::encode_cursor(&e.created_at, &e.id))
    } else {
        None
    };

    Ok(Json(EventListResponse {
        data: events,
        pagination: CursorPaginationInfo { next_cursor, has_more },
    }))
}
