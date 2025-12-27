
CREATE TABLE IF NOT EXISTS user_subscriptions (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    anime_slug VARCHAR(500) NOT NULL,
    anime_title VARCHAR(500) NOT NULL,
    thumbnail VARCHAR(1000),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_user_subscriptions_user_id
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,
    CONSTRAINT user_subscriptions_user_anime_unique
        UNIQUE(user_id, anime_slug)
);

CREATE INDEX IF NOT EXISTS idx_user_subscriptions_slug ON user_subscriptions(anime_slug);
CREATE INDEX IF NOT EXISTS idx_user_subscriptions_user ON user_subscriptions(user_id);
