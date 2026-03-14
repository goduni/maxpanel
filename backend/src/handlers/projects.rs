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
use crate::models::{Project, ProjectMember, ProjectRole};
use crate::services::projects as proj_svc;

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateProjectRequest {
    #[validate(length(min = 1, max = 255))]
    #[schema(min_length = 1, max_length = 255, example = "My Project")]
    pub name: String,
    #[validate(length(min = 2, max = 100))]
    #[schema(min_length = 2, max_length = 100, example = "my-project")]
    pub slug: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateProjectRequest {
    #[validate(length(min = 1, max = 255))]
    #[schema(min_length = 1, max_length = 255, example = "My Project")]
    pub name: String,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    pub role: ProjectRole,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateMemberRoleRequest {
    pub role: ProjectRole,
}

#[derive(Serialize, ToSchema)]
pub struct ProjectListResponse {
    pub data: Vec<Project>,
    pub pagination: PaginationInfo,
}

#[derive(Serialize, ToSchema)]
pub struct ProjectMembersResponse {
    pub data: Vec<ProjectMember>,
}

#[utoipa::path(
    post,
    path = "/api/organizations/{org_slug}/projects",
    params(("org_slug" = String, Path, description = "Organization slug")),
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created", body = Project),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Slug already taken in this org", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Projects"
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_slug): Path<String>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), AppError> {
    validate_request(&req)?;
    validate_slug(&req.slug)?;
    let project = proj_svc::create(&state.db, auth.user_id, &org_slug, &req.name, &req.slug).await?;
    Ok((StatusCode::CREATED, Json(project)))
}

#[utoipa::path(
    get,
    path = "/api/organizations/{org_slug}/projects",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        PaginationQuery,
    ),
    responses(
        (status = 200, description = "List of projects", body = ProjectListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Projects"
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_slug): Path<String>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<ProjectListResponse>, AppError> {
    let (limit, offset) = q.resolve();
    let (projects, total) = proj_svc::list_for_org(&state.db, auth.user_id, &org_slug, limit, offset).await?;
    Ok(Json(ProjectListResponse {
        data: projects,
        pagination: PaginationInfo { total, offset, limit },
    }))
}

#[utoipa::path(
    get,
    path = "/api/organizations/{org_slug}/projects/{project_slug}",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("project_slug" = String, Path, description = "Project slug"),
    ),
    responses(
        (status = 200, description = "Project details", body = Project),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Projects"
)]
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, project_slug)): Path<(String, String)>,
) -> Result<Json<Project>, AppError> {
    let project = proj_svc::get_by_slug(&state.db, auth.user_id, &org_slug, &project_slug).await?;
    Ok(Json(project))
}

#[utoipa::path(
    patch,
    path = "/api/organizations/{org_slug}/projects/{project_slug}",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("project_slug" = String, Path, description = "Project slug"),
    ),
    request_body = UpdateProjectRequest,
    responses(
        (status = 200, description = "Project updated", body = Project),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin required", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Projects"
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, project_slug)): Path<(String, String)>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, AppError> {
    validate_request(&req)?;
    let project = proj_svc::update(&state.db, auth.user_id, &org_slug, &project_slug, &req.name).await?;
    Ok(Json(project))
}

#[utoipa::path(
    delete,
    path = "/api/organizations/{org_slug}/projects/{project_slug}",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("project_slug" = String, Path, description = "Project slug"),
    ),
    responses(
        (status = 200, description = "Project deleted", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin required", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Projects"
)]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, project_slug)): Path<(String, String)>,
) -> Result<Json<OkResponse>, AppError> {
    proj_svc::delete(&state.db, auth.user_id, &org_slug, &project_slug).await?;
    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    get,
    path = "/api/organizations/{org_slug}/projects/{project_slug}/members",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("project_slug" = String, Path, description = "Project slug"),
    ),
    responses(
        (status = 200, description = "List of project members", body = ProjectMembersResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Projects"
)]
pub async fn list_members(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, project_slug)): Path<(String, String)>,
) -> Result<Json<ProjectMembersResponse>, AppError> {
    let members = proj_svc::list_members(&state.db, auth.user_id, &org_slug, &project_slug).await?;
    Ok(Json(ProjectMembersResponse { data: members }))
}

#[utoipa::path(
    post,
    path = "/api/organizations/{org_slug}/projects/{project_slug}/members",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("project_slug" = String, Path, description = "Project slug"),
    ),
    request_body = AddMemberRequest,
    responses(
        (status = 201, description = "Member added", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin required", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Projects"
)]
pub async fn add_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, project_slug)): Path<(String, String)>,
    Json(req): Json<AddMemberRequest>,
) -> Result<(StatusCode, Json<OkResponse>), AppError> {
    proj_svc::add_member(&state.db, auth.user_id, &org_slug, &project_slug, req.user_id, req.role).await?;
    Ok((StatusCode::CREATED, Json(OkResponse { ok: true })))
}

#[utoipa::path(
    patch,
    path = "/api/organizations/{org_slug}/projects/{project_slug}/members/{user_id}",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("project_slug" = String, Path, description = "Project slug"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    request_body = UpdateMemberRoleRequest,
    responses(
        (status = 200, description = "Role updated", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin required", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Projects"
)]
pub async fn update_member_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, project_slug, user_id)): Path<(String, String, Uuid)>,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> Result<Json<OkResponse>, AppError> {
    proj_svc::update_member_role(&state.db, auth.user_id, &org_slug, &project_slug, user_id, req.role).await?;
    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    delete,
    path = "/api/organizations/{org_slug}/projects/{project_slug}/members/{user_id}",
    params(
        ("org_slug" = String, Path, description = "Organization slug"),
        ("project_slug" = String, Path, description = "Project slug"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "Member removed", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin required", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Projects"
)]
pub async fn remove_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_slug, project_slug, user_id)): Path<(String, String, Uuid)>,
) -> Result<Json<OkResponse>, AppError> {
    proj_svc::remove_member(&state.db, auth.user_id, &org_slug, &project_slug, user_id).await?;
    Ok(Json(OkResponse { ok: true }))
}
