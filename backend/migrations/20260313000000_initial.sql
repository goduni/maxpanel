-- Custom types
CREATE TYPE org_role AS ENUM ('owner', 'admin', 'member');
CREATE TYPE project_role AS ENUM ('admin', 'editor', 'viewer');
CREATE TYPE event_mode AS ENUM ('webhook', 'polling');

-- Automatic updated_at trigger function
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Users
CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name          TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Organizations
CREATE TABLE organizations (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL,
    slug       TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER organizations_updated_at
    BEFORE UPDATE ON organizations
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE organization_members (
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role            org_role NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (organization_id, user_id)
);

-- Invites
CREATE TABLE invites (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    email           TEXT NOT NULL,
    role            org_role NOT NULL CHECK (role IN ('admin', 'member')),
    token_hash      TEXT NOT NULL UNIQUE,
    invited_by      UUID NOT NULL REFERENCES users(id),
    expires_at      TIMESTAMPTZ NOT NULL,
    accepted_at     TIMESTAMPTZ,
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_invites_org ON invites (organization_id);
CREATE INDEX idx_invites_email ON invites (email);

-- Projects
CREATE TABLE projects (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    slug            TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (organization_id, slug)
);

CREATE INDEX idx_projects_org ON projects (organization_id);

CREATE TRIGGER projects_updated_at
    BEFORE UPDATE ON projects
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE project_members (
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role       project_role NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, user_id)
);

-- Bots
CREATE TABLE bots (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id         UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name               TEXT NOT NULL,
    access_token_enc   BYTEA NOT NULL,
    access_token_nonce BYTEA NOT NULL,
    key_version        INTEGER NOT NULL DEFAULT 1,
    event_mode         event_mode NOT NULL,
    webhook_secret     UUID UNIQUE,
    webhook_url        TEXT,
    polling_marker     BIGINT,
    is_active          BOOLEAN NOT NULL DEFAULT true,
    max_bot_id         BIGINT,
    max_bot_info       JSONB,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_bots_project ON bots (project_id);
CREATE INDEX idx_bots_active_polling ON bots (is_active, event_mode)
    WHERE is_active = true;

CREATE TRIGGER bots_updated_at
    BEFORE UPDATE ON bots
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Events (partitioned)
CREATE TABLE events (
    id            UUID NOT NULL DEFAULT gen_random_uuid(),
    bot_id        UUID NOT NULL,
    max_update_id BIGINT,
    update_type   TEXT NOT NULL,
    chat_id       BIGINT,
    sender_id     BIGINT,
    timestamp     BIGINT NOT NULL,
    raw_payload   JSONB NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

CREATE INDEX idx_events_bot_created ON events (bot_id, created_at DESC, id DESC);
CREATE INDEX idx_events_bot_chat    ON events (bot_id, chat_id, created_at DESC, id DESC);
CREATE INDEX idx_events_bot_type    ON events (bot_id, update_type, created_at DESC, id DESC);

-- Deduplication index
CREATE UNIQUE INDEX idx_events_dedup ON events (bot_id, max_update_id, created_at)
    WHERE max_update_id IS NOT NULL;

-- Default partition catches all data; the partition_manager worker
-- automatically creates monthly partitions at server startup.
CREATE TABLE events_default PARTITION OF events DEFAULT;

-- Refresh tokens
CREATE TABLE refresh_tokens (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash      TEXT NOT NULL UNIQUE,
    family_id       UUID NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_refresh_tokens_user ON refresh_tokens (user_id);
CREATE INDEX idx_refresh_tokens_family ON refresh_tokens (family_id);
