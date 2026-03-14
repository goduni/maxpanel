use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::errors::AppError;
use crate::models::{OrgRole, Organization, OrganizationMember};

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    slug: &str,
) -> Result<Organization, AppError> {
    // Wrap in transaction so org + member are atomic
    let mut tx = pool.begin().await?;
    let org = sqlx::query_as!(
        Organization,
        r#"INSERT INTO organizations (name, slug)
           VALUES ($1, $2)
           RETURNING id, name, slug, created_at, updated_at"#,
        name,
        slug,
    )
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query!(
        r#"INSERT INTO organization_members (organization_id, user_id, role)
           VALUES ($1, $2, $3)"#,
        org.id,
        user_id,
        OrgRole::Owner as OrgRole,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(org)
}

pub async fn list_for_user(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Organization>, i64), AppError> {
    // Run both queries concurrently
    let (orgs_result, total_result) = tokio::join!(
        db::organizations::list_for_user(pool, user_id, limit, offset),
        db::organizations::count_for_user(pool, user_id),
    );
    let orgs = orgs_result?;
    let total = total_result?;
    Ok((orgs, total))
}

pub async fn get_by_slug(
    pool: &PgPool,
    user_id: Uuid,
    slug: &str,
) -> Result<Organization, AppError> {
    let (org, _role) = resolve_org(pool, user_id, slug).await?;
    Ok(org)
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    slug: &str,
    name: &str,
) -> Result<Organization, AppError> {
    let (org, role) = resolve_org(pool, user_id, slug).await?;
    if role.privilege_level() < OrgRole::Admin.privilege_level() {
        return Err(AppError::Forbidden);
    }
    let updated = db::organizations::update(pool, org.id, name).await?;
    Ok(updated)
}

pub async fn delete(
    pool: &PgPool,
    user_id: Uuid,
    slug: &str,
) -> Result<(), AppError> {
    let (org, role) = resolve_org(pool, user_id, slug).await?;
    if role.privilege_level() < OrgRole::Owner.privilege_level() {
        return Err(AppError::Forbidden);
    }

    tracing::info!(
        target: "audit",
        actor_id = %user_id,
        org_id = %org.id,
        org_slug = %slug,
        "organization deleted"
    );

    // Wrap event cleanup + org delete in one transaction
    let mut tx = pool.begin().await?;
    db::events::delete_for_org(&mut tx, org.id).await?;
    sqlx::query!("DELETE FROM organizations WHERE id = $1", org.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn transfer_ownership(
    pool: &PgPool,
    actor_id: Uuid,
    slug: &str,
    new_owner_id: Uuid,
) -> Result<(), AppError> {
    if actor_id == new_owner_id {
        return Ok(());
    }

    let (org, role) = resolve_org(pool, actor_id, slug).await?;
    if role.privilege_level() < OrgRole::Owner.privilege_level() {
        return Err(AppError::Forbidden);
    }

    let is_member = db::organizations::is_member(pool, org.id, new_owner_id).await?;
    if !is_member {
        return Err(AppError::BadRequest("Target user is not a member".into()));
    }

    tracing::info!(
        target: "audit",
        actor_id = %actor_id,
        org_id = %org.id,
        new_owner_id = %new_owner_id,
        "ownership transferred"
    );

    db::organizations::transfer_ownership(pool, org.id, actor_id, new_owner_id).await?;
    Ok(())
}

pub async fn list_members(
    pool: &PgPool,
    user_id: Uuid,
    slug: &str,
) -> Result<Vec<OrganizationMember>, AppError> {
    let org = get_by_slug(pool, user_id, slug).await?;
    let members = db::organizations::list_members(pool, org.id).await?;
    Ok(members)
}

pub async fn update_member_role(
    pool: &PgPool,
    actor_id: Uuid,
    slug: &str,
    target_user_id: Uuid,
    new_role: OrgRole,
) -> Result<(), AppError> {
    if new_role == OrgRole::Owner {
        return Err(AppError::BadRequest(
            "Use transfer-ownership endpoint to assign owner role".into(),
        ));
    }

    let (org, role) = resolve_org(pool, actor_id, slug).await?;
    if role.privilege_level() < OrgRole::Admin.privilege_level() {
        return Err(AppError::Forbidden);
    }
    let actor_role = role;

    if !actor_role.can_assign(new_role) {
        return Err(AppError::Forbidden);
    }

    let target_role = db::organizations::get_member_role(pool, org.id, target_user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if target_role == OrgRole::Owner {
        return Err(AppError::Forbidden);
    }

    tracing::info!(
        target: "audit",
        actor_id = %actor_id,
        target_user_id = %target_user_id,
        org_id = %org.id,
        old_role = ?target_role,
        new_role = ?new_role,
        "org member role changed"
    );

    db::organizations::update_member_role(pool, org.id, target_user_id, new_role).await?;
    Ok(())
}

pub async fn remove_member(
    pool: &PgPool,
    actor_id: Uuid,
    slug: &str,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    if actor_id == target_user_id {
        return Err(AppError::BadRequest("Cannot remove yourself".into()));
    }

    let (org, role) = resolve_org(pool, actor_id, slug).await?;
    if role.privilege_level() < OrgRole::Admin.privilege_level() {
        return Err(AppError::Forbidden);
    }

    let target_role = db::organizations::get_member_role(pool, org.id, target_user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if target_role == OrgRole::Owner {
        return Err(AppError::BadRequest("Cannot remove the owner".into()));
    }

    let actor_role = role;
    if actor_role.privilege_level() <= target_role.privilege_level() {
        return Err(AppError::Forbidden);
    }

    db::organizations::remove_member(pool, org.id, target_user_id).await?;
    Ok(())
}

/// Returns the actor's role if it meets the minimum required level.
pub async fn require_role(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
    minimum: OrgRole,
) -> Result<OrgRole, AppError> {
    let role = db::organizations::get_member_role(pool, org_id, user_id)
        .await?
        .ok_or(AppError::Forbidden)?;

    if role.privilege_level() < minimum.privilege_level() {
        return Err(AppError::Forbidden);
    }

    Ok(role)
}

/// Resolve org by slug and verify membership, returning (org, role).
/// Uses a single JOIN query to avoid two round-trips.
pub async fn resolve_org(
    pool: &PgPool,
    user_id: Uuid,
    slug: &str,
) -> Result<(Organization, OrgRole), AppError> {
    db::organizations::find_by_slug_with_role(pool, slug, user_id)
        .await?
        .ok_or(AppError::NotFound)
}
