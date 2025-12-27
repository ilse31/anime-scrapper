
CREATE TABLE IF NOT EXISTS episodes (
    id SERIAL PRIMARY KEY,
    anime_slug VARCHAR(500) NOT NULL,
    number VARCHAR(20),
    title VARCHAR(500),
    url VARCHAR(1000) UNIQUE NOT NULL,
    release_date VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_episodes_anime_slug
        FOREIGN KEY (anime_slug)
        REFERENCES anime_details(slug)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_episodes_anime_slug ON episodes(anime_slug);
