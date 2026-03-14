mod common;

use axum::http::StatusCode;
use serde_json::json;
use serial_test::serial;

use common::*;

// ── Invite Creation ──

#[tokio::test]
#[serial]
async fn admin_can_create_invite() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "invadmin@test.com", "Admin").await;

    create_org(&app, &token, "Inv Org", "inv-org").await;

    let (status, body) = post_json_auth(
        &app,
        "/api/organizations/inv-org/invites",
        json!({
            "email": "newguy@test.com",
            "role": "member"
        }),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(body["token"].is_string());
    assert_eq!(body["invite"]["email"], "newguy@test.com");
    assert_eq!(body["invite"]["role"], "member");
    assert!(body["invite"]["id"].is_string());
}

#[tokio::test]
#[serial]
async fn cannot_invite_as_owner() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "ownerinv@test.com", "Owner").await;

    create_org(&app, &token, "Owner Inv Org", "owner-inv-org").await;

    let (status, body) = post_json_auth(
        &app,
        "/api/organizations/owner-inv-org/invites",
        json!({
            "email": "wannabe@test.com",
            "role": "owner"
        }),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("transfer-ownership"));
}

#[tokio::test]
#[serial]
async fn regular_member_cannot_create_invite() {
    let (app, pool) = test_app().await;
    let (owner_token, _) = register_user(&app, "invowner@test.com", "Owner").await;
    let (member_token, member_id) = register_user(&app, "invmember@test.com", "Member").await;

    create_org(&app, &owner_token, "Priv Org", "priv-org").await;

    // Add as regular member
    sqlx::query("INSERT INTO organization_members (organization_id, user_id, role) SELECT o.id, $1, 'member' FROM organizations o WHERE o.slug = 'priv-org'")
        .bind(uuid::Uuid::parse_str(&member_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    let (status, _) = post_json_auth(
        &app,
        "/api/organizations/priv-org/invites",
        json!({
            "email": "someone@test.com",
            "role": "member"
        }),
        &member_token,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Invite Accept ──

#[tokio::test]
#[serial]
async fn accept_invite_adds_user_to_org() {
    let (app, _pool) = test_app().await;
    let (owner_token, _) = register_user(&app, "accowner@test.com", "Owner").await;
    let (invitee_token, _) = register_user(&app, "invitee@test.com", "Invitee").await;

    create_org(&app, &owner_token, "Acc Org", "acc-org").await;

    // Create invite for the invitee's email
    let (_, invite_body) = post_json_auth(
        &app,
        "/api/organizations/acc-org/invites",
        json!({
            "email": "invitee@test.com",
            "role": "member"
        }),
        &owner_token,
    )
    .await;
    let invite_token = invite_body["token"].as_str().unwrap();

    // Accept the invite
    let uri = format!("/api/invites/{}/accept", invite_token);
    let (status, _) = post_json_auth(&app, &uri, json!({}), &invitee_token).await;
    assert_eq!(status, StatusCode::OK);

    // Invitee should now see the org
    let (_, body) = get_auth(&app, "/api/organizations", &invitee_token).await;
    let orgs = body["data"].as_array().unwrap();
    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0]["slug"], "acc-org");
}

#[tokio::test]
#[serial]
async fn accept_invite_fails_if_email_mismatch() {
    let (app, _pool) = test_app().await;
    let (owner_token, _) = register_user(&app, "mmowner@test.com", "Owner").await;
    let (wrong_token, _) = register_user(&app, "wrong@test.com", "Wrong").await;

    create_org(&app, &owner_token, "MM Org", "mm-org").await;

    // Create invite for different email
    let (_, invite_body) = post_json_auth(
        &app,
        "/api/organizations/mm-org/invites",
        json!({
            "email": "correct@test.com",
            "role": "member"
        }),
        &owner_token,
    )
    .await;
    let invite_token = invite_body["token"].as_str().unwrap();

    // Wrong user tries to accept
    let uri = format!("/api/invites/{}/accept", invite_token);
    let (status, _) = post_json_auth(&app, &uri, json!({}), &wrong_token).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── List & Revoke ──

#[tokio::test]
#[serial]
async fn list_pending_shows_only_pending_invites() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "listpend@test.com", "Owner").await;

    create_org(&app, &token, "List Org", "list-org").await;

    // Create two invites
    post_json_auth(
        &app,
        "/api/organizations/list-org/invites",
        json!({"email": "a@test.com", "role": "member"}),
        &token,
    )
    .await;
    post_json_auth(
        &app,
        "/api/organizations/list-org/invites",
        json!({"email": "b@test.com", "role": "admin"}),
        &token,
    )
    .await;

    let (status, body) = get_auth(&app, "/api/organizations/list-org/invites", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
#[serial]
async fn user_can_see_their_pending_invites() {
    let (app, _pool) = test_app().await;
    let (owner_token, _) = register_user(&app, "myinvowner@test.com", "Owner").await;
    let (user_token, _) = register_user(&app, "myinvuser@test.com", "User").await;

    create_org(&app, &owner_token, "MI Org", "mi-org").await;

    // Invite the user
    post_json_auth(
        &app,
        "/api/organizations/mi-org/invites",
        json!({"email": "myinvuser@test.com", "role": "member"}),
        &owner_token,
    )
    .await;

    // User sees their invite via /me/invites
    let (status, body) = get_auth(&app, "/api/auth/me/invites", &user_token).await;
    assert_eq!(status, StatusCode::OK);
    let invites = body["data"].as_array().unwrap();
    assert_eq!(invites.len(), 1);
}

#[tokio::test]
#[serial]
async fn revoke_invite_removes_it() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "revoke@test.com", "Owner").await;

    create_org(&app, &token, "Rev Org", "rev-org").await;

    let (_, invite_body) = post_json_auth(
        &app,
        "/api/organizations/rev-org/invites",
        json!({"email": "revokee@test.com", "role": "member"}),
        &token,
    )
    .await;
    let invite_id = invite_body["invite"]["id"].as_str().unwrap();

    // Revoke
    let uri = format!("/api/organizations/rev-org/invites/{}", invite_id);
    let (status, _) = delete_auth(&app, &uri, &token).await;
    assert_eq!(status, StatusCode::OK);

    // Pending list should be empty
    let (_, body) = get_auth(&app, "/api/organizations/rev-org/invites", &token).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}
