use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::errors::{AppError, FieldError};

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct PaginationQuery {
    /// Offset from the beginning of the list. Default: 0.
    #[schema(example = 0, minimum = 0)]
    pub offset: Option<i64>,
    /// Maximum number of items to return. Default: 20, max: 100.
    #[schema(example = 20, minimum = 1, maximum = 100)]
    pub limit: Option<i64>,
}

impl PaginationQuery {
    /// Resolve pagination parameters with defaults and upper bound.
    pub fn resolve(&self) -> (i64, i64) {
        let limit = self.limit.unwrap_or(20).clamp(1, 100);
        let offset = self.offset.unwrap_or(0).max(0);
        (limit, offset)
    }
}

#[derive(Serialize, ToSchema)]
pub struct OkResponse {
    #[schema(example = true)]
    pub ok: bool,
}

#[derive(Serialize, ToSchema)]
pub struct PaginationInfo {
    #[schema(example = 42)]
    pub total: i64,
    #[schema(example = 0)]
    pub offset: i64,
    #[schema(example = 20)]
    pub limit: i64,
}

#[derive(Serialize, ToSchema)]
pub struct CursorPaginationInfo {
    #[schema(example = "MjAyNi0wMy0xNFQxMjowMDowMFo6NTUwZTg0MDA=")]
    pub next_cursor: Option<String>,
    #[schema(example = true)]
    pub has_more: bool,
}

pub fn validate_request<T: Validate>(req: &T) -> Result<(), AppError> {
    if let Err(errors) = req.validate() {
        let fields: Vec<FieldError> = errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errs)| {
                errs.iter().map(move |e| FieldError {
                    field: field.to_string(),
                    message: e.message.as_ref().map(|m| m.to_string()).unwrap_or_else(|| e.code.to_string()),
                })
            })
            .collect();
        return Err(AppError::Validation(fields));
    }
    Ok(())
}

pub fn validate_slug(slug: &str) -> Result<(), AppError> {
    if slug.len() > 100 {
        return Err(AppError::Validation(vec![FieldError {
            field: "slug".into(),
            message: "Slug must be at most 100 characters".into(),
        }]));
    }
    if slug.len() < 2 {
        return Err(AppError::Validation(vec![FieldError {
            field: "slug".into(),
            message: "Slug must be at least 2 characters".into(),
        }]));
    }
    if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(AppError::Validation(vec![FieldError {
            field: "slug".into(),
            message: "Slug must contain only lowercase letters, digits, and hyphens".into(),
        }]));
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(AppError::Validation(vec![FieldError {
            field: "slug".into(),
            message: "Slug must not start or end with a hyphen".into(),
        }]));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_slug_valid() {
        assert!(validate_slug("my-project").is_ok());
        assert!(validate_slug("ab").is_ok());
        assert!(validate_slug("test123").is_ok());
    }

    #[test]
    fn validate_slug_too_short() {
        assert!(validate_slug("a").is_err());
        assert!(validate_slug("").is_err());
    }

    #[test]
    fn validate_slug_invalid_chars() {
        assert!(validate_slug("My-Project").is_err());
        assert!(validate_slug("my_project").is_err());
        assert!(validate_slug("my project").is_err());
    }

    #[test]
    fn validate_slug_leading_trailing_hyphen() {
        assert!(validate_slug("-test").is_err());
        assert!(validate_slug("test-").is_err());
    }
}
