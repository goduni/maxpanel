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
use crate::extractors::AuthUser;
use crate::handlers::common::{validate_request, validate_slug, PaginationQuery, PaginationInfo};
#[allow(unused_imports)]
use crate::handlers::common::OkResponse; // used in OpenAPI schema annotations
use crate::models::{OrgRole, Organization, OrganizationMember};
use crate::services::organizations as org_svc;

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateOrgRequest {
    #[validate(length(min = 1, max = 255))]
    #[schema(min_length = 1, max_length = 255, example = "My Organization")]
    pub name: String,
    #[validate(length(min = 2, max = 100))]
    #[schema(min_length = 2, max_length = 100, example = "my-org")]
    pub slug: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateOrgRequest {
    #[validate(length(min = 1, max = 255))]
    #[schema(min_length = 1, max_length = 255, example = "My Organization")]
    pub name: String,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TransferOwnershipRequest {
    pub new_owner_id: Uuid,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateMemberRoleRequest {
    pub role: OrgRole,
}

#[derive(Serialize, ToSchema)]
pub struct OrganizationListResponse {
    pub data: Vec<Organization>,
    pub pagination: PaginationInfo,
}

#[derive(Serialize, ToSchema)]
pub struct MembersResponse {
    pub data: Vec<OrganizationMember>,
}

#[utoipa::path(
    post,
    path = "/api/organizations",
    request_body = CreateOrgRequest,
    responses(
        (status = 201, description = "Organization created", body = Organization),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Slug already taken", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Organizations"
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateOrgRequest>,
) -> Result<(StatusCode, Json<Organization>), AppError> {
    validate_request(&req)?;
    validate_slug(&req.slug)?;
    let org = org_svc::create(&state.db, auth.user_id, &req.name, &req.slug).await?;
    Ok((StatusCode::CREATED, Json(org)))
}

#[utoipa::path(
    get,
    path = "/api/organizations",
    params(PaginationQuery),
    responses(
        (status = 200, description = "List of organizations", body = OrganizationListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Organizations"
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<OrganizationListResponse>, AppError> {
    let (limit, offset) = q.resolve();
    let (orgs, total) = org_svc::list_for_user(&state.db, auth.user_id, limit, offset).await?;
    Ok(Json(OrganizationListResponse {
        data: orgs,
        pagination: PaginationInfo { total, offset, limit },
    }))
}

#[utoipa::path(
    get,
    path = "/api/organizations/{org_slug}",
    params(("org_slug" = String, Path, description = "Organization slug")),
    responses(
        (status = 200, description = "Organization details", body = Organization),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Organizations"
)]
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_slug): Path<String>,
) -> Result<Json<Organization>, AppError> {
    let org = org_svc::get_by_slug(&state.db, auth.user_id, &org_slug).await?;
    Ok(Json(org))
}

#[utoipa::path(
    patch,
    path = "/api/organizations/{org_slug}",
    params(("org_slug" = String, Path, description = "Organization slug")),
    request_body = UpdateOrgRequest,
    responses(
        (status = 200, description = "Organization updated", body = Organization),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Organizations"
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_slug): Path<String>,
    Json(req): Json<UpdateOrgRequest>,
) -> Result<Json<Organization>, AppError> {
    validate_request(&req)?;
    let org = org_svc::update(&state.db, auth.user_id, &org_slug, &req.name).await?;
    Ok(Json(org))
}

#[utoipa::path(
    delete,
    path = "/api/organizations/{org_slug}",
    params(("org_slug" = String, Path, description = "Organization slug")),
    responses(
        (status = 200, description = "Organization deleted", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Only owner can delete", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Organizations"
)]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_slug): Path<String>,
) -> Result<Json<OkResponse>, AppError> {
    org_svc::delete(&state.db, auth.user_id, &org_slug).await?;
    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    post,
    path = "/api/organizations/{org_slug}/transfer-ownership",
    params(("org_slug" = String, Path, description = "Organization slug")),
    request_body = TransferOwnershipRequest,
    responses(
        (status = 200, description = "Ownership transferred", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Only owner can transfer", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Organizations"
)]
pub async fn transfer_ownership(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_slug): Path<String>,
    Json(req): Json<TransferOwnershipRequest>,
) -> Result<Json<OkResponse>, AppError> {
    org_svc::transfer_ownership(&state.db, auth.user_id, &org_slug, req.new_owner_id).await?;
    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    get,
    path = "/api/organizations/{org_slug}/members",
    params(("org_slug" = String, Path, description = "Organization slug")),
    responses(
        (status = 200, description = "List of members", body = MembersResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Organizations"
)]
pub async fn list_members(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_slug): Path<String>,
) -> Result<Json<MembersResponse>, AppError> {
    let members = org_svc::list_members(&state.db, auth.user_id, &org_slug).await?;
    Ok(Json(MembersResponse { data: members }))
}

#[utoipa::path(
    patch,
    path = "/api/organizations/{org_slug}/members/{user_id}",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    request_body = UpdateMemberRoleRequest,
    responses(
        (status = 200, description = "Role updated", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Organizations"
)]
pub async fn update_member_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, user_id)): Path<(String, Uuid)>,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> Result<Json<OkResponse>, AppError> {
    org_svc::update_member_role(&state.db, auth.user_id, &org_slug, user_id, req.role).await?;
    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    delete,
    path = "/api/organizations/{org_slug}/members/{user_id}",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "Member removed", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Organizations"
)]
pub async fn remove_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, user_id)): Path<(String, Uuid)>,
) -> Result<Json<OkResponse>, AppError> {
    org_svc::remove_member(&state.db, auth.user_id, &org_slug, user_id).await?;
    Ok(Json(OkResponse { ok: true }))
}
