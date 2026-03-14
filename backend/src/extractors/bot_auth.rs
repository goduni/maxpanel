use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::db;
use crate::db::bots::BotAuthRow;
use crate::errors::AppError;
use crate::extractors::AuthUser;
use crate::models::{OrgRole, ProjectRole};

/// Type-level authorization for flat bot endpoints (`/api/bots/:bot_id/*`).
/// Any handler that takes this extractor gets authorization for free.
#[derive(Debug)]
pub struct BotAuthContext {
    pub user_id: Uuid,
    pub auth_row: BotAuthRow,
    pub effective_role: EffectiveRole,
}

#[derive(Debug, Clone)]
pub enum EffectiveRole {
    Org(OrgRole),
    Project(ProjectRole),
}

impl EffectiveRole {
    /// Can send named Max API calls (proj:editor+)
    pub fn can_send_api(&self) -> bool {
        match self {
            EffectiveRole::Org(r) => r.privilege_level() >= OrgRole::Admin.privilege_level(),
            EffectiveRole::Project(r) => r.privilege_level() >= ProjectRole::Editor.privilege_level(),
        }
    }

    /// Can read bot data (events, chats). Requires project membership or org:admin+.
    pub fn can_read(&self) -> bool {
        match self {
            EffectiveRole::Org(r) => r.privilege_level() >= OrgRole::Admin.privilege_level(),
            EffectiveRole::Project(_) => true,
        }
    }

    /// Can manage bot (proj:admin+) or send raw API
    pub fn can_manage(&self) -> bool {
        match self {
            EffectiveRole::Org(r) => r.privilege_level() >= OrgRole::Admin.privilege_level(),
            EffectiveRole::Project(r) => *r == ProjectRole::Admin,
        }
    }
}

impl FromRequestParts<AppState> for BotAuthContext {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let auth_user = AuthUser::from_request_parts(parts, state).await?;

            // Extract bot_id by name from path params (works with any number of path segments)
            let params = axum::extract::Path::<std::collections::HashMap<String, String>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AppError::BadRequest("Missing path parameters".into()))?;
            let bot_id: Uuid = params.get("bot_id")
                .ok_or_else(|| AppError::BadRequest("Missing bot_id".into()))?
                .parse()
                .map_err(|_| AppError::BadRequest("Invalid bot_id".into()))?;

            let auth_row = db::bots::resolve_bot_auth(&state.db, bot_id, auth_user.user_id)
                .await?
                .ok_or(AppError::NotFound)?;

            // Compute effective role
            let effective_role = match (&auth_row.org_role, &auth_row.proj_role) {
                (Some(org_role), Some(proj_role)) => {
                    // Org admin+ gets org-level access; otherwise use project role
                    if org_role.privilege_level() >= OrgRole::Admin.privilege_level() {
                        EffectiveRole::Org(*org_role)
                    } else if proj_role.privilege_level() > 0 {
                        EffectiveRole::Project(*proj_role)
                    } else {
                        // Org member with no meaningful project role
                        return Err(AppError::Forbidden);
                    }
                }
                (Some(org_role), None) => EffectiveRole::Org(*org_role),
                (None, Some(proj_role)) => EffectiveRole::Project(*proj_role),
                (None, None) => return Err(AppError::NotFound),
            };

            Ok(BotAuthContext {
                user_id: auth_user.user_id,
                auth_row,
                effective_role,
            })
        }
    }
}
