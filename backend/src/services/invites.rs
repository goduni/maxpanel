use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::db;
use crate::errors::AppError;
use crate::models::{InviteResponse, OrgRole};
use crate::services::{crypto, organizations};

pub async fn create_invite(
    pool: &PgPool,
    config: &Config,
    actor_id: Uuid,
    org_slug: &str,
    email: &str,
    role: OrgRole,
) -> Result<(InviteResponse, String), AppError> {
    if role == OrgRole::Owner {
        return Err(AppError::BadRequest(
            "Cannot invite as owner. Use transfer-ownership.".into(),
        ));
    }

    let (org, actor_role) = organizations::resolve_org(pool, actor_id, org_slug).await?;
    if actor_role.privilege_level() < OrgRole::Admin.privilege_level() {
        return Err(AppError::Forbidden);
    }

    // Generate invite token
    let token = Uuid::new_v4().to_string();
    let token_hash = crypto::hash_token(&config.invite_token_hmac_secret, &token);
    let expires_at = Utc::now() + Duration::days(config.invite_ttl_days);

    let invite = db::invites::create(pool, org.id, email, role, &token_hash, actor_id, expires_at)
        .await?;

    tracing::info!(
        target: "audit",
        actor_id = %actor_id,
        email = %email,
        org_id = %org.id,
        role = ?role,
        "invite sent"
    );

    Ok((invite.into(), token))
}

pub async fn accept_invite(
    pool: &PgPool,
    config: &Config,
    user_id: Uuid,
    user_email: &str,
    token: &str,
) -> Result<(), AppError> {
    let token_hash = crypto::hash_token(&config.invite_token_hmac_secret, token);
    let invite = db::invites::find_by_token_hash(pool, &token_hash)
        .await?
        .ok_or(AppError::NotFound)?;

    // Email must match
    if invite.email != user_email {
        return Err(AppError::Forbidden);
    }

    // Atomically claim the invite to prevent concurrent double-accept (TOCTOU fix)
    let mut tx = pool.begin().await?;
    let rows = sqlx::query(
        "UPDATE invites SET accepted_at = now() WHERE id = $1 AND accepted_at IS NULL AND revoked_at IS NULL AND expires_at > now()",
    )
    .bind(invite.id)
    .execute(&mut *tx)
    .await?;
    if rows.rows_affected() == 0 {
        return Err(AppError::Conflict("Invite already accepted or expired".into()));
    }

    sqlx::query!(
        r#"INSERT INTO organization_members (organization_id, user_id, role)
           VALUES ($1, $2, $3)
           ON CONFLICT (organization_id, user_id) DO NOTHING"#,
        invite.organization_id,
        user_id,
        invite.role as crate::models::OrgRole,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    tracing::info!(
        target: "audit",
        user_id = %user_id,
        email = %user_email,
        org_id = %invite.organization_id,
        role = ?invite.role,
        "invite accepted"
    );

    Ok(())
}

pub async fn list_pending(
    pool: &PgPool,
    actor_id: Uuid,
    org_slug: &str,
) -> Result<Vec<InviteResponse>, AppError> {
    let (org, actor_role) = organizations::resolve_org(pool, actor_id, org_slug).await?;
    if actor_role.privilege_level() < OrgRole::Admin.privilege_level() {
        return Err(AppError::Forbidden);
    }

    let invites = db::invites::list_pending_for_org(pool, org.id).await?;
    Ok(invites.into_iter().map(Into::into).collect())
}

pub async fn revoke_invite(
    pool: &PgPool,
    actor_id: Uuid,
    org_slug: &str,
    invite_id: Uuid,
) -> Result<(), AppError> {
    let (org, actor_role) = organizations::resolve_org(pool, actor_id, org_slug).await?;
    if actor_role.privilege_level() < OrgRole::Admin.privilege_level() {
        return Err(AppError::Forbidden);
    }

    let invite = db::invites::find_by_id(pool, invite_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if invite.organization_id != org.id {
        return Err(AppError::NotFound);
    }

    tracing::info!(
        target: "audit",
        actor_id = %actor_id,
        invite_id = %invite_id,
        org_id = %org.id,
        "invite revoked"
    );

    db::invites::revoke(pool, invite_id).await?;
    Ok(())
}

pub async fn list_for_user(
    pool: &PgPool,
    email: &str,
) -> Result<Vec<InviteResponse>, AppError> {
    let invites = db::invites::list_for_email(pool, email).await?;
    Ok(invites.into_iter().map(Into::into).collect())
}
