-- Bot chats: stores chat/channel/dialog metadata discovered via events or Max API sync.
CREATE TABLE bot_chats (
    bot_id        UUID        NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    chat_id       BIGINT      NOT NULL,
    chat_type     TEXT,
    title         TEXT,
    icon_url      TEXT,
    participants  INT,
    last_event_at TIMESTAMPTZ,
    synced_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (bot_id, chat_id)
);

CREATE INDEX idx_bot_chats_last_event ON bot_chats (bot_id, last_event_at DESC NULLS LAST);
