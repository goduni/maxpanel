mod common;

use axum::http::StatusCode;
use serde_json::json;
use serial_test::serial;

use common::*;

// ── Project CRUD ──

#[tokio::test]
#[serial]
async fn creator_becomes_project_admin() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "projowner@test.com", "Proj Owner").await;

    create_org(&app, &token, "PO", "po-org").await;
    create_project(&app, &token, "po-org", "My Project", "my-proj").await;

    let (status, body) = get_auth(
        &app,
        "/api/organizations/po-org/projects/my-proj/members",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let members = body["data"].as_array().unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0]["role"], "admin");
}

#[tokio::test]
#[serial]
async fn duplicate_project_slug_within_org_returns_conflict() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "dupproj@test.com", "Dup").await;

    create_org(&app, &token, "Dup Org", "dup-org").await;
    create_project(&app, &token, "dup-org", "First", "same-slug").await;

    let (status, _) = post_json_auth(
        &app,
        "/api/organizations/dup-org/projects",
        json!({
            "name": "Second",
            "slug": "same-slug"
        }),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
#[serial]
async fn project_list_scoped_to_org() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "scope@test.com", "Scope").await;

    create_org(&app, &token, "Org A", "org-a").await;
    create_org(&app, &token, "Org B", "org-b").await;

    create_project(&app, &token, "org-a", "Proj A1", "proj-a1").await;
    create_project(&app, &token, "org-a", "Proj A2", "proj-a2").await;
    create_project(&app, &token, "org-b", "Proj B1", "proj-b1").await;

    // Org A should have 2 projects
    let (_, body) = get_auth(&app, "/api/organizations/org-a/projects", &token).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);

    // Org B should have 1 project
    let (_, body) = get_auth(&app, "/api/organizations/org-b/projects", &token).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
#[serial]
async fn only_org_members_can_access_projects() {
    let (app, _pool) = test_app().await;
    let (token_a, _) = register_user(&app, "proja@test.com", "A").await;
    let (token_b, _) = register_user(&app, "projb@test.com", "B").await;

    create_org(&app, &token_a, "Private Org", "private-org").await;
    create_project(&app, &token_a, "private-org", "Secret", "secret").await;

    // Non-member can't list projects
    let (status, _) = get_auth(&app, "/api/organizations/private-org/projects", &token_b).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Non-member can't get specific project
    let (status, _) = get_auth(
        &app,
        "/api/organizations/private-org/projects/secret",
        &token_b,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Project Update/Delete requires admin ──

#[tokio::test]
#[serial]
async fn project_update_requires_admin() {
    let (app, pool) = test_app().await;
    let (admin_token, _) = register_user(&app, "projadm@test.com", "Admin").await;
    let (member_token, member_id) = register_user(&app, "projmem@test.com", "Member").await;

    create_org(&app, &admin_token, "UpdOrg", "upd-org").await;
    create_project(&app, &admin_token, "upd-org", "Updatable", "updatable").await;

    // Add member to org + project as viewer
    sqlx::query("INSERT INTO organization_members (organization_id, user_id, role) SELECT o.id, $1, 'member' FROM organizations o WHERE o.slug = 'upd-org'")
        .bind(uuid::Uuid::parse_str(&member_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    // Get project id for adding member
    let proj_row = sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM projects WHERE slug = 'updatable'")
        .fetch_one(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project_members (project_id, user_id, role) VALUES ($1, $2, 'viewer')")
        .bind(proj_row)
        .bind(uuid::Uuid::parse_str(&member_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    // Viewer cannot update project
    let (status, _) = patch_auth(
        &app,
        "/api/organizations/upd-org/projects/updatable",
        json!({"name": "Hacked"}),
        &member_token,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Admin can update
    let (status, body) = patch_auth(
        &app,
        "/api/organizations/upd-org/projects/updatable",
        json!({"name": "Updated"}),
        &admin_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Updated");
}

#[tokio::test]
#[serial]
async fn project_delete_requires_admin() {
    let (app, pool) = test_app().await;
    let (admin_token, _) = register_user(&app, "deladm@test.com", "Admin").await;
    let (member_token, member_id) = register_user(&app, "delmem@test.com", "Member").await;

    create_org(&app, &admin_token, "DelOrg", "del-org").await;
    create_project(&app, &admin_token, "del-org", "Deletable", "deletable").await;

    // Add member to org
    sqlx::query("INSERT INTO organization_members (organization_id, user_id, role) SELECT o.id, $1, 'member' FROM organizations o WHERE o.slug = 'del-org'")
        .bind(uuid::Uuid::parse_str(&member_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    // Member without project role cannot delete (gets 404 — resource hiding)
    let (status, _) = delete_auth(
        &app,
        "/api/organizations/del-org/projects/deletable",
        &member_token,
    )
    .await;
    assert!(status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND);

    // Admin can delete
    let (status, _) = delete_auth(
        &app,
        "/api/organizations/del-org/projects/deletable",
        &admin_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

// ── Org admin has implicit project admin ──

#[tokio::test]
#[serial]
async fn org_admin_has_implicit_project_admin() {
    let (app, pool) = test_app().await;
    let (owner_token, _) = register_user(&app, "orgowner@test.com", "Owner").await;
    let (admin_token, admin_id) = register_user(&app, "orgadmin@test.com", "OrgAdmin").await;

    create_org(&app, &owner_token, "Implicit Org", "implicit-org").await;
    create_project(&app, &owner_token, "implicit-org", "Some Project", "some-proj").await;

    // Add user as org admin (not project member)
    sqlx::query("INSERT INTO organization_members (organization_id, user_id, role) SELECT o.id, $1, 'admin' FROM organizations o WHERE o.slug = 'implicit-org'")
        .bind(uuid::Uuid::parse_str(&admin_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    // Org admin can update project even without being a project member
    let (status, body) = patch_auth(
        &app,
        "/api/organizations/implicit-org/projects/some-proj",
        json!({"name": "Admin Updated"}),
        &admin_token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Admin Updated");
}

// ── Add member requires org membership ──

#[tokio::test]
#[serial]
async fn adding_project_member_requires_org_membership() {
    let (app, _pool) = test_app().await;
    let (owner_token, _) = register_user(&app, "pmowner@test.com", "Owner").await;
    let (_, outsider_id) = register_user(&app, "outsider@test.com", "Outsider").await;

    create_org(&app, &owner_token, "PM Org", "pm-org").await;
    create_project(&app, &owner_token, "pm-org", "PM Proj", "pm-proj").await;

    // Try to add non-org-member to project
    let (status, body) = post_json_auth(
        &app,
        "/api/organizations/pm-org/projects/pm-proj/members",
        json!({
            "user_id": outsider_id,
            "role": "viewer"
        }),
        &owner_token,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("organization member"));
}

// ── Pagination ──

#[tokio::test]
#[serial]
async fn project_list_respects_pagination() {
    let (app, _pool) = test_app().await;
    let (token, _) = register_user(&app, "projpag@test.com", "Pag").await;

    create_org(&app, &token, "Pag Org", "pag-org").await;

    for i in 0..5 {
        create_project(
            &app,
            &token,
            "pag-org",
            &format!("P {}", i),
            &format!("p-{}", i),
        )
        .await;
    }

    let (_, body) = get_auth(
        &app,
        "/api/organizations/pag-org/projects?limit=2&offset=0",
        &token,
    )
    .await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["pagination"]["total"], 5);

    let (_, body) = get_auth(
        &app,
        "/api/organizations/pag-org/projects?limit=2&offset=4",
        &token,
    )
    .await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}
