use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Project, ProjectMember, ProjectRole};

pub async fn create(pool: &PgPool, org_id: Uuid, name: &str, slug: &str) -> Result<Project, sqlx::Error> {
    sqlx::query_as!(
        Project,
        r#"INSERT INTO projects (organization_id, name, slug)
           VALUES ($1, $2, $3)
           RETURNING id, organization_id, name, slug, created_at, updated_at"#,
        org_id,
        name,
        slug,
    )
    .fetch_one(pool)
    .await
}

pub async fn find_by_slug(pool: &PgPool, org_id: Uuid, slug: &str) -> Result<Option<Project>, sqlx::Error> {
    sqlx::query_as!(
        Project,
        r#"SELECT id, organization_id, name, slug, created_at, updated_at
           FROM projects WHERE organization_id = $1 AND slug = $2"#,
        org_id,
        slug,
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_for_org(pool: &PgPool, org_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as!(
        Project,
        r#"SELECT id, organization_id, name, slug, created_at, updated_at
           FROM projects WHERE organization_id = $1
           ORDER BY created_at DESC
           LIMIT $2 OFFSET $3"#,
        org_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
}

pub async fn count_for_org(pool: &PgPool, org_id: Uuid) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM projects WHERE organization_id = $1"#,
        org_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_for_org_member(pool: &PgPool, org_id: Uuid, user_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as!(
        Project,
        r#"SELECT p.id, p.organization_id, p.name, p.slug, p.created_at, p.updated_at
           FROM projects p
           JOIN project_members pm ON pm.project_id = p.id
           WHERE p.organization_id = $1 AND pm.user_id = $2
           ORDER BY p.created_at DESC LIMIT $3 OFFSET $4"#,
        org_id,
        user_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
}

pub async fn count_for_org_member(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM projects p
           JOIN project_members pm ON pm.project_id = p.id
           WHERE p.organization_id = $1 AND pm.user_id = $2"#,
        org_id,
        user_id,
    )
    .fetch_one(pool)
    .await
}

pub async fn update(pool: &PgPool, id: Uuid, name: &str) -> Result<Project, sqlx::Error> {
    sqlx::query_as!(
        Project,
        r#"UPDATE projects SET name = $2 WHERE id = $1
           RETURNING id, organization_id, name, slug, created_at, updated_at"#,
        id,
        name,
    )
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM projects WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_member(pool: &PgPool, project_id: Uuid, user_id: Uuid, role: ProjectRole) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO project_members (project_id, user_id, role)
           VALUES ($1, $2, $3)"#,
        project_id,
        user_id,
        role as ProjectRole,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_member_role(pool: &PgPool, project_id: Uuid, user_id: Uuid) -> Result<Option<ProjectRole>, sqlx::Error> {
    let row = sqlx::query_scalar!(
        r#"SELECT role as "role: ProjectRole" FROM project_members
           WHERE project_id = $1 AND user_id = $2"#,
        project_id,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_members(pool: &PgPool, project_id: Uuid) -> Result<Vec<ProjectMember>, sqlx::Error> {
    sqlx::query_as!(
        ProjectMember,
        r#"SELECT pm.project_id, pm.user_id, pm.role as "role: ProjectRole", pm.created_at,
                  u.name as "user_name: Option<String>", u.email as "user_email: Option<String>"
           FROM project_members pm
           LEFT JOIN users u ON u.id = pm.user_id
           WHERE pm.project_id = $1
           ORDER BY pm.created_at"#,
        project_id,
    )
    .fetch_all(pool)
    .await
}

pub async fn update_member_role(pool: &PgPool, project_id: Uuid, user_id: Uuid, role: ProjectRole) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE project_members SET role = $3
           WHERE project_id = $1 AND user_id = $2"#,
        project_id,
        user_id,
        role as ProjectRole,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_member(pool: &PgPool, project_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM project_members WHERE project_id = $1 AND user_id = $2",
        project_id,
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}
