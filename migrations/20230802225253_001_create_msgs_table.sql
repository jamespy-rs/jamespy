-- 001_create_msgs_table.sql
-- sqlx-migrate:up
CREATE TABLE msgs (
    guild_id bigint,
    channel_id bigint,
    message_id bigint,
    user_id bigint,
    content text,
    attachments text,
    timestamp timestamp
);
