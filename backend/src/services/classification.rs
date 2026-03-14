pub fn classify_outbound(method: &str, path: &str) -> &'static str {
    let method_upper = method.to_uppercase();
    let path_base = path.split('?').next().unwrap_or(path);

    match (method_upper.as_str(), path_base) {
        ("POST", "/messages") => "message_sent",
        // starts_with intentional: PUT/DELETE /messages/{mid} include message ID in path
        ("PUT", p) if p.starts_with("/messages") => "message_edited",
        ("DELETE", p) if p.starts_with("/messages") => "message_deleted",
        ("POST", "/answers") => "callback_answered",
        ("POST", "/chats") => "chat_created",
        ("POST", "/uploads") => "file_uploaded",
        ("PATCH", p) if is_chats_id(p) && !p.contains("/members") => "chat_updated",
        ("POST", p) if p.ends_with("/members") && p.starts_with("/chats/") => "member_added",
        ("DELETE", p) if p.ends_with("/members") && p.starts_with("/chats/") => "member_removed",
        _ => "api_call",
    }
}

fn is_chats_id(path: &str) -> bool {
    path.starts_with("/chats/") && path.split('/').count() == 3
}

pub fn extract_chat_id_outbound(
    path: &str,
    request_body: Option<&serde_json::Value>,
    response_body: Option<&serde_json::Value>,
) -> Option<i64> {
    // 1. request_body.chat_id
    if let Some(body) = request_body {
        if let Some(cid) = body.get("chat_id").and_then(|v| v.as_i64()) {
            return Some(cid);
        }
    }

    // 2. Path: /chats/{chat_id}/...
    let path_base = path.split('?').next().unwrap_or(path);
    let segments: Vec<&str> = path_base.split('/').collect();
    if segments.len() >= 3 && segments[1] == "chats" {
        if let Ok(cid) = segments[2].parse::<i64>() {
            return Some(cid);
        }
    }

    // 3. response_body.message.recipient.chat_id
    if let Some(resp) = response_body {
        if let Some(cid) = resp
            .get("message")
            .and_then(|m| m.get("recipient"))
            .and_then(|r| r.get("chat_id"))
            .and_then(|v| v.as_i64())
        {
            return Some(cid);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_post_messages() {
        assert_eq!(classify_outbound("POST", "/messages"), "message_sent");
    }

    #[test]
    fn test_classify_put_messages() {
        assert_eq!(classify_outbound("PUT", "/messages"), "message_edited");
    }

    #[test]
    fn test_classify_delete_messages() {
        assert_eq!(classify_outbound("DELETE", "/messages"), "message_deleted");
    }

    #[test]
    fn test_classify_post_answers() {
        assert_eq!(classify_outbound("POST", "/answers"), "callback_answered");
    }

    #[test]
    fn test_classify_post_chats() {
        assert_eq!(classify_outbound("POST", "/chats"), "chat_created");
    }

    #[test]
    fn test_classify_patch_chats_id() {
        assert_eq!(classify_outbound("PATCH", "/chats/12345"), "chat_updated");
    }

    #[test]
    fn test_classify_post_members() {
        assert_eq!(classify_outbound("POST", "/chats/123/members"), "member_added");
    }

    #[test]
    fn test_classify_delete_members() {
        assert_eq!(classify_outbound("DELETE", "/chats/123/members"), "member_removed");
    }

    #[test]
    fn test_classify_post_uploads() {
        assert_eq!(classify_outbound("POST", "/uploads"), "file_uploaded");
    }

    #[test]
    fn test_classify_unknown_fallback() {
        assert_eq!(classify_outbound("GET", "/me"), "api_call");
        assert_eq!(classify_outbound("POST", "/subscriptions"), "api_call");
    }

    #[test]
    fn test_classify_case_insensitive_method() {
        assert_eq!(classify_outbound("post", "/messages"), "message_sent");
        assert_eq!(classify_outbound("Post", "/messages"), "message_sent");
    }

    #[test]
    fn test_classify_path_with_query_params() {
        assert_eq!(classify_outbound("POST", "/messages?param=value"), "message_sent");
    }

    #[test]
    fn test_extract_chat_id_from_request_body() {
        let body = serde_json::json!({"chat_id": 123, "text": "hello"});
        assert_eq!(extract_chat_id_outbound("/messages", Some(&body), None), Some(123));
    }

    #[test]
    fn test_extract_chat_id_from_path() {
        assert_eq!(extract_chat_id_outbound("/chats/456/members", None, None), Some(456));
    }

    #[test]
    fn test_extract_chat_id_from_response() {
        let resp = serde_json::json!({"message": {"recipient": {"chat_id": 789}}});
        assert_eq!(extract_chat_id_outbound("/messages", None, Some(&resp)), Some(789));
    }

    #[test]
    fn test_extract_chat_id_none() {
        assert_eq!(extract_chat_id_outbound("/uploads", None, None), None);
    }
}
