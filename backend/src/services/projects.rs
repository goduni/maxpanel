use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::errors::AppError;
use crate::models::{OrgRole, Project, ProjectMember, ProjectRole};
use crate::services::organizations;

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    org_slug: &str,
    name: &str,
    slug: &str,
) -> Result<Project, AppError> {
    let (org, org_role) = organizations::resolve_org(pool, user_id, org_slug).await?;
    if org_role.privilege_level() < OrgRole::Admin.privilege_level() {
        return Err(AppError::Forbidden);
    }

    // Wrap in transaction so project + member are atomic
    let mut tx = pool.begin().await?;
    let project = sqlx::query_as!(
        Project,
        r#"INSERT INTO projects (organization_id, name, slug)
           VALUES ($1, $2, $3)
           RETURNING id, organization_id, name, slug, created_at, updated_at"#,
        org.id,
        name,
        slug,
    )
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query!(
        r#"INSERT INTO project_members (project_id, user_id, role)
           VALUES ($1, $2, $3)"#,
        project.id,
        user_id,
        ProjectRole::Admin as ProjectRole,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(project)
}

pub async fn list_for_org(
    pool: &PgPool,
    user_id: Uuid,
    org_slug: &str,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Project>, i64), AppError> {
    let (org, role) = organizations::resolve_org(pool, user_id, org_slug).await?;
    if role.privilege_level() >= OrgRole::Admin.privilege_level() {
        // Admins see all projects
        let (projects_result, total_result) = tokio::join!(
            db::projects::list_for_org(pool, org.id, limit, offset),
            db::projects::count_for_org(pool, org.id),
        );
        let projects = projects_result?;
        let total = total_result?;
        Ok((projects, total))
    } else {
        // Members see only their projects
        let (projects_result, total_result) = tokio::join!(
            db::projects::list_for_org_member(pool, org.id, user_id, limit, offset),
            db::projects::count_for_org_member(pool, org.id, user_id),
        );
        let projects = projects_result?;
        let total = total_result?;
        Ok((projects, total))
    }
}

pub async fn get_by_slug(
    pool: &PgPool,
    user_id: Uuid,
    org_slug: &str,
    project_slug: &str,
) -> Result<Project, AppError> {
    let (org, role) = organizations::resolve_org(pool, user_id, org_slug).await?;
    let project = db::projects::find_by_slug(pool, org.id, project_slug)
        .await?
        .ok_or(AppError::NotFound)?;
    // Non-admin org members must be project members
    if role.privilege_level() < OrgRole::Admin.privilege_level() {
        let is_member = db::projects::get_member_role(pool, project.id, user_id).await?;
        if is_member.is_none() {
            return Err(AppError::NotFound);
        }
    }
    Ok(project)
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    org_slug: &str,
    project_slug: &str,
    name: &str,
) -> Result<Project, AppError> {
    let (org, org_role) = organizations::resolve_org(pool, user_id, org_slug).await?;
    let project = db::projects::find_by_slug(pool, org.id, project_slug)
        .await?
        .ok_or(AppError::NotFound)?;
    require_project_admin_with_org_role(pool, user_id, &project, org_role).await?;
    let updated = db::projects::update(pool, project.id, name).await?;
    Ok(updated)
}

pub async fn delete(
    pool: &PgPool,
    user_id: Uuid,
    org_slug: &str,
    project_slug: &str,
) -> Result<(), AppError> {
    let (org, org_role) = organizations::resolve_org(pool, user_id, org_slug).await?;
    let project = db::projects::find_by_slug(pool, org.id, project_slug)
        .await?
        .ok_or(AppError::NotFound)?;
    require_project_admin_with_org_role(pool, user_id, &project, org_role).await?;

    tracing::info!(
        target: "audit",
        actor_id = %user_id,
        project_id = %project.id,
        project_slug = %project_slug,
        "project deleted"
    );

    // Clean up events for bots in this project before deleting (events table is partitioned, no FK cascade)
    let mut tx = pool.begin().await?;
    db::events::delete_for_project(&mut tx, project.id).await?;
    sqlx::query!("DELETE FROM projects WHERE id = $1", project.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn list_members(
    pool: &PgPool,
    user_id: Uuid,
    org_slug: &str,
    project_slug: &str,
) -> Result<Vec<ProjectMember>, AppError> {
    let project = get_by_slug(pool, user_id, org_slug, project_slug).await?;
    let members = db::projects::list_members(pool, project.id).await?;
    Ok(members)
}

pub async fn add_member(
    pool: &PgPool,
    actor_id: Uuid,
    org_slug: &str,
    project_slug: &str,
    target_user_id: Uuid,
    role: ProjectRole,
) -> Result<(), AppError> {
    let (org, org_role) = organizations::resolve_org(pool, actor_id, org_slug).await?;
    let project = db::projects::find_by_slug(pool, org.id, project_slug)
        .await?
        .ok_or(AppError::NotFound)?;

    require_project_admin_with_org_role(pool, actor_id, &project, org_role).await?;

    // Target must be org member
    let is_org_member = db::organizations::is_member(pool, org.id, target_user_id).await?;
    if !is_org_member {
        return Err(AppError::BadRequest(
            "User must be an organization member first".into(),
        ));
    }

    db::projects::add_member(pool, project.id, target_user_id, role).await?;
    Ok(())
}

pub async fn update_member_role(
    pool: &PgPool,
    actor_id: Uuid,
    org_slug: &str,
    project_slug: &str,
    target_user_id: Uuid,
    new_role: ProjectRole,
) -> Result<(), AppError> {
    let (org, org_role) = organizations::resolve_org(pool, actor_id, org_slug).await?;
    let project = db::projects::find_by_slug(pool, org.id, project_slug)
        .await?
        .ok_or(AppError::NotFound)?;
    let actor_effective = get_effective_project_role_with_org(pool, actor_id, &project, org_role).await?;

    if !can_manage_project(&actor_effective) {
        return Err(AppError::Forbidden);
    }

    tracing::info!(
        target: "audit",
        actor_id = %actor_id,
        target_user_id = %target_user_id,
        project_id = %project.id,
        new_role = ?new_role,
        "project member role changed"
    );

    db::projects::update_member_role(pool, project.id, target_user_id, new_role).await?;
    Ok(())
}

pub async fn remove_member(
    pool: &PgPool,
    actor_id: Uuid,
    org_slug: &str,
    project_slug: &str,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    if actor_id == target_user_id {
        return Err(AppError::BadRequest("Cannot remove yourself".into()));
    }

    let (org, org_role) = organizations::resolve_org(pool, actor_id, org_slug).await?;
    let project = db::projects::find_by_slug(pool, org.id, project_slug)
        .await?
        .ok_or(AppError::NotFound)?;
    require_project_admin_with_org_role(pool, actor_id, &project, org_role).await?;
    db::projects::remove_member(pool, project.id, target_user_id).await?;
    Ok(())
}

/// The effective role for project access: org:admin+ can manage any project.
#[derive(Debug)]
pub enum EffectiveProjectRole {
    OrgLevel(OrgRole),
    ProjectLevel(ProjectRole),
}

/// Variant that accepts a pre-resolved OrgRole to avoid redundant DB lookups.
pub async fn get_effective_project_role_with_org(
    pool: &PgPool,
    user_id: Uuid,
    project: &Project,
    org_role: OrgRole,
) -> Result<EffectiveProjectRole, AppError> {
    // Org admin+ has implicit project admin
    if org_role.privilege_level() >= OrgRole::Admin.privilege_level() {
        return Ok(EffectiveProjectRole::OrgLevel(org_role));
    }

    // Check project-level role
    let proj_role = db::projects::get_member_role(pool, project.id, user_id).await?;
    match proj_role {
        Some(role) => Ok(EffectiveProjectRole::ProjectLevel(role)),
        None => Err(AppError::NotFound),
    }
}

fn can_manage_project(role: &EffectiveProjectRole) -> bool {
    match role {
        EffectiveProjectRole::OrgLevel(r) => r.privilege_level() >= OrgRole::Admin.privilege_level(),
        EffectiveProjectRole::ProjectLevel(r) => *r == ProjectRole::Admin,
    }
}

/// Variant that accepts a pre-resolved OrgRole.
pub async fn require_project_admin_with_org_role(
    pool: &PgPool,
    user_id: Uuid,
    project: &Project,
    org_role: OrgRole,
) -> Result<(), AppError> {
    let role = get_effective_project_role_with_org(pool, user_id, project, org_role).await?;
    if !can_manage_project(&role) {
        return Err(AppError::Forbidden);
    }
    Ok(())
}
