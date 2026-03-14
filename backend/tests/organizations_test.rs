mod common;

use axum::http::StatusCode;
use serde_json::json;
use serial_test::serial;

use common::*;

// ── Organization CRUD ──

#[tokio::test]
#[serial]
async fn creator_becomes_owner() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "owner@test.com", "Owner").await;

    create_org(&app, &token, "Owned Org", "owned-org").await;

    let (status, body) = get_auth(&app, "/api/organizations/owned-org/members", &token).await;
    assert_eq!(status, StatusCode::OK);

    let members = body["data"].as_array().unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0]["role"], "owner");
}

#[tokio::test]
#[serial]
async fn duplicate_slug_returns_conflict() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "slug@test.com", "Slug").await;

    create_org(&app, &token, "First", "dup-slug").await;

    let (status, body) = post_json_auth(&app, "/api/organizations", json!({
        "name": "Second",
        "slug": "dup-slug"
    }), &token).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "CONFLICT");
}

#[tokio::test]
#[serial]
async fn invalid_slug_rejected() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "badslug@test.com", "Bad").await;

    // Slug starting with hyphen
    let (status, _) = post_json_auth(&app, "/api/organizations", json!({
        "name": "Bad Slug",
        "slug": "-bad"
    }), &token).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Slug with uppercase
    let (status, _) = post_json_auth(&app, "/api/organizations", json!({
        "name": "Bad Slug",
        "slug": "BadSlug"
    }), &token).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn list_only_shows_user_orgs() {
    let (app, _pool) = test_app().await;
    let (token_a, _) = register_user(&app, "a@test.com", "A").await;
    let (token_b, _) = register_user(&app, "b@test.com", "B").await;

    create_org(&app, &token_a, "Org A", "org-a").await;
    create_org(&app, &token_b, "Org B", "org-b").await;

    // A only sees Org A
    let (_, body) = get_auth(&app, "/api/organizations", &token_a).await;
    let orgs = body["data"].as_array().unwrap();
    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0]["slug"], "org-a");

    // B only sees Org B
    let (_, body) = get_auth(&app, "/api/organizations", &token_b).await;
    let orgs = body["data"].as_array().unwrap();
    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0]["slug"], "org-b");
}

#[tokio::test]
#[serial]
async fn non_member_gets_404_not_403() {
    let (app, _pool) = test_app().await;
    let (token_a, _) = register_user(&app, "visible@test.com", "Visible").await;
    let (token_b, _) = register_user(&app, "outsider@test.com", "Outsider").await;

    create_org(&app, &token_a, "Secret Org", "secret-org").await;

    // Non-member should get 404, not 403 (resource existence hiding)
    let (status, _) = get_auth(&app, "/api/organizations/secret-org", &token_b).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Only owner can delete ──

#[tokio::test]
#[serial]
async fn only_owner_can_delete_organization() {
    let (app, pool) = test_app().await;
    let (owner_token, owner_id) = register_user(&app, "delowner@test.com", "Del Owner").await;
    let (admin_token, admin_id) = register_user(&app, "deladmin@test.com", "Del Admin").await;

    create_org(&app, &owner_token, "Deletable", "deletable").await;

    // Add admin as member
    sqlx::query("INSERT INTO organization_members (organization_id, user_id, role) SELECT o.id, $1, 'admin' FROM organizations o WHERE o.slug = 'deletable'")
        .bind(uuid::Uuid::parse_str(&admin_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    // Admin can't delete
    let (status, _) = delete_auth(&app, "/api/organizations/deletable", &admin_token).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Owner can delete
    let (status, _) = delete_auth(&app, "/api/organizations/deletable", &owner_token).await;
    assert_eq!(status, StatusCode::OK);
}

// ── Ownership Transfer ──

#[tokio::test]
#[serial]
async fn ownership_transfer_swaps_roles() {
    let (app, pool) = test_app().await;
    let (owner_token, _owner_id) = register_user(&app, "xferowner@test.com", "Owner").await;
    let (_new_token, new_id) = register_user(&app, "xfernew@test.com", "New Owner").await;

    create_org(&app, &owner_token, "Transfer Org", "xfer-org").await;

    // Add new_owner as member first
    sqlx::query("INSERT INTO organization_members (organization_id, user_id, role) SELECT o.id, $1, 'member' FROM organizations o WHERE o.slug = 'xfer-org'")
        .bind(uuid::Uuid::parse_str(&new_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    // Transfer
    let (status, _) = post_json_auth(&app, "/api/organizations/xfer-org/transfer-ownership",
        json!({"new_owner_id": new_id}), &owner_token).await;
    assert_eq!(status, StatusCode::OK);

    // Original owner should now be admin, not owner
    let (_, members_body) = get_auth(&app, "/api/organizations/xfer-org/members", &owner_token).await;
    let members = members_body["data"].as_array().unwrap();
    for m in members {
        let uid = m["user_id"].as_str().unwrap();
        if uid == new_id {
            assert_eq!(m["role"], "owner");
        } else {
            assert_eq!(m["role"], "admin");
        }
    }
}

// ── Role Assignment Constraints ──

#[tokio::test]
#[serial]
async fn cannot_assign_owner_role_via_update() {
    let (app, pool) = test_app().await;
    let (owner_token, _) = register_user(&app, "roleowner@test.com", "Role Owner").await;
    let (_, member_id) = register_user(&app, "rolemember@test.com", "Member").await;

    create_org(&app, &owner_token, "Role Org", "role-org").await;

    // Add member
    sqlx::query("INSERT INTO organization_members (organization_id, user_id, role) SELECT o.id, $1, 'member' FROM organizations o WHERE o.slug = 'role-org'")
        .bind(uuid::Uuid::parse_str(&member_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    // Try to make them owner via PATCH — should be rejected
    let uri = format!("/api/organizations/role-org/members/{}", member_id);
    let (status, body) = patch_auth(&app, &uri, json!({"role": "owner"}), &owner_token).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"]["message"].as_str().unwrap().contains("transfer-ownership"));
}

#[tokio::test]
#[serial]
async fn cannot_remove_owner() {
    let (app, pool) = test_app().await;
    let (admin_token, admin_id) = register_user(&app, "rmadmin@test.com", "Admin").await;
    let (_, owner_id) = register_user(&app, "rmowner@test.com", "Owner").await;

    // Owner creates org
    let (owner_token, _) = register_user(&app, "rmowner2@test.com", "Owner2").await;
    create_org(&app, &owner_token, "Rm Org", "rm-org").await;

    // Add admin
    sqlx::query("INSERT INTO organization_members (organization_id, user_id, role) SELECT o.id, $1, 'admin' FROM organizations o WHERE o.slug = 'rm-org'")
        .bind(uuid::Uuid::parse_str(&admin_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    // Get owner's user_id
    let (_, members) = get_auth(&app, "/api/organizations/rm-org/members", &admin_token).await;
    let owner_uid = members["data"].as_array().unwrap()
        .iter()
        .find(|m| m["role"] == "owner")
        .unwrap()["user_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Admin cannot remove owner
    let uri = format!("/api/organizations/rm-org/members/{}", owner_uid);
    let (status, _) = delete_auth(&app, &uri, &admin_token).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ── Pagination ──

#[tokio::test]
#[serial]
async fn organization_list_respects_pagination() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "pag@test.com", "Pag").await;

    for i in 0..5 {
        create_org(&app, &token, &format!("Org {}", i), &format!("org-{}", i)).await;
    }

    let (_, body) = get_auth(&app, "/api/organizations?limit=2&offset=0", &token).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total"], 5);
    assert_eq!(body["pagination"]["limit"], 2);
    assert_eq!(body["pagination"]["offset"], 0);

    let (_, body) = get_auth(&app, "/api/organizations?limit=2&offset=4", &token).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}
