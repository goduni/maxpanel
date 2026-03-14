use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::app_state::AppState;
use crate::errors::AppError;
#[allow(unused_imports)]
use crate::errors::ErrorResponse; // used in OpenAPI schema annotations
use crate::extractors::{AuthUser, BotAuthContext};
use crate::handlers::common::{validate_request, PaginationQuery, PaginationInfo};
#[allow(unused_imports)]
use crate::handlers::common::OkResponse; // used in OpenAPI schema annotations
use crate::models::{BotResponse, EventMode};
use crate::services::bots as bot_svc;

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateBotRequest {
    #[validate(length(min = 1, max = 255))]
    #[schema(min_length = 1, max_length = 255, example = "My Bot")]
    pub name: String,
    #[validate(length(min = 1, max = 512))]
    #[schema(min_length = 1, max_length = 512)]
    pub access_token: String,
    pub event_mode: EventMode,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateBotRequest {
    #[validate(length(min = 1, max = 255))]
    #[schema(min_length = 1, max_length = 255, example = "My Bot")]
    pub name: Option<String>,
    #[validate(range(min = 0, max = 10000))]
    #[schema(minimum = 0, maximum = 10000, example = 100)]
    pub history_limit: Option<i32>,
}

#[derive(Serialize, ToSchema)]
pub struct BotListResponse {
    pub data: Vec<BotResponse>,
    pub pagination: PaginationInfo,
}

#[utoipa::path(
    post,
    path = "/api/organizations/{org_slug}/projects/{project_slug}/bots",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("project_slug" = String, Path, description = "Project slug"),
    ),
    request_body = CreateBotRequest,
    responses(
        (status = 201, description = "Bot created", body = BotResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Bots"
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, project_slug)): Path<(String, String)>,
    Json(req): Json<CreateBotRequest>,
) -> Result<(StatusCode, Json<BotResponse>), AppError> {
    validate_request(&req)?;
    let bot = bot_svc::create(
        &state.db,
        &state.config,
        &state.http_client,
        auth.user_id,
        &org_slug,
        &project_slug,
        &req.name,
        &req.access_token,
        req.event_mode,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(bot)))
}

#[utoipa::path(
    get,
    path = "/api/organizations/{org_slug}/projects/{project_slug}/bots",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("project_slug" = String, Path, description = "Project slug"),
        PaginationQuery,
    ),
    responses(
        (status = 200, description = "List of bots", body = BotListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Bots"
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, project_slug)): Path<(String, String)>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<BotListResponse>, AppError> {
    let (limit, offset) = q.resolve();
    let (bots, total) = bot_svc::list_for_project(
        &state.db,
        auth.user_id,
        &org_slug,
        &project_slug,
        limit,
        offset,
    )
    .await?;
    Ok(Json(BotListResponse {
        data: bots,
        pagination: PaginationInfo { total, offset, limit },
    }))
}

#[utoipa::path(
    get,
    path = "/api/organizations/{org_slug}/projects/{project_slug}/bots/{bot_id}",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("project_slug" = String, Path, description = "Project slug"),
        ("bot_id" = Uuid, Path, description = "Bot ID"),
    ),
    responses(
        (status = 200, description = "Bot details", body = BotResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Bots"
)]
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, project_slug, bot_id)): Path<(String, String, Uuid)>,
) -> Result<Json<BotResponse>, AppError> {
    let bot = bot_svc::get_by_id(&state.db, auth.user_id, &org_slug, &project_slug, bot_id).await?;
    Ok(Json(bot))
}

#[utoipa::path(
    patch,
    path = "/api/organizations/{org_slug}/projects/{project_slug}/bots/{bot_id}",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("project_slug" = String, Path, description = "Project slug"),
        ("bot_id" = Uuid, Path, description = "Bot ID"),
    ),
    request_body = UpdateBotRequest,
    responses(
        (status = 200, description = "Bot updated", body = BotResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Bots"
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, project_slug, bot_id)): Path<(String, String, Uuid)>,
    Json(req): Json<UpdateBotRequest>,
) -> Result<Json<BotResponse>, AppError> {
    validate_request(&req)?;
    let (org, org_role) = crate::services::organizations::resolve_org(&state.db, auth.user_id, &org_slug).await?;
    let project = crate::db::projects::find_by_slug(&state.db, org.id, &project_slug).await?.ok_or(AppError::Forbidden)?;
    crate::services::projects::require_project_admin_with_org_role(&state.db, auth.user_id, &project, org_role).await?;

    let updated = crate::db::bots::update_bot_fields(
        &state.db, bot_id, project.id, req.name.as_deref(), req.history_limit,
    ).await?.ok_or(AppError::NotFound)?;
    tracing::info!(target: "audit", actor_id = %auth.user_id, bot_id = %bot_id, "bot updated");
    Ok(Json(updated.into()))
}

#[utoipa::path(
    delete,
    path = "/api/organizations/{org_slug}/projects/{project_slug}/bots/{bot_id}",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("project_slug" = String, Path, description = "Project slug"),
        ("bot_id" = Uuid, Path, description = "Bot ID"),
    ),
    responses(
        (status = 200, description = "Bot deleted", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin required", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Bots"
)]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, project_slug, bot_id)): Path<(String, String, Uuid)>,
) -> Result<Json<OkResponse>, AppError> {
    let (org, org_role) = crate::services::organizations::resolve_org(&state.db, auth.user_id, &org_slug).await?;
    let project = crate::db::projects::find_by_slug(&state.db, org.id, &project_slug).await?.ok_or(AppError::Forbidden)?;
    crate::services::projects::require_project_admin_with_org_role(&state.db, auth.user_id, &project, org_role).await?;
    bot_svc::delete_by_id(&state.db, &state.config, &state.http_client, auth.user_id, &org_slug, &project_slug, bot_id).await?;
    Ok(Json(OkResponse { ok: true }))
}

// Flat bot endpoints (use BotAuthContext)

#[utoipa::path(
    post,
    path = "/api/bots/{bot_id}/start",
    params(("bot_id" = Uuid, Path, description = "Bot ID")),
    responses(
        (status = 200, description = "Bot started", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin required", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Bots"
)]
pub async fn start(
    State(state): State<AppState>,
    ctx: BotAuthContext,
) -> Result<Json<OkResponse>, AppError> {
    if !ctx.effective_role.can_manage() {
        return Err(AppError::Forbidden);
    }
    bot_svc::set_active(&state.db, ctx.auth_row.bot_id, true).await?;
    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    post,
    path = "/api/bots/{bot_id}/stop",
    params(("bot_id" = Uuid, Path, description = "Bot ID")),
    responses(
        (status = 200, description = "Bot stopped", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin required", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Bots"
)]
pub async fn stop(
    State(state): State<AppState>,
    ctx: BotAuthContext,
) -> Result<Json<OkResponse>, AppError> {
    if !ctx.effective_role.can_manage() {
        return Err(AppError::Forbidden);
    }
    bot_svc::set_active(&state.db, ctx.auth_row.bot_id, false).await?;
    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    post,
    path = "/api/bots/{bot_id}/verify",
    params(("bot_id" = Uuid, Path, description = "Bot ID")),
    responses(
        (status = 200, description = "Bot token verified, returns bot info from Max API", body = serde_json::Value, content_type = "application/json"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin required", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Bots"
)]
pub async fn verify(
    State(state): State<AppState>,
    ctx: BotAuthContext,
) -> Result<Json<serde_json::Value>, AppError> {
    if !ctx.effective_role.can_manage() {
        return Err(AppError::Forbidden);
    }
    // Use auth_row data directly to avoid redundant DB lookup.
    let token = bot_svc::decrypt_bot_token_from_auth(&state.config, &ctx.auth_row)?;
    let info = crate::services::max_api::get_my_info(&state.http_client, &state.config, &token).await?;

    tracing::info!(
        target: "audit",
        bot_id = %ctx.auth_row.bot_id,
        "bot token verified"
    );

    Ok(Json(info))
}
