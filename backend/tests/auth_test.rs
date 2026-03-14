mod common;

use axum::http::StatusCode;
use serde_json::json;
use serial_test::serial;

use common::*;

// ── Registration ──

#[tokio::test]
#[serial]
async fn register_creates_user_and_returns_tokens() {
    let (app, _pool) = test_app().await;

    let (status, body) = post_json(&app, "/api/auth/register", json!({
        "email": "alice@example.com",
        "password": "securepass123",
        "name": "Alice"
    })).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["tokens"]["access_token"].is_string());
    assert!(body["tokens"]["refresh_token"].is_string());
    assert_eq!(body["user"]["email"], "alice@example.com");
    assert_eq!(body["user"]["name"], "Alice");
    // password_hash must never be exposed in API responses
    assert!(body["user"]["password_hash"].is_null());
}

#[tokio::test]
#[serial]
async fn duplicate_email_returns_409_conflict() {
    let (app, _pool) = test_app().await;

    register_user(&app, "dup@example.com", "First").await;

    let (status, body) = post_json(&app, "/api/auth/register", json!({
        "email": "dup@example.com",
        "password": "anotherpass123",
        "name": "Second"
    })).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "CONFLICT");
}

#[tokio::test]
#[serial]
async fn short_password_rejected_at_registration() {
    let (app, _pool) = test_app().await;

    let (status, body) = post_json(&app, "/api/auth/register", json!({
        "email": "short@example.com",
        "password": "123",
        "name": "Short"
    })).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
#[serial]
async fn invalid_email_rejected_at_registration() {
    let (app, _pool) = test_app().await;

    let (status, body) = post_json(&app, "/api/auth/register", json!({
        "email": "not-an-email",
        "password": "securepass123",
        "name": "Invalid"
    })).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

// ── Login ──

#[tokio::test]
#[serial]
async fn login_succeeds_with_correct_credentials() {
    let (app, _pool) = test_app().await;

    register_user(&app, "login@example.com", "Login User").await;

    let (status, body) = post_json(&app, "/api/auth/login", json!({
        "email": "login@example.com",
        "password": "testpassword123"
    })).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["tokens"]["access_token"].is_string());
    assert_eq!(body["user"]["email"], "login@example.com");
}

#[tokio::test]
#[serial]
async fn login_fails_with_wrong_password() {
    let (app, _pool) = test_app().await;

    register_user(&app, "wrong@example.com", "Wrong Pass").await;

    let (status, body) = post_json(&app, "/api/auth/login", json!({
        "email": "wrong@example.com",
        "password": "wrongpassword"
    })).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
#[serial]
async fn login_fails_for_nonexistent_user() {
    let (app, _pool) = test_app().await;

    let (status, _) = post_json(&app, "/api/auth/login", json!({
        "email": "ghost@example.com",
        "password": "irrelevant123"
    })).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── JWT & Protected Endpoints ──

#[tokio::test]
#[serial]
async fn me_endpoint_requires_auth() {
    let (app, _pool) = test_app().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/auth/me")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn me_returns_current_user_info() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "me@example.com", "Me User").await;

    let (status, body) = get_auth(&app, "/api/auth/me", &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["email"], "me@example.com");
    assert_eq!(body["name"], "Me User");
    assert!(body["password_hash"].is_null());
}

#[tokio::test]
#[serial]
async fn update_name_via_patch_me() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "rename@example.com", "Old Name").await;

    let (status, body) = patch_auth(&app, "/api/auth/me", json!({"name": "New Name"}), &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "New Name");
}

// ── Refresh Token ──

#[tokio::test]
#[serial]
async fn refresh_token_issues_new_access_token() {
    let (app, _pool) = test_app().await;

    let (_, body) = post_json(&app, "/api/auth/register", json!({
        "email": "refresh@example.com",
        "password": "testpassword123",
        "name": "Refresh"
    })).await;

    let refresh_token = body["tokens"]["refresh_token"].as_str().unwrap();

    let (status, body) = post_json(&app, "/api/auth/refresh", json!({
        "refresh_token": refresh_token
    })).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
    // New refresh token must differ (rotation)
    assert_ne!(body["refresh_token"].as_str().unwrap(), refresh_token);
}

#[tokio::test]
#[serial]
async fn used_refresh_token_cannot_be_reused() {
    let (app, _pool) = test_app().await;

    let (_, body) = post_json(&app, "/api/auth/register", json!({
        "email": "reuse@example.com",
        "password": "testpassword123",
        "name": "Reuse"
    })).await;

    let refresh_token = body["tokens"]["refresh_token"].as_str().unwrap().to_string();

    // First refresh succeeds
    let (status, _) = post_json(&app, "/api/auth/refresh", json!({
        "refresh_token": &refresh_token
    })).await;
    assert_eq!(status, StatusCode::OK);

    // Second use of same token should fail (it was deleted)
    let (status, _) = post_json(&app, "/api/auth/refresh", json!({
        "refresh_token": &refresh_token
    })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Password Change ──

#[tokio::test]
#[serial]
async fn change_password_requires_current_password() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "chpass@example.com", "ChPass").await;

    // Wrong current password
    let (status, _) = post_json_auth(&app, "/api/auth/change-password", json!({
        "current_password": "wrongcurrent",
        "new_password": "newsecurepass123"
    }), &token).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn change_password_invalidates_all_sessions() {
    let (app, _pool) = test_app().await;

    let (_, reg_body) = post_json(&app, "/api/auth/register", json!({
        "email": "sessions@example.com",
        "password": "testpassword123",
        "name": "Sessions"
    })).await;

    let token = reg_body["tokens"]["access_token"].as_str().unwrap();
    let refresh = reg_body["tokens"]["refresh_token"].as_str().unwrap().to_string();

    // Change password
    let (status, _) = post_json_auth(&app, "/api/auth/change-password", json!({
        "current_password": "testpassword123",
        "new_password": "brandnewpass456"
    }), token).await;
    assert_eq!(status, StatusCode::OK);

    // Old refresh token should no longer work
    let (status, _) = post_json(&app, "/api/auth/refresh", json!({
        "refresh_token": &refresh
    })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Login with new password should work
    let (status, _) = post_json(&app, "/api/auth/login", json!({
        "email": "sessions@example.com",
        "password": "brandnewpass456"
    })).await;
    assert_eq!(status, StatusCode::OK);
}

// ── Logout ──

#[tokio::test]
#[serial]
async fn logout_revokes_specific_refresh_token() {
    let (app, _pool) = test_app().await;

    let (_, reg_body) = post_json(&app, "/api/auth/register", json!({
        "email": "logout@example.com",
        "password": "testpassword123",
        "name": "Logout"
    })).await;

    let refresh = reg_body["tokens"]["refresh_token"].as_str().unwrap().to_string();

    let (status, _) = post_json(&app, "/api/auth/logout", json!({
        "refresh_token": &refresh
    })).await;
    assert_eq!(status, StatusCode::OK);

    // Refresh with revoked token fails
    let (status, _) = post_json(&app, "/api/auth/refresh", json!({
        "refresh_token": &refresh
    })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
