-- droping the old foreign key constraint.
ALTER TABLE emote_usage
    DROP CONSTRAINT IF EXISTS emote_usage_emote_id_fkey;

-- set all the new data.
UPDATE emote_usage
SET emote_id = e.id
FROM emotes e
WHERE emote_usage.emote_id = e.discord_id;

-- readd the constraint, all new and set to the right column.
ALTER TABLE emote_usage
    ADD CONSTRAINT emote_usage_emote_id_fkey
    FOREIGN KEY (emote_id) REFERENCES emotes(id);


ALTER TABLE emote_usage
    DROP CONSTRAINT IF EXISTS emote_usage_emote_id_fkey;

ALTER TABLE emotes
    ALTER COLUMN discord_id DROP NOT NULL;

ALTER TABLE emote_usage
    ALTER COLUMN emote_id DROP NOT NULL;

CREATE INDEX idx_emote_name_discord_id ON emotes(emote_name, discord_id);
CREATE INDEX idx_message_id_emote ON emote_usage(message_id);

-- Composite unique index for (emote_name, discord_id)
CREATE UNIQUE INDEX emote_name_discord_id_unique
ON emotes (emote_name, discord_id);

-- Partial unique index for when discord_id is NULL
CREATE UNIQUE INDEX emote_name_null_discord_id_unique
ON emotes (emote_name)
WHERE discord_id IS NULL;

