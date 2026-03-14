use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use std::collections::HashMap;
use uuid::Uuid;

use crate::db;
use crate::errors::AppError;
use crate::models::api_key::ApiKeyRow;
use crate::services::api_keys;
use crate::app_state::AppState;

/// Minimum key length: "ak_" (3) + PREFIX_HEX_LEN (8) = 11
const MIN_API_KEY_LEN: usize = 11;
const API_KEY_PREFIX_LEN: usize = 11;

#[derive(Debug)]
pub struct ApiKeyAuth {
    pub bot_id: Uuid,
    pub api_key: ApiKeyRow,
}

impl FromRequestParts<AppState> for ApiKeyAuth {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            // Extract bot_id from path
            let params = axum::extract::Path::<HashMap<String, String>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AppError::BadRequest("Missing path parameters".into()))?;
            let bot_id: Uuid = params
                .get("bot_id")
                .ok_or_else(|| AppError::BadRequest("Missing bot_id".into()))?
                .parse()
                .map_err(|_| AppError::BadRequest("Invalid bot_id".into()))?;

            // Extract Bearer token from Authorization header
            let auth_header = parts
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .ok_or(AppError::Unauthorized)?;

            let key = auth_header
                .strip_prefix("Bearer ")
                .ok_or(AppError::Unauthorized)?;

            if !key.starts_with("ak_") || key.len() < MIN_API_KEY_LEN {
                return Err(AppError::Unauthorized);
            }

            // Extract prefix (ak_ + 8 hex chars)
            let prefix = &key[..API_KEY_PREFIX_LEN];

            // Look up candidates by prefix
            let candidates = db::api_keys::find_by_prefix(&state.db, bot_id, prefix).await
                .map_err(|e| AppError::Internal(e.into()))?;

            if candidates.is_empty() {
                return Err(AppError::Unauthorized);
            }

            // Verify HMAC against each candidate
            let secret = &state.config.bot_api_key_hmac_secret;
            for candidate in candidates {
                if api_keys::verify_api_key(secret, key, &candidate.key_hash) {
                    // Update last_used_at (fire-and-forget)
                    let pool = state.db.clone();
                    let key_id = candidate.id;
                    tokio::spawn(async move {
                        let _ = db::api_keys::update_last_used(&pool, key_id).await;
                    });

                    return Ok(ApiKeyAuth {
                        bot_id,
                        api_key: candidate,
                    });
                }
            }

            Err(AppError::Unauthorized)
        }
    }
}
