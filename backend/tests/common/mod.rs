use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

use tokio_util::sync::CancellationToken;

use max_dashboard_backend::app_state::AppState;
use max_dashboard_backend::config::Config;
use max_dashboard_backend::router::build_router;

/// Build the full application router backed by a real database.
pub async fn test_app() -> (Router, PgPool) {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("Config must be valid for tests");
    let pool = PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to test database");

    // Clean all data before test (idempotent)
    clean_db(&pool).await;

    let cancel = CancellationToken::new();
    let state = AppState::new(pool.clone(), config, cancel);
    let app = build_router(state);
    (app, pool)
}

async fn clean_db(pool: &PgPool) {
    // Delete in dependency order
    sqlx::query("DELETE FROM refresh_tokens").execute(pool).await.ok();
    sqlx::query("DELETE FROM events").execute(pool).await.ok();
    sqlx::query("DELETE FROM bot_chats").execute(pool).await.ok();
    sqlx::query("DELETE FROM invites").execute(pool).await.ok();
    sqlx::query("DELETE FROM bots").execute(pool).await.ok();
    sqlx::query("DELETE FROM project_members").execute(pool).await.ok();
    sqlx::query("DELETE FROM projects").execute(pool).await.ok();
    sqlx::query("DELETE FROM organization_members").execute(pool).await.ok();
    sqlx::query("DELETE FROM organizations").execute(pool).await.ok();
    sqlx::query("DELETE FROM users").execute(pool).await.ok();
}

// ── HTTP helpers ──

pub async fn post_json(app: &Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    send(app, req).await
}

pub async fn post_json_auth(app: &Router, uri: &str, body: Value, token: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    send(app, req).await
}

pub async fn get_auth(app: &Router, uri: &str, token: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    send(app, req).await
}

pub async fn patch_auth(app: &Router, uri: &str, body: Value, token: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("PATCH")
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    send(app, req).await
}

pub async fn delete_auth(app: &Router, uri: &str, token: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("DELETE")
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    send(app, req).await
}

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = if body_bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body_bytes).unwrap_or(Value::Null)
    };
    (status, body)
}

// ── Scenario helpers ──

/// Register a user and return (access_token, user_id)
pub async fn register_user(app: &Router, email: &str, name: &str) -> (String, String) {
    let (status, body) = post_json(app, "/api/auth/register", serde_json::json!({
        "email": email,
        "password": "testpassword123",
        "name": name,
    })).await;
    assert!(status == StatusCode::OK || status == StatusCode::CREATED, "register failed: {:?}", body);
    let token = body["tokens"]["access_token"].as_str().unwrap().to_string();
    let user_id = body["user"]["id"].as_str().unwrap().to_string();
    (token, user_id)
}

/// Create an organization and return its slug
pub async fn create_org(app: &Router, token: &str, name: &str, slug: &str) -> Value {
    let (status, body) = post_json_auth(app, "/api/organizations", serde_json::json!({
        "name": name,
        "slug": slug,
    }), token).await;
    assert!(status == StatusCode::OK || status == StatusCode::CREATED, "create org failed: {:?}", body);
    body
}

/// Create a project within an org, return full response
pub async fn create_project(app: &Router, token: &str, org_slug: &str, name: &str, slug: &str) -> Value {
    let uri = format!("/api/organizations/{}/projects", org_slug);
    let (status, body) = post_json_auth(app, &uri, serde_json::json!({
        "name": name,
        "slug": slug,
    }), token).await;
    assert!(status == StatusCode::OK || status == StatusCode::CREATED, "create project failed: {:?}", body);
    body
}
