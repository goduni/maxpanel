mod common;

use axum::http::StatusCode;
use serial_test::serial;

/// Test that listing events requires authentication.
#[tokio::test]
#[serial]
async fn list_events_requires_auth() {
    let (app, _pool) = common::test_app().await;
    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/bots/00000000-0000-0000-0000-000000000000/events")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that getting a single event requires authentication.
#[tokio::test]
#[serial]
async fn get_event_requires_auth() {
    let (app, _pool) = common::test_app().await;
    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/bots/00000000-0000-0000-0000-000000000000/events/00000000-0000-0000-0000-000000000001")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test that webhook endpoint returns 404 for unknown secret.
#[tokio::test]
#[serial]
async fn webhook_unknown_secret_returns_404() {
    let (app, _pool) = common::test_app().await;
    let (status, _body) = common::post_json(
        &app,
        "/webhooks/00000000-0000-0000-0000-000000000099",
        serde_json::json!({"update_type": "message_created"}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
