
CREATE TABLE IF NOT EXISTS video_sources (
    id SERIAL PRIMARY KEY,
    episode_url VARCHAR(1000) NOT NULL,
    server VARCHAR(100),
    quality VARCHAR(20),
    url VARCHAR(2000),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_video_sources_episode_url ON video_sources(episode_url);
