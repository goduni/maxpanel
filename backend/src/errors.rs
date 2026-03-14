use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Validation error")]
    Validation(Vec<FieldError>),

    #[error("{0}")]
    BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not found")]
    NotFound,

    #[error("{0}")]
    Conflict(String),

    #[error("Max API error: {status}")]
    MaxApiError {
        status: u16,
        body: serde_json::Value,
    },

    #[error("Rate limited")]
    RateLimited,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FieldError {
    #[schema(example = "email")]
    pub field: String,
    #[schema(example = "Invalid email address")]
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    #[schema(example = "VALIDATION_ERROR")]
    pub code: String,
    #[schema(example = "Validation failed")]
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
    /// Full upstream response body (only present for MAX_API_ERROR)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details, upstream) = match &self {
            AppError::Validation(fields) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Validation failed".to_string(),
                Some(fields.clone()),
                None,
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                msg.clone(),
                None,
                None,
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Unauthorized".to_string(),
                None,
                None,
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Forbidden".to_string(),
                None,
                None,
            ),
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "Resource not found".to_string(),
                None,
                None,
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                "CONFLICT",
                msg.clone(),
                None,
                None,
            ),
            AppError::MaxApiError { status, body } => {
                let http_status = if *status >= 500 {
                    StatusCode::BAD_GATEWAY
                } else {
                    StatusCode::UNPROCESSABLE_ENTITY
                };
                let msg = body
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Max API error")
                    .to_string();
                (http_status, "MAX_API_ERROR", msg, None, Some(body.clone()))
            }
            AppError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
                "Too many requests".to_string(),
                None,
                None,
            ),
            AppError::Internal(err) => {
                tracing::error!(error = %err, "Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error".to_string(),
                    None,
                    None,
                )
            }
        };

        let body = ErrorResponse {
            error: ErrorBody {
                code: code.to_string(),
                message,
                details,
                upstream,
            },
        };

        let mut response = (status, Json(body)).into_response();
        if matches!(self, AppError::RateLimited) {
            response.headers_mut().insert(
                "Retry-After",
                axum::http::HeaderValue::from_static("10"),
            );
        }
        response
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::Database(db_err) => {
                if db_err.code().as_deref() == Some("23505") {
                    return AppError::Conflict("Resource already exists".to_string());
                }
            }
            sqlx::Error::RowNotFound => return AppError::NotFound,
            _ => {}
        }
        AppError::Internal(err.into())
    }
}
