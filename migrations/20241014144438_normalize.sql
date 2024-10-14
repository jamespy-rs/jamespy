-- Rename the old tables for backup
ALTER TABLE msgs RENAME TO msgs_old;
ALTER TABLE msgs_edits RENAME TO msgs_edits_old;
ALTER TABLE msgs_deletions RENAME TO msgs_deletions_old;

-- Set guild_id to NULL for specific values
UPDATE msgs_old
SET guild_id = NULL
WHERE guild_id IN (0, 1);

UPDATE msgs_edits_old
SET guild_id = NULL
WHERE guild_id IN (0, 1);

UPDATE msgs_deletions_old
SET guild_id = NULL
WHERE guild_id IN (0, 1);

-- Create new tables
CREATE TABLE guilds (
    guild_id BIGINT PRIMARY KEY
);

CREATE TABLE channels (
    channel_id BIGINT PRIMARY KEY,
    guild_id BIGINT,
    FOREIGN KEY (guild_id) REFERENCES guilds(guild_id)
);

CREATE TABLE users (
    user_id BIGINT PRIMARY KEY
);

CREATE TABLE messages (
    message_id BIGINT PRIMARY KEY,
    channel_id BIGINT,
    guild_id BIGINT,
    user_id BIGINT,
    content TEXT NOT NULL,
    created_at BIGINT,
    FOREIGN KEY (channel_id) REFERENCES channels(channel_id),
    FOREIGN KEY (guild_id) REFERENCES guilds(guild_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

CREATE TABLE attachments (
    attachment_id BIGINT PRIMARY KEY,
    message_id BIGINT,
    file_name TEXT,
    file_size INT,
    file_url TEXT,
    FOREIGN KEY (message_id) REFERENCES messages(message_id) ON DELETE CASCADE
);

CREATE TABLE embeds (
    message_id BIGINT PRIMARY KEY,
    embed_data JSON,
    FOREIGN KEY (message_id) REFERENCES messages(message_id) ON DELETE CASCADE
);

-- Create message edits table
CREATE TABLE message_edits (
    edit_id SERIAL PRIMARY KEY,
    message_id BIGINT,
    channel_id BIGINT,
    guild_id BIGINT,
    user_id BIGINT,
    content TEXT NOT NULL,
    edited_at BIGINT,
    FOREIGN KEY (channel_id) REFERENCES channels(channel_id) ON DELETE CASCADE,
    FOREIGN KEY (guild_id) REFERENCES guilds(guild_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

-- Create a composite index for message edits
CREATE INDEX idx_message_id_edit_id ON message_edits (message_id, edit_id);

-- Create message deletion table
CREATE TABLE message_deletion (
    message_id BIGINT PRIMARY KEY,
    channel_id BIGINT,
    guild_id BIGINT,
    user_id BIGINT,
    content TEXT,
    deleted_at BIGINT,
    FOREIGN KEY (channel_id) REFERENCES channels(channel_id) ON DELETE CASCADE,
    FOREIGN KEY (guild_id) REFERENCES guilds(guild_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

-- Insert Unique Guilds
INSERT INTO guilds (guild_id)
SELECT DISTINCT guild_id FROM msgs_old
WHERE guild_id IS NOT NULL;

-- Insert Unique Channels
INSERT INTO channels (channel_id, guild_id)
SELECT DISTINCT channel_id, guild_id FROM msgs_old
WHERE channel_id IS NOT NULL AND guild_id IS NOT NULL;

-- Insert Unique Users
INSERT INTO users (user_id)
SELECT DISTINCT user_id FROM msgs_old
UNION
SELECT DISTINCT user_id FROM msgs_edits_old
UNION
SELECT DISTINCT user_id FROM msgs_deletions_old;

-- Insert Messages
INSERT INTO messages (message_id, channel_id, user_id, content, created_at)
SELECT
    message_id,
    channel_id,
    user_id,
    COALESCE(content, '') AS content,
    EXTRACT(EPOCH FROM timestamp)
FROM
    msgs_old
WHERE
    channel_id IS NOT NULL
    AND user_id IS NOT NULL
    AND channel_id IN (SELECT channel_id FROM channels)
    AND user_id IN (SELECT user_id FROM users);

-- Insert Unique Message Edits
INSERT INTO message_edits (message_id, user_id, channel_id, content, edited_at)
SELECT
    message_id,
    user_id,
    channel_id,
    COALESCE(new_content, '') AS content,
    EXTRACT(EPOCH FROM timestamp)
FROM
    msgs_edits_old
WHERE
    message_id IN (SELECT message_id FROM messages)
    AND user_id IS NOT NULL
    AND channel_id IS NOT NULL;

-- Insert Unique Message Deletions without duplicates
-- Discord has sent a few of these twice for me, this removes them.
WITH unique_deletions AS (
    SELECT DISTINCT ON (message_id)
        message_id,
        user_id,
        channel_id,
        COALESCE(content, '') AS content,
        EXTRACT(EPOCH FROM timestamp) AS deleted_at
    FROM
        msgs_deletions_old
    WHERE
        message_id IN (SELECT message_id FROM messages)
        AND user_id IS NOT NULL
        AND channel_id IS NOT NULL
    ORDER BY message_id, deleted_at DESC
)
INSERT INTO message_deletion (message_id, user_id, channel_id, content, deleted_at)
SELECT
    message_id,
    user_id,
    channel_id,
    content,
    deleted_at
FROM
    unique_deletions;
