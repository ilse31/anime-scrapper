
CREATE TABLE IF NOT EXISTS user_history (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    episode_slug VARCHAR(500) NOT NULL,
    anime_slug VARCHAR(500) NOT NULL,
    episode_title VARCHAR(500),
    anime_title VARCHAR(500),
    thumbnail VARCHAR(1000),
    watched_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_user_history_user_id
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,
    CONSTRAINT user_history_user_episode_unique
        UNIQUE(user_id, episode_slug)
);

CREATE INDEX IF NOT EXISTS idx_user_history_episode ON user_history(episode_slug);
CREATE INDEX IF NOT EXISTS idx_user_history_user ON user_history(user_id);
CREATE INDEX IF NOT EXISTS idx_user_history_watched ON user_history(watched_at DESC);
