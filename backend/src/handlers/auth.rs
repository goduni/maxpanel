use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::app_state::AppState;
use crate::errors::AppError;
#[allow(unused_imports)]
use crate::errors::ErrorResponse; // used in OpenAPI schema annotations
use crate::extractors::AuthUser;
use crate::handlers::common::validate_request;
#[allow(unused_imports)]
use crate::handlers::common::OkResponse; // used in OpenAPI schema annotations
use crate::models::{AuthTokens, LoginResponse, UserResponse};
use crate::services::auth as auth_svc;

const AUTH_RATE_LIMIT_MAX: f64 = 5.0;
const AUTH_RATE_LIMIT_REFILL: f64 = 0.1;

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RegisterRequest {
    #[validate(email)]
    #[schema(example = "user@example.com", format = "email")]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    #[schema(min_length = 8, max_length = 128, example = "secureP@ss1")]
    pub password: String,
    #[validate(length(min = 1, max = 255))]
    #[schema(min_length = 1, max_length = 255, example = "John Doe")]
    pub name: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    #[validate(email)]
    #[schema(example = "user@example.com", format = "email")]
    pub email: String,
    #[schema(example = "secureP@ss1")]
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8, max = 128))]
    #[schema(min_length = 8, max_length = 128)]
    pub new_password: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateMeRequest {
    #[validate(length(min = 1, max = 255))]
    #[schema(min_length = 1, max_length = 255, example = "John Doe")]
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered", body = LoginResponse),
        (status = 409, description = "Email already registered", body = ErrorResponse),
        (status = 429, description = "Too many requests. Retry-After header indicates wait time", body = ErrorResponse),
    ),
    tag = "Auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), AppError> {
    validate_request(&req)?;
    // Rate limit on auth endpoints
    if !state.rate_limiter.check(&format!("register:{}", req.email), AUTH_RATE_LIMIT_MAX, AUTH_RATE_LIMIT_REFILL) {
        return Err(AppError::RateLimited);
    }
    let resp = auth_svc::register(&state.db, &state.config, &req.email, &req.password, &req.name).await?;
    Ok((StatusCode::CREATED, Json(resp)))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 429, description = "Too many requests. Retry-After header indicates wait time", body = ErrorResponse),
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    validate_request(&req)?;
    // Rate limit per email
    if !state.rate_limiter.check(&format!("login:{}", req.email), AUTH_RATE_LIMIT_MAX, AUTH_RATE_LIMIT_REFILL) {
        return Err(AppError::RateLimited);
    }
    let resp = auth_svc::login(&state.db, &state.config, &req.email, &req.password).await?;
    Ok(Json(resp))
}

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Tokens refreshed", body = AuthTokens),
        (status = 401, description = "Invalid or expired refresh token", body = ErrorResponse),
        (status = 429, description = "Too many requests. Retry-After header indicates wait time", body = ErrorResponse),
    ),
    tag = "Auth"
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<AuthTokens>, AppError> {
    // Per-token-prefix rate limit on refresh
    if !state.rate_limiter.check(&format!("refresh:{}", &req.refresh_token.get(..8).unwrap_or("x")), AUTH_RATE_LIMIT_MAX, AUTH_RATE_LIMIT_REFILL) {
        return Err(AppError::RateLimited);
    }
    let tokens = auth_svc::refresh(&state.db, &state.config, &req.refresh_token).await?;
    Ok(Json(tokens))
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Logged out", body = OkResponse),
    ),
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<LogoutRequest>,
) -> Result<Json<OkResponse>, AppError> {
    if !state.rate_limiter.check(&format!("logout:{}", &req.refresh_token.get(..8).unwrap_or("x")), 10.0, 0.5) {
        return Err(AppError::RateLimited);
    }
    auth_svc::logout(&state.db, &state.config, &req.refresh_token).await?;
    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    post,
    path = "/api/auth/logout-all",
    responses(
        (status = 200, description = "All sessions invalidated", body = OkResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Auth"
)]
pub async fn logout_all(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<OkResponse>, AppError> {
    auth_svc::logout_all(&state.db, auth.user_id).await?;
    Ok(Json(OkResponse { ok: true }))
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "Current user profile", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Auth"
)]
pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserResponse>, AppError> {
    let user = auth_svc::get_profile(&state.db, auth.user_id).await?;
    Ok(Json(user))
}

#[utoipa::path(
    patch,
    path = "/api/auth/me",
    request_body = UpdateMeRequest,
    responses(
        (status = 200, description = "Profile updated", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Auth"
)]
pub async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateMeRequest>,
) -> Result<Json<UserResponse>, AppError> {
    validate_request(&req)?;
    let user = auth_svc::update_profile(&state.db, auth.user_id, &req.name).await?;
    Ok(Json(user))
}

#[utoipa::path(
    post,
    path = "/api/auth/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed", body = OkResponse),
        (status = 401, description = "Invalid current password", body = ErrorResponse),
        (status = 429, description = "Too many requests. Retry-After header indicates wait time", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Auth"
)]
pub async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<OkResponse>, AppError> {
    validate_request(&req)?;
    if !state.rate_limiter.check(&format!("change_pw:{}", auth.user_id), AUTH_RATE_LIMIT_MAX, AUTH_RATE_LIMIT_REFILL) {
        return Err(AppError::RateLimited);
    }
    auth_svc::change_password(&state.db, auth.user_id, &req.current_password, &req.new_password).await?;
    Ok(Json(OkResponse { ok: true }))
}
