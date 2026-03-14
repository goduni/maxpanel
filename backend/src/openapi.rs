use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "MaxPanel API",
        description = "API for managing Max messenger bots",
        version = "0.1.0",
        license(name = "BSL-1.1"),
    ),
    paths(
        // Auth
        crate::handlers::auth::register,
        crate::handlers::auth::login,
        crate::handlers::auth::refresh,
        crate::handlers::auth::logout,
        crate::handlers::auth::logout_all,
        crate::handlers::auth::me,
        crate::handlers::auth::update_me,
        crate::handlers::auth::change_password,
        // Organizations
        crate::handlers::organizations::create,
        crate::handlers::organizations::list,
        crate::handlers::organizations::get,
        crate::handlers::organizations::update,
        crate::handlers::organizations::delete,
        crate::handlers::organizations::transfer_ownership,
        crate::handlers::organizations::list_members,
        crate::handlers::organizations::update_member_role,
        crate::handlers::organizations::remove_member,
        // Projects
        crate::handlers::projects::create,
        crate::handlers::projects::list,
        crate::handlers::projects::get,
        crate::handlers::projects::update,
        crate::handlers::projects::delete,
        crate::handlers::projects::list_members,
        crate::handlers::projects::add_member,
        crate::handlers::projects::update_member_role,
        crate::handlers::projects::remove_member,
        // Bots
        crate::handlers::bots::create,
        crate::handlers::bots::list,
        crate::handlers::bots::get,
        crate::handlers::bots::update,
        crate::handlers::bots::delete,
        crate::handlers::bots::start,
        crate::handlers::bots::stop,
        crate::handlers::bots::verify,
        // Events
        crate::handlers::events::list_events,
        crate::handlers::events::get_event,
        crate::handlers::bot_chats::list_bot_chats,
        crate::handlers::bot_chats::sync_chats,
        crate::handlers::bot_chats::sync_chat_history,
        crate::handlers::bot_chats::proxy_chat_history,
        crate::handlers::events::list_chat_events,
        // Invites
        crate::handlers::invites::create_invite,
        crate::handlers::invites::list_pending,
        crate::handlers::invites::revoke_invite,
        crate::handlers::invites::accept_invite,
        crate::handlers::invites::list_my_invites,
        // Max API proxy
        crate::handlers::max_api::raw_proxy,
        // Media
        crate::handlers::media_proxy::media_info,
        crate::handlers::media_proxy::media_stream,
        // API Keys
        crate::handlers::api_keys::create_api_key,
        crate::handlers::api_keys::list_api_keys,
        crate::handlers::api_keys::delete_api_key,
        // Gateway
        crate::handlers::gateway::gateway,
        // Ingestion API
        crate::handlers::ingestion_api::ingest_outgoing,
        // Webhooks
        crate::handlers::webhooks::handle_webhook,
    ),
    components(schemas(
        // Models
        crate::models::UserResponse,
        crate::models::Organization,
        crate::models::OrganizationMember,
        crate::models::OrgRole,
        crate::models::Project,
        crate::models::ProjectMember,
        crate::models::ProjectRole,
        crate::models::BotResponse,
        crate::models::EventMode,
        crate::models::Event,
        crate::models::BotChat,
        crate::models::InviteResponse,
        // Auth responses
        crate::models::AuthTokens,
        crate::models::LoginResponse,
        // Common responses
        crate::handlers::common::OkResponse,
        crate::handlers::common::PaginationInfo,
        crate::handlers::common::CursorPaginationInfo,
        crate::handlers::common::PaginationQuery,
        // List responses
        crate::handlers::organizations::OrganizationListResponse,
        crate::handlers::organizations::MembersResponse,
        crate::handlers::projects::ProjectListResponse,
        crate::handlers::projects::ProjectMembersResponse,
        crate::handlers::bots::BotListResponse,
        crate::handlers::events::EventListResponse,
        crate::handlers::invites::CreateInviteResponse,
        crate::handlers::invites::InviteListResponse,
        // Requests
        crate::handlers::auth::RegisterRequest,
        crate::handlers::auth::LoginRequest,
        crate::handlers::auth::RefreshRequest,
        crate::handlers::auth::LogoutRequest,
        crate::handlers::auth::ChangePasswordRequest,
        crate::handlers::auth::UpdateMeRequest,
        crate::handlers::organizations::CreateOrgRequest,
        crate::handlers::organizations::UpdateOrgRequest,
        crate::handlers::organizations::TransferOwnershipRequest,
        crate::handlers::organizations::UpdateMemberRoleRequest,
        crate::handlers::projects::CreateProjectRequest,
        crate::handlers::projects::UpdateProjectRequest,
        crate::handlers::projects::AddMemberRequest,
        crate::handlers::projects::UpdateMemberRoleRequest,
        crate::handlers::bots::CreateBotRequest,
        crate::handlers::bots::UpdateBotRequest,
        crate::handlers::events::EventsQuery,
        crate::handlers::events::EventHintQuery,
        crate::handlers::bot_chats::BotChatQuery,
        crate::handlers::bot_chats::BotChatListResponse,
        crate::handlers::bot_chats::SyncChatsResponse,
        crate::handlers::bot_chats::SyncHistoryResponse,
        crate::handlers::bot_chats::HistoryQuery,
        crate::handlers::max_api::RawProxyRequest,
        crate::handlers::invites::CreateInviteRequest,
        // Media
        crate::handlers::media_proxy::MediaInfoResponse,
        // API Keys
        crate::models::api_key::ApiKeyResponse,
        crate::models::api_key::ApiKeyCreateResponse,
        crate::handlers::api_keys::CreateApiKeyRequest,
        // Gateway
        crate::handlers::gateway::GatewayRequest,
        // Ingestion API
        crate::handlers::ingestion_api::IngestOutgoingRequest,
        crate::handlers::ingestion_api::OutgoingEventPayload,
        // Errors
        crate::errors::ErrorResponse,
        crate::errors::ErrorBody,
        crate::errors::FieldError,
    )),
    security(
        ("bearer" = []),
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::Http::new(
                    utoipa::openapi::security::HttpAuthScheme::Bearer,
                ),
            ),
        );
        components.add_security_scheme(
            "api_key",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::Http::builder()
                    .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                    .description(Some("Bot API key (ak_...)"))
                    .build(),
            ),
        );
    }
}
