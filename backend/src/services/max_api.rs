use crate::config::Config;
use crate::errors::AppError;

/// Returns the access token as the Authorization header value.
/// The Max API uses raw token format (no "Bearer" prefix).
/// This function exists as an abstraction point in case the auth scheme changes.
fn auth_header(access_token: &str) -> &str {
    access_token
}

/// Call getMyInfo to verify a bot token.
pub async fn get_my_info(
    client: &reqwest::Client,
    config: &Config,
    access_token: &str,
) -> Result<serde_json::Value, AppError> {
    let url = format!("{}/me", config.max_api_base_url);
    let resp = client
        .get(&url)
        .header("Authorization", auth_header(access_token))
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Max API request failed: {}", e)))?;

    handle_max_response(resp).await
}

/// Subscribe a webhook URL with the Max API.
pub async fn subscribe_webhook(
    client: &reqwest::Client,
    config: &Config,
    access_token: &str,
    webhook_url: &str,
) -> Result<serde_json::Value, AppError> {
    let url = format!("{}/subscriptions", config.max_api_base_url);
    let body = serde_json::json!({ "url": webhook_url });
    let resp = client
        .post(&url)
        .header("Authorization", auth_header(access_token))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Max API subscribe failed: {}", e)))?;

    handle_max_response(resp).await
}

/// Unsubscribe webhooks.
pub async fn unsubscribe_webhook(
    client: &reqwest::Client,
    config: &Config,
    access_token: &str,
) -> Result<serde_json::Value, AppError> {
    let url = format!("{}/subscriptions", config.max_api_base_url);
    let resp = client
        .delete(&url)
        .header("Authorization", auth_header(access_token))
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Max API unsubscribe failed: {}", e)))?;

    handle_max_response(resp).await
}

/// All update types supported by the Max API.
const ALL_UPDATE_TYPES: &str = "message_created,message_callback,message_edited,message_removed,bot_added,bot_removed,user_added,user_removed,bot_started,chat_title_changed,message_construction_request,message_construction_result,message_chat_created";

/// Get updates for long polling.
pub async fn get_updates(
    client: &reqwest::Client,
    config: &Config,
    access_token: &str,
    marker: Option<i64>,
    timeout: u64,
) -> Result<serde_json::Value, AppError> {
    let url = format!("{}/updates", config.max_api_base_url);
    let mut query: Vec<(&str, String)> = vec![
        ("timeout", timeout.to_string()),
        ("types", ALL_UPDATE_TYPES.to_string()),
    ];
    if let Some(m) = marker {
        query.push(("marker", m.to_string()));
    }

    let resp = client
        .get(&url)
        .header("Authorization", auth_header(access_token))
        .query(&query)
        .timeout(std::time::Duration::from_secs(timeout + 5))
        .send()
        .await
        .map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Max API polling failed: {}", e))
        })?;

    handle_max_response(resp).await
}

/// Fetch one page of chats from the Max API.
pub async fn get_chats(
    client: &reqwest::Client,
    config: &Config,
    access_token: &str,
    count: i64,
    marker: Option<i64>,
) -> Result<serde_json::Value, AppError> {
    let url = format!("{}/chats", config.max_api_base_url);
    let mut query: Vec<(&str, String)> = vec![
        ("count", count.to_string()),
    ];
    if let Some(m) = marker {
        query.push(("marker", m.to_string()));
    }

    let resp = client
        .get(&url)
        .header("Authorization", auth_header(access_token))
        .query(&query)
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Max API get_chats failed: {}", e)))?;

    handle_max_response(resp).await
}

/// Fetch messages from a chat via Max API.
/// Returns raw JSON with `messages` array.
/// `to` is an optional Unix timestamp — accepts both millis and seconds,
/// auto-converts millis to seconds for the Max API.
pub async fn get_messages(
    client: &reqwest::Client,
    config: &Config,
    access_token: &str,
    chat_id: i64,
    to: Option<i64>,
    count: i32,
) -> Result<serde_json::Value, AppError> {
    let url = format!("{}/messages", config.max_api_base_url);
    let mut query: Vec<(&str, String)> = vec![
        ("chat_id", chat_id.to_string()),
        ("count", count.clamp(1, 100).to_string()),
    ];
    if let Some(ts) = to {
        // Max API expects seconds; our timestamps are in milliseconds
        let ts_secs = if ts > 1_000_000_000_000 { ts / 1000 } else { ts };
        query.push(("to", ts_secs.to_string()));
    }

    let resp = client
        .get(&url)
        .header("Authorization", auth_header(access_token))
        .query(&query)
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Max API get_messages failed: {}", e)))?;

    handle_max_response(resp).await
}

/// Proxy an arbitrary call to the Max API, returning the raw status code and body.
/// Returns `Ok((status, body))` for ALL HTTP responses (2xx and non-2xx).
/// Returns `Err` only for network or parse errors.
pub async fn proxy_call_raw(
    client: &reqwest::Client,
    config: &Config,
    access_token: &str,
    method: &str,
    path: &str,
    body: Option<serde_json::Value>,
) -> Result<(u16, serde_json::Value), AppError> {
    validate_path(path)?;

    let url = format!("{}{}", config.max_api_base_url, path);

    // Defense-in-depth: validate final URL host matches configured base
    let parsed = url::Url::parse(&url)
        .map_err(|_| AppError::BadRequest("Invalid path".into()))?;
    if parsed.host_str() != Some(config.max_api_host.as_str()) {
        return Err(AppError::Forbidden);
    }

    let auth = auth_header(access_token);
    let req = match method.to_uppercase().as_str() {
        "GET" => client.get(&url).header("Authorization", auth),
        "POST" => {
            let r = client.post(&url).header("Authorization", auth);
            if let Some(b) = &body { r.json(b) } else { r }
        }
        "PUT" => {
            let r = client.put(&url).header("Authorization", auth);
            if let Some(b) = &body { r.json(b) } else { r }
        }
        "PATCH" => {
            let r = client.patch(&url).header("Authorization", auth);
            if let Some(b) = &body { r.json(b) } else { r }
        }
        "DELETE" => {
            let r = client.delete(&url).header("Authorization", auth);
            if let Some(b) = &body { r.json(b) } else { r }
        }
        _ => return Err(AppError::BadRequest("Unsupported HTTP method".into())),
    };

    let resp = req.send().await.map_err(|e| {
        if e.is_timeout() {
            AppError::MaxApiError {
                status: 504,
                body: serde_json::json!({"message": "Gateway Timeout"}),
            }
        } else {
            AppError::Internal(anyhow::anyhow!("Max API request failed: {}", e))
        }
    })?;

    let status = resp.status().as_u16();
    let resp_body: serde_json::Value = match resp.json().await {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(error = %e, status = status, "Max API returned non-JSON response");
            serde_json::json!({})
        }
    };

    Ok((status, resp_body))
}

/// Proxy an arbitrary call to the Max API (used by /raw endpoint).
/// Thin wrapper around `proxy_call_raw` that converts non-2xx to `AppError::MaxApiError`.
pub async fn proxy_call(
    client: &reqwest::Client,
    config: &Config,
    access_token: &str,
    method: &str,
    path: &str,
    body: Option<serde_json::Value>,
) -> Result<serde_json::Value, AppError> {
    let (status, resp_body) = proxy_call_raw(client, config, access_token, method, path, body).await?;
    if (200..300).contains(&status) {
        Ok(resp_body)
    } else {
        Err(AppError::MaxApiError { status, body: resp_body })
    }
}

fn validate_path(path: &str) -> Result<(), AppError> {
    if !path.starts_with('/') {
        return Err(AppError::BadRequest("Path must start with /".into()));
    }
    // Block traversal, protocol injection, double slashes, null bytes, CRLF
    if path.contains("://")
        || path.contains("..")
        || path.contains("//")
        || path.contains('\0')
        || path.contains('\r')
        || path.contains('\n')
    {
        return Err(AppError::BadRequest("Invalid path".into()));
    }
    // Only allow safe characters in paths
    if !path.chars().all(|c| c.is_ascii_alphanumeric() || "/-_.?=&".contains(c)) {
        return Err(AppError::BadRequest("Invalid path characters".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_path_valid() {
        assert!(validate_path("/messages").is_ok());
        assert!(validate_path("/chats/123/members").is_ok());
        assert!(validate_path("/messages?message_id=abc").is_ok());
    }

    #[test]
    fn validate_path_must_start_with_slash() {
        assert!(validate_path("messages").is_err());
    }

    #[test]
    fn validate_path_blocks_traversal() {
        assert!(validate_path("/../../etc/passwd").is_err());
    }

    #[test]
    fn validate_path_blocks_protocol_injection() {
        assert!(validate_path("/http://evil.com").is_err());
    }

    #[test]
    fn validate_path_blocks_null_bytes() {
        assert!(validate_path("/test\0").is_err());
    }

    #[test]
    fn validate_path_blocks_double_slash() {
        assert!(validate_path("//evil.com").is_err());
    }

    #[test]
    fn validate_path_blocks_unsafe_chars() {
        assert!(validate_path("/test%00").is_err());
        assert!(validate_path("/test<script>").is_err());
    }
}

async fn handle_max_response(resp: reqwest::Response) -> Result<serde_json::Value, AppError> {
    let status = resp.status().as_u16();
    let body: serde_json::Value = match resp.json().await {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(error = %e, status = status, "Max API returned non-JSON response");
            serde_json::json!({})
        }
    };

    if (200..300).contains(&status) {
        Ok(body)
    } else {
        Err(AppError::MaxApiError { status, body })
    }
}
