use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{Json, Response};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;
use url::Url;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
#[allow(unused_imports)]
use crate::errors::ErrorResponse; // used in OpenAPI schema annotations
use crate::extractors::BotAuthContext;

/// Query parameters for the media proxy endpoint.
#[derive(Deserialize)]
pub struct MediaProxyQuery {
    /// The upstream media URL to proxy (must be URL-encoded).
    pub url: String,
}

/// Allowed domain suffixes for the media proxy.
/// Requests to any other domain are rejected.
const ALLOWED_DOMAIN_SUFFIXES: &[&str] = &[".okcdn.ru", ".mycdn.me", ".oneme.ru"];

/// Returns a dedicated reqwest::Client for media proxying with appropriate timeouts.
///
/// Uses OnceLock so the client is built once and reused across all requests.
/// This client has longer timeouts than the general-purpose one (which is 60s)
/// because media files can be large and take time to transfer.
fn media_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(120))
            .redirect(reqwest::redirect::Policy::limited(5))
            .pool_max_idle_per_host(10)
            // Do not send a default User-Agent that identifies as a browser
            .user_agent("MaxPanel-MediaProxy/1.0")
            .build()
            .expect("Failed to build media proxy HTTP client")
    })
}

/// Validates that a URL points to an allowed CDN domain.
///
/// Returns the parsed URL on success, or an AppError::BadRequest on failure.
fn validate_media_url(raw_url: &str) -> Result<Url, AppError> {
    let parsed = Url::parse(raw_url).map_err(|_| {
        AppError::BadRequest("Invalid media URL".into())
    })?;

    // Allow HTTP and HTTPS (Max CDN serves video over HTTP)
    if parsed.scheme() != "https" && parsed.scheme() != "http" {
        return Err(AppError::BadRequest(
            "Only HTTP(S) media URLs are allowed".into(),
        ));
    }

    let host = parsed.host_str().ok_or_else(|| {
        AppError::BadRequest("Media URL must have a host".into())
    })?;

    let is_allowed = ALLOWED_DOMAIN_SUFFIXES
        .iter()
        .any(|suffix| host == &suffix[1..] || host.ends_with(suffix));

    if !is_allowed {
        return Err(AppError::BadRequest(format!(
            "Domain '{}' is not in the allowed media domain list",
            host
        )));
    }

    Ok(parsed)
}

/// Response from the media-info endpoint.
#[derive(Serialize, ToSchema)]
pub struct MediaInfoResponse {
    /// URL to stream the video through our proxy
    pub proxy_url: String,
    /// Thumbnail/preview image URL (can be loaded directly, it's HTTPS)
    pub thumbnail_url: Option<String>,
    /// Duration in seconds
    pub duration: Option<u64>,
    /// Video width in pixels
    pub width: Option<u64>,
    /// Video height in pixels
    pub height: Option<u64>,
}

/// Get media info for a video token. Fetches metadata from Max API
/// and returns a proxy URL + thumbnail + dimensions.
///
/// `GET /api/bots/{bot_id}/media-info/{token}`
#[utoipa::path(
    get,
    path = "/api/bots/{bot_id}/media-info/{token}",
    params(
        ("bot_id" = Uuid, Path, description = "Bot ID"),
        ("token" = String, Path, description = "Video token from Max API"),
    ),
    responses(
        (status = 200, description = "Video metadata", body = MediaInfoResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Media"
)]
pub async fn media_info(
    State(state): State<AppState>,
    ctx: BotAuthContext,
    Path((_bot_id, token)): Path<(Uuid, String)>,
) -> Result<Json<MediaInfoResponse>, AppError> {
    // Decrypt the bot's access token to call Max API
    let access_token = crate::services::bots::decrypt_bot_token_from_auth(&state.config, &ctx.auth_row)?;

    validate_media_token(&token)?;

    // Call Max API: GET /videos/{token}
    let max_response = state
        .http_client
        .get(format!("{}/videos/{}", state.config.max_api_base_url, token))
        .header("Authorization", &access_token)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to fetch video info from Max API");
            AppError::Internal(anyhow::anyhow!("Failed to fetch video info"))
        })?;

    if !max_response.status().is_success() {
        return Err(AppError::MaxApiError {
            status: max_response.status().as_u16(),
            body: serde_json::json!({"message": "Max API returned error for video info"}),
        });
    }

    let video_data: serde_json::Value = max_response.json().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to parse video info response");
        AppError::Internal(anyhow::anyhow!("Failed to parse video info"))
    })?;

    // Verify a video URL exists in the response
    if extract_best_video_url(&video_data).is_none() {
        return Err(AppError::BadRequest("No video URL in Max API response".into()));
    }

    // Build proxy URL using the same token
    let proxy_url = format!(
        "/api/bots/{}/media-stream/{}",
        ctx.auth_row.bot_id,
        token,
    );

    let thumbnail_url = video_data
        .get("thumbnail")
        .and_then(|t| t.get("url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let duration = video_data
        .get("duration")
        .and_then(|v| v.as_u64())
        .map(|ms| ms / 1000);

    let width = video_data.get("width").and_then(|v| v.as_u64());
    let height = video_data.get("height").and_then(|v| v.as_u64());

    Ok(Json(MediaInfoResponse {
        proxy_url,
        thumbnail_url,
        duration,
        width,
        height,
    }))
}

/// Extract the best video URL from Max API /videos/{token} response.
fn extract_best_video_url(video_data: &serde_json::Value) -> Option<String> {
    video_data
        .get("urls")
        .and_then(|urls| {
            for key in &["mp4_1080", "mp4_720", "mp4_480", "mp4_360", "mp4_240"] {
                if let Some(url) = urls.get(key).and_then(|v| v.as_str()) {
                    return Some(url.to_string());
                }
            }
            urls.as_object().and_then(|obj| {
                obj.values()
                    .find_map(|v| v.as_str().map(|s| s.to_string()))
            })
        })
        .or_else(|| {
            video_data.get("url").and_then(|v| v.as_str().map(|s| s.to_string()))
        })
}

/// Headers that are safe to forward from the upstream CDN response to the client.
/// We explicitly enumerate them rather than forwarding everything to avoid
/// leaking internal CDN headers (Set-Cookie, server info, etc).
const FORWARDABLE_RESPONSE_HEADERS: &[header::HeaderName] = &[
    header::CONTENT_TYPE,
    header::CONTENT_LENGTH,
    header::ACCEPT_RANGES,
    header::CONTENT_RANGE,
    header::CACHE_CONTROL,
    header::ETAG,
    header::LAST_MODIFIED,
];

/// Stream media by video token. Fetches the video URL from Max API,
/// then streams the CDN response without buffering.
/// Supports Range requests for video seeking.
///
/// `GET /api/bots/{bot_id}/media-stream/{token}`
#[utoipa::path(
    get,
    path = "/api/bots/{bot_id}/media-stream/{token}",
    params(
        ("bot_id" = Uuid, Path, description = "Bot ID"),
        ("token" = String, Path, description = "Video token from Max API"),
    ),
    responses(
        (status = 200, description = "Video stream (supports Range requests)"),
        (status = 206, description = "Partial content (Range response)"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer" = [])),
    tag = "Media"
)]
pub async fn media_stream(
    State(state): State<AppState>,
    ctx: BotAuthContext,
    headers: HeaderMap,
    Path((_bot_id, token)): Path<(Uuid, String)>,
) -> Result<Response, AppError> {
    validate_media_token(&token)?;

    // Check cache before calling Max API
    let cache_key = (ctx.auth_row.bot_id, token.clone());
    let cached_url = state.video_url_cache.get(&cache_key)
        .filter(|entry| entry.value().1.elapsed() < Duration::from_secs(300))
        .map(|entry| entry.value().0.clone());

    let video_url = if let Some(url) = cached_url {
        url
    } else {
        // Decrypt bot token and fetch video URL from Max API
        let access_token = crate::services::bots::decrypt_bot_token_from_auth(&state.config, &ctx.auth_row)?;

        let max_response = state
            .http_client
            .get(format!("{}/videos/{}", state.config.max_api_base_url, token))
            .header("Authorization", &access_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to fetch video info: {}", e)))?;

        if !max_response.status().is_success() {
            return Err(AppError::MaxApiError {
                status: max_response.status().as_u16(),
                body: serde_json::json!({"message": "Failed to get video URL from Max API"}),
            });
        }

        let video_data: serde_json::Value = max_response.json().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to parse video info: {}", e)))?;

        let url = extract_best_video_url(&video_data)
            .ok_or_else(|| AppError::BadRequest("No video URL in Max API response".into()))?;
        state.video_url_cache.insert(cache_key, (url.clone(), std::time::Instant::now()));
        url
    };

    let upstream_url = validate_media_url(&video_url)?;

    let client = media_client();
    let mut request = client.get(upstream_url.as_str());

    // Forward Range header for video seeking support
    if let Some(range) = headers.get(header::RANGE) {
        request = request.header(header::RANGE, range);
    }

    // Forward If-None-Match / If-Modified-Since for conditional requests
    if let Some(etag) = headers.get(header::IF_NONE_MATCH) {
        request = request.header(header::IF_NONE_MATCH, etag);
    }
    if let Some(ims) = headers.get(header::IF_MODIFIED_SINCE) {
        request = request.header(header::IF_MODIFIED_SINCE, ims);
    }

    let upstream_resp = request.send().await.map_err(|e| {
        if e.is_timeout() {
            tracing::warn!(url = %upstream_url, "Media proxy request timed out");
            AppError::MaxApiError {
                status: 504,
                body: serde_json::json!({"message": "Media proxy timeout"}),
            }
        } else if e.is_connect() {
            tracing::warn!(url = %upstream_url, error = %e, "Media proxy connection failed");
            AppError::MaxApiError {
                status: 502,
                body: serde_json::json!({"message": "Failed to connect to media server"}),
            }
        } else {
            tracing::error!(url = %upstream_url, error = %e, "Media proxy request failed");
            AppError::Internal(anyhow::anyhow!("Media proxy request failed: {}", e))
        }
    })?;

    let upstream_status = upstream_resp.status();

    // For non-success statuses, return an appropriate error
    if upstream_status.is_client_error() || upstream_status.is_server_error() {
        tracing::warn!(
            url = %upstream_url,
            status = %upstream_status,
            "Media proxy: upstream returned error"
        );
        return Err(AppError::MaxApiError {
            status: upstream_status.as_u16(),
            body: serde_json::json!({
                "message": format!("Upstream media server returned {}", upstream_status)
            }),
        });
    }

    // Build response with forwarded headers
    let mut response_builder = Response::builder().status(StatusCode::from_u16(
        upstream_status.as_u16(),
    ).unwrap_or(StatusCode::OK));

    // Forward allowed headers from upstream
    for header_name in FORWARDABLE_RESPONSE_HEADERS {
        if let Some(value) = upstream_resp.headers().get(header_name) {
            response_builder = response_builder.header(header_name, value);
        }
    }

    // Ensure we don't accidentally set Content-Type if upstream didn't provide one
    // Default to application/octet-stream for safety
    if !upstream_resp.headers().contains_key(header::CONTENT_TYPE) {
        response_builder = response_builder.header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );
    }

    // Set cache headers to allow browsers to cache media locally
    if !upstream_resp.headers().contains_key(header::CACHE_CONTROL) {
        response_builder = response_builder.header(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=86400, immutable"),
        );
    }

    // Stream the body without buffering
    let body_stream = upstream_resp.bytes_stream();
    let body = axum::body::Body::from_stream(body_stream);

    response_builder
        .body(body)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to build response: {}", e)))
}

/// Validate that a media token contains only safe characters (no path traversal or query injection).
fn validate_media_token(token: &str) -> Result<(), AppError> {
    if token.is_empty() || token.len() > 256 {
        return Err(AppError::BadRequest("Invalid media token".into()));
    }
    if !token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err(AppError::BadRequest("Invalid media token characters".into()));
    }
    if token.contains("..") {
        return Err(AppError::BadRequest("Invalid media token".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_url_allows_okcdn() {
        assert!(validate_media_url("https://vd123.okcdn.ru/video.mp4").is_ok());
        assert!(validate_media_url("https://st.okcdn.ru/image.jpg").is_ok());
    }

    #[test]
    fn validate_url_allows_mycdn() {
        assert!(validate_media_url("https://cdn1.mycdn.me/media/file.mp4").is_ok());
    }

    #[test]
    fn validate_url_allows_oneme() {
        assert!(validate_media_url("https://media.oneme.ru/content/video.mp4").is_ok());
    }

    #[test]
    fn validate_url_allows_bare_domain() {
        // Domain without subdomain should also work (e.g., okcdn.ru itself)
        assert!(validate_media_url("https://okcdn.ru/file.mp4").is_ok());
        assert!(validate_media_url("https://mycdn.me/file.mp4").is_ok());
        assert!(validate_media_url("https://oneme.ru/file.mp4").is_ok());
    }

    #[test]
    fn validate_url_allows_http_for_cdn() {
        assert!(validate_media_url("http://vd123.okcdn.ru/video.mp4").is_ok());
        assert!(validate_media_url("http://cdn1.mycdn.me/file.mp4").is_ok());
    }

    #[test]
    fn validate_url_rejects_unknown_domain() {
        let err = validate_media_url("https://evil.com/video.mp4");
        assert!(err.is_err());
    }

    #[test]
    fn validate_url_rejects_suffix_trick() {
        // "notokcdn.ru" should not match ".okcdn.ru"
        let err = validate_media_url("https://notokcdn.ru/video.mp4");
        assert!(err.is_err());
    }

    #[test]
    fn validate_url_rejects_no_scheme() {
        let err = validate_media_url("vd123.okcdn.ru/video.mp4");
        assert!(err.is_err());
    }

    #[test]
    fn validate_url_rejects_empty() {
        let err = validate_media_url("");
        assert!(err.is_err());
    }

    #[test]
    fn validate_url_rejects_data_uri() {
        let err = validate_media_url("data:text/html,<script>alert(1)</script>");
        assert!(err.is_err());
    }

    #[test]
    fn validate_url_rejects_javascript() {
        let err = validate_media_url("javascript:alert(1)");
        assert!(err.is_err());
    }
}
