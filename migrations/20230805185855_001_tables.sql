-- 001_tables.sql
-- sqlx-migrate:up
CREATE TABLE msgs (
    guild_id bigint,
    channel_id bigint,
    message_id bigint,
    user_id bigint,
    content text,
    timestamp timestamp,
    attachments text,
    embeds text
);

CREATE TABLE msgs_edits (
    guild_id bigint,
    channel_id bigint,
    message_id bigint,
    user_id bigint,
    old_content text,
    new_content text,
    timestamp timestamp,
    attachments text,
    embeds text
);

CREATE TABLE msgs_deletions (
    guild_id bigint,
    channel_id bigint,
    message_id bigint,
    user_id bigint,
    content text,
    timestamp timestamp,
    attachments text,
    embeds text
);
