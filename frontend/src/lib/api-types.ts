// === Auth ===

export interface User {
  id: string;
  email: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface AuthTokens {
  access_token: string;
  refresh_token: string;
}

export interface LoginResponse {
  user: User;
  tokens: AuthTokens;
}

export interface RegisterRequest {
  email: string;
  password: string;
  name: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RefreshRequest {
  refresh_token: string;
}

export interface LogoutRequest {
  refresh_token: string;
}

export interface ChangePasswordRequest {
  current_password: string;
  new_password: string;
}

export interface UpdateMeRequest {
  name: string;
}

// === Organizations ===

export interface Organization {
  id: string;
  name: string;
  slug: string;
  created_at: string;
  updated_at: string;
}

export interface CreateOrgRequest {
  name: string;
  slug: string;
}

export interface UpdateOrgRequest {
  name: string;
}

export interface TransferOwnershipRequest {
  new_owner_id: string;
}

export type OrgRole = "owner" | "admin" | "member";

export interface OrganizationMember {
  organization_id: string;
  user_id: string;
  user_name: string | null;
  user_email: string | null;
  role: OrgRole;
  created_at: string;
}

// === Projects ===

export interface Project {
  id: string;
  organization_id: string;
  name: string;
  slug: string;
  created_at: string;
  updated_at: string;
}

export interface CreateProjectRequest {
  name: string;
  slug: string;
}

export interface UpdateProjectRequest {
  name: string;
}

export type ProjectRole = "admin" | "editor" | "viewer";

export interface ProjectMember {
  project_id: string;
  user_id: string;
  role: ProjectRole;
  created_at: string;
}

export interface AddProjectMemberRequest {
  user_id: string;
  role: ProjectRole;
}

export interface UpdateMemberRoleRequest {
  role: OrgRole | ProjectRole;
}

// === Bots ===

export interface Bot {
  id: string;
  project_id: string;
  name: string;
  event_mode: "webhook" | "polling";
  is_active: boolean;
  history_limit: number;
  max_bot_id: number | null;
  max_bot_info: Record<string, unknown> | null;
  created_at: string;
  updated_at: string;
}

export interface CreateBotRequest {
  name: string;
  access_token: string;
  event_mode: "webhook" | "polling";
}

export interface UpdateBotRequest {
  name?: string;
  history_limit?: number;
}

// === Events ===

export interface BotEvent {
  id: string;
  bot_id: string;
  max_update_id: number | null;
  update_type: string;
  chat_id: number | null;
  sender_id: number | null;
  timestamp: number;
  raw_payload: unknown; // Can be any valid JSON (object, array, etc.)
  created_at: string;
  direction?: "inbound" | "outbound";
  source?: "webhook" | "polling" | "proxy" | "gateway" | "ingestion_api" | "history_sync" | "history";
}

export interface ApiKey {
  id: string;
  name: string;
  key_prefix: string;
  created_at: string;
  last_used_at: string | null;
  is_active: boolean;
}

export interface ApiKeyCreateResponse {
  id: string;
  name: string;
  key: string;
  key_prefix: string;
  created_at: string;
}

export interface CreateApiKeyRequest {
  name: string;
}

export interface BotChat {
  bot_id: string;
  chat_id: number;
  chat_type: string | null;
  title: string | null;
  icon_url: string | null;
  participants: number | null;
  last_event_at: string | null;
  synced_at: string;
}

// === Invites ===

export interface Invite {
  id: string;
  email: string;
  role: OrgRole;
  invited_by: string;
  expires_at: string;
  created_at: string;
}

export interface CreateInviteRequest {
  email: string;
  role: "admin" | "member";
}

export interface CreateInviteResponse {
  invite: Invite;
  token: string;
}

// === Max API Proxy ===

export interface RawProxyRequest {
  method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  path: string;
  body?: Record<string, unknown>;
}

// === Common ===

export interface OkResponse {
  ok: boolean;
}

export interface PaginationParams {
  limit?: number;
  offset?: number;
}

export interface CursorParams {
  cursor?: string;
  limit?: number;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    total: number;
    offset: number;
    limit: number;
  };
}

export interface CursorPaginatedResponse<T> {
  data: T[];
  pagination: {
    next_cursor: string | null;
    has_more: boolean;
  };
}

export interface ApiErrorResponse {
  error: {
    code: string;
    message: string;
    details?: { field: string; message: string }[];
    upstream?: Record<string, unknown>;
  };
}
