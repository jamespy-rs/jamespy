CREATE TABLE owner_access (
    user_id BIGINT NOT NULL,
    command_name TEXT,
    CONSTRAINT unique_user_command UNIQUE (user_id, command_name),
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);


CREATE TABLE banned_users(
    user_id BIGINT PRIMARY KEY,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);
