-- Functional index to support ORDER BY COALESCE(last_event_at, synced_at) in bot_chats queries.
CREATE INDEX idx_bot_chats_coalesced_ts
    ON bot_chats (bot_id, COALESCE(last_event_at, synced_at) DESC, chat_id DESC);
