
CREATE TABLE IF NOT EXISTS user_favorites (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    anime_slug VARCHAR(500) NOT NULL,
    anime_title VARCHAR(500) NOT NULL,
    thumbnail VARCHAR(1000),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_user_favorites_user_id
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,
    CONSTRAINT user_favorites_user_anime_unique
        UNIQUE(user_id, anime_slug)
);

CREATE INDEX IF NOT EXISTS idx_user_favorites_slug ON user_favorites(anime_slug);
CREATE INDEX IF NOT EXISTS idx_user_favorites_user ON user_favorites(user_id);
