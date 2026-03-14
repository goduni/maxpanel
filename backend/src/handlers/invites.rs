use axum::extract::{Path, State};
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
use crate::extractors::AuthUser;
use crate::handlers::common::validate_request;
#[allow(unused_imports)]
use crate::handlers::common::OkResponse; // used in OpenAPI schema annotations
use crate::models::{InviteResponse, OrgRole};
use crate::services::invites as invite_svc;
use crate::services::auth as auth_svc;

const INVITE_ACCEPT_RATE_LIMIT_MAX: f64 = 5.0;
const INVITE_ACCEPT_RATE_LIMIT_REFILL: f64 = 0.1;

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateInviteRequest {
    #[validate(email)]
    #[schema(example = "invitee@example.com", format = "email")]
    pub email: String,
    pub role: OrgRole,
}

#[derive(Serialize, ToSchema)]
pub struct CreateInviteResponse {
    pub invite: InviteResponse,
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct InviteListResponse {
    pub data: Vec<InviteResponse>,
}

#[utoipa::path(
    post,
    path = "/api/organizations/{org_slug}/invites",
    params(("org_slug" = String, Path, description = "Organization slug")),
    request_body = CreateInviteRequest,
    responses(
        (status = 201, description = "Invite created", body = CreateInviteResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin required", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Invites"
)]
pub async fn create_invite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_slug): Path<String>,
    Json(req): Json<CreateInviteRequest>,
) -> Result<(StatusCode, Json<CreateInviteResponse>), AppError> {
    validate_request(&req)?;
    let (invite, token) = invite_svc::create_invite(
        &state.db,
        &state.config,
        auth.user_id,
        &org_slug,
        &req.email,
        req.role,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(CreateInviteResponse { invite, token })))
}

#[utoipa::path(
    get,
    path = "/api/organizations/{org_slug}/invites",
    params(("org_slug" = String, Path, description = "Organization slug")),
    responses(
        (status = 200, description = "Pending invites", body = InviteListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Invites"
)]
pub async fn list_pending(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_slug): Path<String>,
) -> Result<Json<InviteListResponse>, AppError> {
    let invites = invite_svc::list_pending(&state.db, auth.user_id, &org_slug).await?;
    Ok(Json(InviteListResponse { data: invites }))
}

#[utoipa::path(
    delete,
    path = "/api/organizations/{org_slug}/invites/{invite_id}",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("invite_id" = Uuid, Path, description = "Invite ID"),
    ),
    responses(
        (status = 200, description = "Invite revoked", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin required", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Invites"
)]
pub async fn revoke_invite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, invite_id)): Path<(String, Uuid)>,
) -> Result<Json<OkResponse>, AppError> {
    invite_svc::revoke_invite(&state.db, auth.user_id, &org_slug, invite_id).await?;
    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    post,
    path = "/api/invites/{token}/accept",
    params(("token" = String, Path, description = "Invite token")),
    responses(
        (status = 200, description = "Invite accepted", body = OkResponse),
        (status = 400, description = "Email mismatch or expired", body = ErrorResponse),
        (status = 429, description = "Too many requests. Retry-After header indicates wait time", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Invites"
)]
/// Accept an invite using its token.
///
/// The token is passed in the URL path rather than a request body. This is a
/// deliberate design choice: invite links are shared out-of-band (e.g. email)
/// and the token alone does not grant access — the user must also be
/// authenticated and the token's email must match the authenticated user.
pub async fn accept_invite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(token): Path<String>,
) -> Result<Json<OkResponse>, AppError> {
    if !state.rate_limiter.check(&format!("invite_accept:{}", auth.user_id), INVITE_ACCEPT_RATE_LIMIT_MAX, INVITE_ACCEPT_RATE_LIMIT_REFILL) {
        return Err(AppError::RateLimited);
    }
    let user = auth_svc::get_profile(&state.db, auth.user_id).await?;
    invite_svc::accept_invite(&state.db, &state.config, auth.user_id, &user.email, &token).await?;
    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    get,
    path = "/api/auth/me/invites",
    responses(
        (status = 200, description = "User's pending invites", body = InviteListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Invites"
)]
pub async fn list_my_invites(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<InviteListResponse>, AppError> {
    let user = auth_svc::get_profile(&state.db, auth.user_id).await?;
    let invites = invite_svc::list_for_user(&state.db, &user.email).await?;
    Ok(Json(InviteListResponse { data: invites }))
}
