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


CREATE TABLE owner_commands(
    user_id BIGINT NOT NULL,
    -- No command name = global
    command_name TEXT,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
)

CREATE TABLE events(
    no_log_channels JSON,
    no_log_users JSON,
    regex JSON,
    guild_name_override JSON
)

CREATE TABLE banned_users(
    user_id BIGINT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
)
