use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{OrgRole, Organization, OrganizationMember};

#[derive(sqlx::FromRow)]
struct OrgWithRole {
    id: Uuid,
    name: String,
    slug: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    role: OrgRole,
}

pub async fn find_by_slug_with_role(pool: &PgPool, slug: &str, user_id: Uuid) -> Result<Option<(Organization, OrgRole)>, sqlx::Error> {
    let row = sqlx::query_as::<_, OrgWithRole>(
        r#"SELECT o.id, o.name, o.slug, o.created_at, o.updated_at,
                  om.role
           FROM organizations o
           JOIN organization_members om ON o.id = om.organization_id
           WHERE o.slug = $1 AND om.user_id = $2"#,
    )
    .bind(slug)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| (
        Organization {
            id: r.id,
            name: r.name,
            slug: r.slug,
            created_at: r.created_at,
            updated_at: r.updated_at,
        },
        r.role,
    )))
}

pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Organization>, sqlx::Error> {
    sqlx::query_as!(
        Organization,
        "SELECT id, name, slug, created_at, updated_at FROM organizations WHERE slug = $1",
        slug,
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_for_user(pool: &PgPool, user_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Organization>, sqlx::Error> {
    sqlx::query_as!(
        Organization,
        r#"SELECT o.id, o.name, o.slug, o.created_at, o.updated_at
           FROM organizations o
           JOIN organization_members om ON o.id = om.organization_id
           WHERE om.user_id = $1
           ORDER BY o.created_at DESC
           LIMIT $2 OFFSET $3"#,
        user_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
}

pub async fn count_for_user(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM organization_members WHERE user_id = $1"#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update(pool: &PgPool, id: Uuid, name: &str) -> Result<Organization, sqlx::Error> {
    sqlx::query_as!(
        Organization,
        r#"UPDATE organizations SET name = $2 WHERE id = $1
           RETURNING id, name, slug, created_at, updated_at"#,
        id,
        name,
    )
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM organizations WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_member(pool: &PgPool, org_id: Uuid, user_id: Uuid, role: OrgRole) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO organization_members (organization_id, user_id, role)
           VALUES ($1, $2, $3)"#,
        org_id,
        user_id,
        role as OrgRole,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_member_role(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> Result<Option<OrgRole>, sqlx::Error> {
    let row = sqlx::query_scalar!(
        r#"SELECT role as "role: OrgRole" FROM organization_members
           WHERE organization_id = $1 AND user_id = $2"#,
        org_id,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_members(pool: &PgPool, org_id: Uuid) -> Result<Vec<OrganizationMember>, sqlx::Error> {
    sqlx::query_as!(
        OrganizationMember,
        r#"SELECT om.organization_id, om.user_id, om.role as "role: OrgRole", om.created_at,
                  u.name as "user_name: Option<String>", u.email as "user_email: Option<String>"
           FROM organization_members om
           LEFT JOIN users u ON u.id = om.user_id
           WHERE om.organization_id = $1
           ORDER BY om.created_at"#,
        org_id,
    )
    .fetch_all(pool)
    .await
}

pub async fn update_member_role(pool: &PgPool, org_id: Uuid, user_id: Uuid, role: OrgRole) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE organization_members SET role = $3
           WHERE organization_id = $1 AND user_id = $2"#,
        org_id,
        user_id,
        role as OrgRole,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_member(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM organization_members WHERE organization_id = $1 AND user_id = $2",
        org_id,
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn is_member(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM organization_members WHERE organization_id = $1 AND user_id = $2) as "exists!""#,
        org_id,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

pub async fn transfer_ownership(pool: &PgPool, org_id: Uuid, current_owner_id: Uuid, new_owner_id: Uuid) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query!(
        r#"UPDATE organization_members SET role = 'admin'
           WHERE organization_id = $1 AND user_id = $2"#,
        org_id,
        current_owner_id,
    )
    .execute(&mut *tx)
    .await?;
    let rows = sqlx::query!(
        r#"UPDATE organization_members SET role = 'owner'
           WHERE organization_id = $1 AND user_id = $2"#,
        org_id,
        new_owner_id,
    )
    .execute(&mut *tx)
    .await?;
    if rows.rows_affected() != 1 {
        return Err(sqlx::Error::RowNotFound);
    }
    tx.commit().await?;
    Ok(())
}
