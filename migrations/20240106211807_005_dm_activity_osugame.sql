-- Add migration script here
CREATE TABLE dm_activity (
    user_id bigint PRIMARY KEY,
    last_announced bigint,
    until bigint,
    count smallint
);
