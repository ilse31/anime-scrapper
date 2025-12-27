
CREATE TABLE IF NOT EXISTS crawled_anime (
    id SERIAL PRIMARY KEY,
    slug VARCHAR(500) UNIQUE NOT NULL,
    title VARCHAR(500) NOT NULL,
    url VARCHAR(1000) UNIQUE NOT NULL,
    thumbnail VARCHAR(1000),
    status VARCHAR(50),
    type VARCHAR(50),
    episode_status VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);


CREATE INDEX IF NOT EXISTS idx_crawled_anime_slug ON crawled_anime(slug);
CREATE INDEX IF NOT EXISTS idx_crawled_anime_status ON crawled_anime(status);
CREATE INDEX IF NOT EXISTS idx_crawled_anime_type ON crawled_anime(type);
