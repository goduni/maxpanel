use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
#[allow(unused_imports)]
use uuid::Uuid; // used in OpenAPI schema annotations

use crate::app_state::AppState;
use crate::errors::AppError;
#[allow(unused_imports)]
use crate::errors::ErrorResponse; // used in OpenAPI schema annotations
use crate::extractors::BotAuthContext;
use crate::handlers::common::CursorPaginationInfo;
use crate::models::BotChat;
use crate::services::{bots as bot_svc, bot_chats as bot_chats_svc};

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct BotChatQuery {
    /// Opaque cursor for pagination. Pass the `next_cursor` value from a previous response.
    pub cursor: Option<String>,
    /// Maximum number of items to return. Default: 50, max: 200.
    #[schema(minimum = 1, maximum = 200, example = 50)]
    pub limit: Option<i64>,
    /// Search by chat title or chat ID.
    #[schema(example = "john")]
    pub search: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct BotChatListResponse {
    pub data: Vec<BotChat>,
    pub pagination: CursorPaginationInfo,
}

#[derive(Serialize, ToSchema)]
pub struct SyncChatsResponse {
    /// Number of chats synced from Max API
    #[schema(example = 15)]
    pub synced: i64,
}

#[derive(Serialize, ToSchema)]
pub struct SyncHistoryResponse {
    /// Number of messages synced from Max API
    #[schema(example = 100)]
    pub synced: i64,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct HistoryQuery {
    /// Unix timestamp (millis) — fetch messages before this time
    pub to: Option<i64>,
    /// Max messages to return (1-100, default 50)
    #[schema(minimum = 1, maximum = 100, example = 50)]
    pub count: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/api/bots/{bot_id}/chats",
    params(
        ("bot_id" = Uuid, Path, description = "Bot ID"),
        BotChatQuery,
    ),
    responses(
        (status = 200, description = "List of bot chats", body = BotChatListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Chats"
)]
pub async fn list_bot_chats(
    State(state): State<AppState>,
    ctx: BotAuthContext,
    Query(q): Query<BotChatQuery>,
) -> Result<Json<BotChatListResponse>, AppError> {
    if !ctx.effective_role.can_read() {
        return Err(AppError::Forbidden);
    }
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let cursor = match &q.cursor {
        Some(c) => Some(bot_chats_svc::decode_chat_cursor(c)?),
        None => None,
    };

    let mut chats = bot_chats_svc::list_chats(&state.db, ctx.auth_row.bot_id, cursor, limit + 1, q.search.as_deref()).await?;
    let has_more = chats.len() as i64 > limit;
    if has_more { chats.pop(); }
    let next_cursor = if has_more {
        chats.last().map(|c| {
            let sort_time = c.last_event_at.unwrap_or(c.synced_at);
            bot_chats_svc::encode_chat_cursor(&sort_time, c.chat_id)
        })
    } else {
        None
    };

    Ok(Json(BotChatListResponse {
        data: chats,
        pagination: CursorPaginationInfo { next_cursor, has_more },
    }))
}

#[utoipa::path(
    post,
    path = "/api/bots/{bot_id}/chats/sync",
    params(("bot_id" = Uuid, Path, description = "Bot ID")),
    responses(
        (status = 200, description = "Chats synced from Max API", body = SyncChatsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin required", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Chats"
)]
pub async fn sync_chats(
    State(state): State<AppState>,
    ctx: BotAuthContext,
) -> Result<Json<SyncChatsResponse>, AppError> {
    if !ctx.effective_role.can_manage() {
        return Err(AppError::Forbidden);
    }

    let token = bot_svc::decrypt_bot_token_from_auth(&state.config, &ctx.auth_row)?;
    let synced = bot_chats_svc::sync_chats(
        &state.db,
        &state.config,
        &state.http_client,
        ctx.auth_row.bot_id,
        &token,
    )
    .await?;

    Ok(Json(SyncChatsResponse { synced }))
}

#[utoipa::path(
    post,
    path = "/api/bots/{bot_id}/chats/{chat_id}/sync-history",
    params(
        ("bot_id" = Uuid, Path, description = "Bot ID"),
        ("chat_id" = i64, Path, description = "Chat ID"),
    ),
    responses(
        (status = 200, description = "Messages synced", body = SyncHistoryResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin required", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Chats"
)]
pub async fn sync_chat_history(
    State(state): State<AppState>,
    ctx: BotAuthContext,
    Path((_bot_id, chat_id)): Path<(Uuid, i64)>,
) -> Result<Json<SyncHistoryResponse>, AppError> {
    if !ctx.effective_role.can_manage() {
        return Err(AppError::Forbidden);
    }
    // Rate limit: 1 sync per bot per 60s
    if !state.rate_limiter.check(
        &format!("sync_history:{}", ctx.auth_row.bot_id),
        1.0,
        1.0 / 60.0,
    ) {
        return Err(AppError::RateLimited);
    }

    let history_limit = ctx.auth_row.history_limit;
    if history_limit <= 0 {
        return Ok(Json(SyncHistoryResponse { synced: 0 }));
    }

    let token = bot_svc::decrypt_bot_token_from_auth(&state.config, &ctx.auth_row)?;
    let synced = bot_chats_svc::sync_chat_history(
        &state.db,
        &state.config,
        &state.http_client,
        ctx.auth_row.bot_id,
        ctx.auth_row.max_bot_id,
        chat_id,
        &token,
        history_limit,
    ).await?;

    tracing::info!(
        target: "audit",
        bot_id = %ctx.auth_row.bot_id,
        chat_id = chat_id,
        synced = synced,
        "history synced"
    );
    Ok(Json(SyncHistoryResponse { synced }))
}

#[utoipa::path(
    get,
    path = "/api/bots/{bot_id}/chats/{chat_id}/history",
    params(
        ("bot_id" = Uuid, Path, description = "Bot ID"),
        ("chat_id" = i64, Path, description = "Chat ID"),
        HistoryQuery,
    ),
    responses(
        (status = 200, description = "Messages from Max API"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Chats"
)]
pub async fn proxy_chat_history(
    State(state): State<AppState>,
    ctx: BotAuthContext,
    Path((_bot_id, chat_id)): Path<(Uuid, i64)>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !ctx.effective_role.can_read() {
        return Err(AppError::Forbidden);
    }
    // Rate limit: 30 requests per bot per second (matches Max API limit)
    if !state.rate_limiter.check(
        &format!("proxy_history:{}", ctx.auth_row.bot_id),
        30.0,
        10.0,
    ) {
        return Err(AppError::RateLimited);
    }
    if let Some(to) = q.to {
        if to < 0 {
            return Err(AppError::BadRequest("'to' must be a positive timestamp".into()));
        }
    }

    let token = bot_svc::decrypt_bot_token_from_auth(&state.config, &ctx.auth_row)?;
    let count = q.count.unwrap_or(50).clamp(1, 100);
    let result = bot_chats_svc::proxy_chat_history(
        &state.config,
        &state.http_client,
        &token,
        chat_id,
        q.to,
        count,
    ).await?;

    Ok(Json(result))
}