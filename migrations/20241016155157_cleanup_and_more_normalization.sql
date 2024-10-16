-- I made a mistake and accidentally added empty fields.
DELETE FROM embeds
WHERE embed_data::text = '[]';

-- I forgot to set this originally.
ALTER TABLE embeds
ALTER COLUMN embed_data SET NOT NULL;

-- Remove obselete tables.
DROP TABLE msgs_old;
DROP TABLE msgs_edits_old;
DROP TABLE msgs_deletions_old;

INSERT INTO users (user_id)
SELECT DISTINCT user_id FROM usernames
UNION
SELECT DISTINCT user_id FROM global_names
UNION
SELECT DISTINCT user_id FROM nicknames
ON CONFLICT (user_id) DO NOTHING;

INSERT INTO guilds (guild_id)
SELECT DISTINCT guild_id FROM nicknames
ON CONFLICT (guild_id) DO NOTHING;

ALTER TABLE usernames
ADD CONSTRAINT fk_usernames_user_id
FOREIGN KEY (user_id)
REFERENCES users (user_id);

ALTER TABLE global_names
ADD CONSTRAINT fk_global_names_user_id
FOREIGN KEY (user_id)
REFERENCES users (user_id);

ALTER TABLE nicknames
ADD CONSTRAINT fk_nicknames_user_id
FOREIGN KEY (user_id)
REFERENCES users (user_id),
ADD CONSTRAINT fk_nicknames_guild_id
FOREIGN KEY (guild_id)
REFERENCES guilds (guild_id);
