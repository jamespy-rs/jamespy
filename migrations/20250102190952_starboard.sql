CREATE TYPE starboard_status AS ENUM ('InReview', 'Denied', 'Accepted');

CREATE TABLE starboard (
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    username VARCHAR(32) NOT NULL,
    avatar_url TEXT,
    content TEXT NOT NULL,
    channel_id BIGINT NOT NULL,
    message_id BIGINT NOT NULL,
    attachment_urls TEXT[] NOT NULL,
    star_count SMALLINT NOT NULL,
    starboard_status starboard_status DEFAULT 'InReview' NOT NULL,
    starboard_message_id BIGINT NOT NULL,
    starboard_message_channel BIGINT NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(user_id),
    FOREIGN KEY (channel_id) REFERENCES channels(channel_id),
    FOREIGN KEY (starboard_message_channel) REFERENCES channels(channel_id)
);
