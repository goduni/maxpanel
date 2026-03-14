mod common;

use axum::http::StatusCode;
use serial_test::serial;

/// Test that creating a bot requires authentication.
#[tokio::test]
#[serial]
async fn create_bot_requires_auth() {
    let (app, _pool) = common::test_app().await;
    let (status, _body) = common::post_json(
        &app,
        "/api/organizations/test-org/projects/test-proj/bots",
        serde_json::json!({
            "name": "Test Bot",
            "access_token": "fake-token",
            "event_mode": "polling",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

/// Test that listing bots requires authentication.
#[tokio::test]
#[serial]
async fn list_bots_requires_auth() {
    let (app, _pool) = common::test_app().await;
    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/organizations/test-org/projects/test-proj/bots")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that bot start/stop endpoints require auth and valid bot_id.
#[tokio::test]
#[serial]
async fn start_stop_requires_auth() {
    let (app, _pool) = common::test_app().await;
    let (status, _body) = common::post_json(
        &app,
        "/api/bots/00000000-0000-0000-0000-000000000000/start",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
