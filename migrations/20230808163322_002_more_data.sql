-- 002_more_data.sql
-- sqlx-migrate:up
CREATE TABLE join_tracks (
    guild_id bigint,
    author_id bigint,
    user_id bigint
);

CREATE TABLE snippets (
    guild_id bigint,
    name VARCHAR(32) NOT NULL,
    title text,
    description text,
    image text,
    thumbnail text,
    color text
);
