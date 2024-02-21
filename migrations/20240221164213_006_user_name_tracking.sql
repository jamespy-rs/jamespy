-- sqlx-migrate:up
CREATE TABLE usernames (
    user_id bigint NOT NULL,
    username text NOT NULL,
    timestamp timestamp NOT NULL
);

CREATE TABLE global_names (
    user_id bigint NOT NULL,
    global_name text NOT NULL,
    timestamp timestamp NOT NULL
);

CREATE TABLE nicknames (
    guild_id bigint NOT NULL,
    user_id bigint NOT NULL,
    nickname text  NOT NULL,
    timestamp timestamp NOT NULL
);
