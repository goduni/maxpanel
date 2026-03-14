use axum::routing::{delete, get, patch, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::app_state::AppState;
use crate::handlers;
use crate::openapi::ApiDoc;

pub fn build_router(state: AppState) -> Router {
    // Auth routes
    let auth_routes = Router::new()
        .route("/register", post(handlers::auth::register))
        .route("/login", post(handlers::auth::login))
        .route("/refresh", post(handlers::auth::refresh))
        .route("/logout", post(handlers::auth::logout))
        .route("/logout-all", post(handlers::auth::logout_all))
        .route("/me", get(handlers::auth::me).patch(handlers::auth::update_me))
        .route("/me/invites", get(handlers::invites::list_my_invites))
        .route("/change-password", post(handlers::auth::change_password));

    // Organization member routes
    let org_member_routes = Router::new()
        .route("/", get(handlers::organizations::list_members))
        .route("/{user_id}", patch(handlers::organizations::update_member_role)
            .delete(handlers::organizations::remove_member));

    // Project member routes
    let project_member_routes = Router::new()
        .route("/", get(handlers::projects::list_members)
            .post(handlers::projects::add_member))
        .route("/{user_id}", patch(handlers::projects::update_member_role)
            .delete(handlers::projects::remove_member));

    // Bot CRUD routes (nested under org/project)
    let bot_crud_routes = Router::new()
        .route("/", get(handlers::bots::list)
            .post(handlers::bots::create))
        .route("/{bot_id}", get(handlers::bots::get)
            .patch(handlers::bots::update)
            .delete(handlers::bots::delete));

    // Project routes
    let project_routes = Router::new()
        .route("/", get(handlers::projects::list)
            .post(handlers::projects::create))
        .route("/{project_slug}", get(handlers::projects::get)
            .patch(handlers::projects::update)
            .delete(handlers::projects::delete))
        .nest("/{project_slug}/members", project_member_routes)
        .nest("/{project_slug}/bots", bot_crud_routes);

    // Invite routes
    let invite_routes = Router::new()
        .route("/", get(handlers::invites::list_pending)
            .post(handlers::invites::create_invite))
        .route("/{invite_id}", delete(handlers::invites::revoke_invite));

    // Organization routes
    let org_routes = Router::new()
        .route("/", get(handlers::organizations::list)
            .post(handlers::organizations::create))
        .route("/{org_slug}", get(handlers::organizations::get)
            .patch(handlers::organizations::update)
            .delete(handlers::organizations::delete))
        .route("/{org_slug}/transfer-ownership", post(handlers::organizations::transfer_ownership))
        .nest("/{org_slug}/members", org_member_routes)
        .nest("/{org_slug}/invites", invite_routes)
        .nest("/{org_slug}/projects", project_routes);

    // Flat bot endpoints
    let bot_flat_routes = Router::new()
        .route("/{bot_id}/start", post(handlers::bots::start))
        .route("/{bot_id}/stop", post(handlers::bots::stop))
        .route("/{bot_id}/verify", post(handlers::bots::verify))
        .route("/{bot_id}/events", get(handlers::events::list_events))
        .route("/{bot_id}/events/{event_id}", get(handlers::events::get_event))
        .route("/{bot_id}/chats", get(handlers::bot_chats::list_bot_chats))
        .route("/{bot_id}/chats/sync", post(handlers::bot_chats::sync_chats))
        .route("/{bot_id}/chats/{chat_id}/events", get(handlers::events::list_chat_events))
        .route("/{bot_id}/chats/{chat_id}/sync-history", post(handlers::bot_chats::sync_chat_history))
        .route("/{bot_id}/chats/{chat_id}/history", get(handlers::bot_chats::proxy_chat_history))
        .route("/{bot_id}/max", post(handlers::max_api::raw_proxy))
        .route("/{bot_id}/media-info/{token}", get(handlers::media_proxy::media_info))
        .route("/{bot_id}/media-stream/{token}", get(handlers::media_proxy::media_stream))
        .route("/{bot_id}/api-keys", post(handlers::api_keys::create_api_key).get(handlers::api_keys::list_api_keys))
        .route("/{bot_id}/api-keys/{key_id}", delete(handlers::api_keys::delete_api_key))
        .route("/{bot_id}/gateway", post(handlers::gateway::gateway))
        .route("/{bot_id}/outgoing-events", post(handlers::ingestion_api::ingest_outgoing));

    // Invite accept (separate path)
    let invite_accept = Router::new()
        .route("/{token}/accept", post(handlers::invites::accept_invite));

    // Webhook route
    let webhook_routes = Router::new()
        .route("/{webhook_secret}", post(handlers::webhooks::handle_webhook));

    // Assemble
    let mut app = Router::new()
        .route("/health", get(|| async { axum::Json(serde_json::json!({"status": "ok"})) }))
        .nest("/api/auth", auth_routes)
        .nest("/api/organizations", org_routes)
        .nest("/api/bots", bot_flat_routes)
        .nest("/api/invites", invite_accept)
        .nest("/webhooks", webhook_routes);

    // Swagger UI only in non-production
    if !state.config.is_production() {
        app = app.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
    }

    app.with_state(state)
}
