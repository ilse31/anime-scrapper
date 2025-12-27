
CREATE TABLE IF NOT EXISTS anime_details (
    id SERIAL PRIMARY KEY,
    slug VARCHAR(500) UNIQUE NOT NULL,
    title VARCHAR(500) NOT NULL,
    alternate_titles TEXT,
    poster VARCHAR(1000),
    rating VARCHAR(20),
    trailer_url VARCHAR(1000),
    status VARCHAR(50),
    studio VARCHAR(200),
    release_date VARCHAR(100),
    duration VARCHAR(50),
    season VARCHAR(100),
    type VARCHAR(50),
    total_episodes VARCHAR(50),
    director VARCHAR(200),
    casts TEXT[],
    genres TEXT[],
    synopsis TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_anime_details_slug ON anime_details(slug);
