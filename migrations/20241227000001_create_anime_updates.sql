
CREATE TABLE IF NOT EXISTS anime_updates (
    id SERIAL PRIMARY KEY,
    title VARCHAR(500) NOT NULL,
    episode_url VARCHAR(1000) UNIQUE NOT NULL,
    thumbnail VARCHAR(1000),
    episode_number VARCHAR(50),
    type VARCHAR(50),
    series_title VARCHAR(500),
    series_url VARCHAR(1000),
    status VARCHAR(50),
    release_info VARCHAR(200),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);


CREATE INDEX IF NOT EXISTS idx_anime_updates_episode_url ON anime_updates(episode_url);
