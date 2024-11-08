-- I swear I'm an idiot.
ALTER TABLE messages
  ALTER COLUMN message_id SET NOT NULL,
  ALTER COLUMN channel_id SET NOT NULL,
  ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE message_edits
  ALTER COLUMN message_id SET NOT NULL,
  ALTER COLUMN channel_id SET NOT NULL,
  ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE message_deletion
  ALTER COLUMN message_id SET NOT NULL,
  ALTER COLUMN channel_id SET NOT NULL,
  ALTER COLUMN user_id SET NOT NULL;

CREATE TABLE stickers(
    sticker_id BIGINT PRIMARY KEY,
    sticker_name TEXT NOT NULL
);

CREATE TABLE sticker_usage(
    id SERIAL PRIMARY KEY,
    message_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    sticker_id BIGINT NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(message_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    FOREIGN KEY (sticker_id) REFERENCES stickers(sticker_id),
    FOREIGN KEY (channel_id) REFERENCES channels(channel_id),
    FOREIGN KEY (guild_id) REFERENCES guilds(guild_id)
);

CREATE INDEX idx_user_id ON sticker_usage(user_id);
CREATE INDEX idx_guild_id ON sticker_usage(guild_id);

CREATE TABLE emotes(
    id SERIAL PRIMARY KEY,
    emote_name TEXT NOT NULL,
    discord_id BIGINT UNIQUE NOT NULL
);

CREATE INDEX idx_discord_id ON emotes(discord_id);

CREATE TYPE EmoteUsageType AS ENUM ('Message', 'ReactionAdd', 'ReactionRemove');
CREATE TABLE emote_usage(
    id SERIAL PRIMARY KEY,
    -- No foreign key beacuse I might not have gotten the message.
    message_id BIGINT NOT NULL,
    emote_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    used_at BIGINT NOT NULL,
    usage_type EmoteUsageType NOT NULL,
    FOREIGN KEY (emote_id) REFERENCES emotes(discord_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    FOREIGN KEY (channel_id) REFERENCES channels(channel_id),
    FOREIGN KEY (guild_id) REFERENCES guilds(guild_id)
);

CREATE INDEX idx_user_id_emote ON emote_usage(user_id);
CREATE INDEX idx_guild_id_emote ON emote_usage(guild_id);
