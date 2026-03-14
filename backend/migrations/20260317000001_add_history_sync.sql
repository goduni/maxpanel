-- Add history_limit to bots (0 = disabled)
ALTER TABLE bots ADD COLUMN history_limit INT NOT NULL DEFAULT 0;
