-- Add direction and source to events table
ALTER TABLE events ADD COLUMN direction TEXT NOT NULL DEFAULT 'inbound';
ALTER TABLE events ADD COLUMN source TEXT NOT NULL DEFAULT 'webhook';

-- Index for filtering by direction in unified timeline
CREATE INDEX idx_events_bot_direction
  ON events (bot_id, direction, created_at DESC, id DESC);

-- API keys for bot M2M authentication (gateway + ingestion API)
CREATE TABLE bot_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    key_hash TEXT NOT NULL,
    key_prefix TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX idx_bot_api_keys_bot_id ON bot_api_keys (bot_id);
