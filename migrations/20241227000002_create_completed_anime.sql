
CREATE TABLE IF NOT EXISTS completed_anime (
    id SERIAL PRIMARY KEY,
    title VARCHAR(500) NOT NULL,
    url VARCHAR(1000) UNIQUE NOT NULL,
    thumbnail VARCHAR(1000),
    type VARCHAR(50),
    episode_count VARCHAR(50),
    status VARCHAR(50),
    posted_by VARCHAR(100),
    posted_at VARCHAR(100),
    series_title VARCHAR(500),
    series_url VARCHAR(1000),
    genres TEXT[],
    rating VARCHAR(20),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_completed_anime_url ON completed_anime(url);
