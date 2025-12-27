//! Repository module for anime data persistence
//!
//! Provides CRUD operations with upsert logic for anime_updates, completed_anime,
//! anime_details, episodes, video_sources, and crawled_anime tables.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use thiserror::Error;

use crate::models::{CrawledAnime, CrawledAnimeRecord};
use crate::parser::{AnimeDetail, AnimeUpdate, CompletedAnime, Episode, VideoSource};

/// Repository-related errors
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Record not found: {0}")]
    NotFound(String),
}

/// Result type for repository operations
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Extract slug from a URL
/// 
/// Takes a URL like "https://x3.sokuja.uk/anime/one-piece-subtitle-indonesia/"
/// and returns "one-piece-subtitle-indonesia"
fn extract_slug_from_url(url: &str) -> String {
    url.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string()
}

// ============================================================================
// Anime Updates Repository
// ============================================================================

/// Save anime updates to the database with upsert logic
///
/// Uses ON CONFLICT UPDATE to update existing records based on episode_url
pub async fn save_anime_updates(
    pool: &PgPool,
    updates: &[AnimeUpdate],
) -> RepositoryResult<()> {
    for update in updates {
        sqlx::query(
            r#"
            INSERT INTO anime_updates (
                title, episode_url, thumbnail, episode_number, type,
                series_title, series_url, status, release_info, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_TIMESTAMP)
            ON CONFLICT (episode_url) DO UPDATE SET
                title = EXCLUDED.title,
                thumbnail = EXCLUDED.thumbnail,
                episode_number = EXCLUDED.episode_number,
                type = EXCLUDED.type,
                series_title = EXCLUDED.series_title,
                series_url = EXCLUDED.series_url,
                status = EXCLUDED.status,
                release_info = EXCLUDED.release_info,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&update.title)
        .bind(&update.episode_url)
        .bind(&update.thumbnail)
        .bind(&update.episode_number)
        .bind(&update.anime_type)
        .bind(&update.series_title)
        .bind(&update.series_url)
        .bind(&update.status)
        .bind(&update.release_info)
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Get all anime updates from the database
pub async fn get_anime_updates(pool: &PgPool) -> RepositoryResult<Vec<AnimeUpdate>> {
    let rows = sqlx::query(
        r#"
        SELECT title, episode_url, thumbnail, episode_number, type,
               series_title, series_url, status, release_info
        FROM anime_updates
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let updates = rows
        .into_iter()
        .map(|row| {
            let series_url: String = row.get::<Option<String>, _>("series_url").unwrap_or_default();
            AnimeUpdate {
                slug: extract_slug_from_url(&series_url),
                title: row.get::<String, _>("title"),
                episode_url: row.get::<String, _>("episode_url"),
                thumbnail: row.get::<Option<String>, _>("thumbnail").unwrap_or_default(),
                episode_number: row.get::<Option<String>, _>("episode_number").unwrap_or_default(),
                anime_type: row.get::<Option<String>, _>("type").unwrap_or_default(),
                series_title: row.get::<Option<String>, _>("series_title").unwrap_or_default(),
                series_url,
                status: row.get::<Option<String>, _>("status").unwrap_or_default(),
                release_info: row.get::<Option<String>, _>("release_info").unwrap_or_default(),
            }
        })
        .collect();

    Ok(updates)
}

/// Delete all anime updates from the database
pub async fn delete_all_anime_updates(pool: &PgPool) -> RepositoryResult<u64> {
    let result = sqlx::query("DELETE FROM anime_updates")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

// ============================================================================
// Completed Anime Repository
// ============================================================================

/// Save completed anime to the database with upsert logic
///
/// Uses ON CONFLICT UPDATE to update existing records based on url
pub async fn save_completed_anime(
    pool: &PgPool,
    anime_list: &[CompletedAnime],
) -> RepositoryResult<()> {
    for anime in anime_list {
        sqlx::query(
            r#"
            INSERT INTO completed_anime (
                title, url, thumbnail, type, episode_count, status,
                posted_by, posted_at, series_title, series_url, genres, rating, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, CURRENT_TIMESTAMP)
            ON CONFLICT (url) DO UPDATE SET
                title = EXCLUDED.title,
                thumbnail = EXCLUDED.thumbnail,
                type = EXCLUDED.type,
                episode_count = EXCLUDED.episode_count,
                status = EXCLUDED.status,
                posted_by = EXCLUDED.posted_by,
                posted_at = EXCLUDED.posted_at,
                series_title = EXCLUDED.series_title,
                series_url = EXCLUDED.series_url,
                genres = EXCLUDED.genres,
                rating = EXCLUDED.rating,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&anime.title)
        .bind(&anime.url)
        .bind(&anime.thumbnail)
        .bind(&anime.anime_type)
        .bind(&anime.episode_count)
        .bind(&anime.status)
        .bind(&anime.posted_by)
        .bind(&anime.posted_at)
        .bind(&anime.series_title)
        .bind(&anime.series_url)
        .bind(&anime.genres)
        .bind(&anime.rating)
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Get all completed anime from the database
pub async fn get_completed_anime(pool: &PgPool) -> RepositoryResult<Vec<CompletedAnime>> {
    let rows = sqlx::query(
        r#"
        SELECT title, url, thumbnail, type, episode_count, status,
               posted_by, posted_at, series_title, series_url, genres, rating
        FROM completed_anime
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let anime_list = rows
        .into_iter()
        .map(|row| {
            let url: String = row.get::<String, _>("url");
            CompletedAnime {
                slug: extract_slug_from_url(&url),
                title: row.get::<String, _>("title"),
                url,
                thumbnail: row.get::<Option<String>, _>("thumbnail").unwrap_or_default(),
                anime_type: row.get::<Option<String>, _>("type").unwrap_or_default(),
                episode_count: row.get::<Option<String>, _>("episode_count").unwrap_or_default(),
                status: row.get::<Option<String>, _>("status").unwrap_or_default(),
                posted_by: row.get::<Option<String>, _>("posted_by").unwrap_or_default(),
                posted_at: row.get::<Option<String>, _>("posted_at").unwrap_or_default(),
                series_title: row.get::<Option<String>, _>("series_title").unwrap_or_default(),
                series_url: row.get::<Option<String>, _>("series_url").unwrap_or_default(),
                genres: row.get::<Option<Vec<String>>, _>("genres").unwrap_or_default(),
                rating: row.get::<Option<String>, _>("rating").unwrap_or_default(),
            }
        })
        .collect();

    Ok(anime_list)
}

/// Delete all completed anime from the database
pub async fn delete_all_completed_anime(pool: &PgPool) -> RepositoryResult<u64> {
    let result = sqlx::query("DELETE FROM completed_anime")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

// ============================================================================
// Anime Details Repository
// ============================================================================

/// Save anime detail to the database with upsert logic
///
/// Uses ON CONFLICT UPDATE to update existing records based on slug.
/// Note: Episodes are saved separately using save_episodes.
pub async fn save_anime_detail(
    pool: &PgPool,
    slug: &str,
    detail: &AnimeDetail,
) -> RepositoryResult<()> {
    sqlx::query(
        r#"
        INSERT INTO anime_details (
            slug, title, alternate_titles, poster, rating, trailer_url,
            status, studio, release_date, duration, season, type,
            total_episodes, director, casts, genres, synopsis, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, CURRENT_TIMESTAMP)
        ON CONFLICT (slug) DO UPDATE SET
            title = EXCLUDED.title,
            alternate_titles = EXCLUDED.alternate_titles,
            poster = EXCLUDED.poster,
            rating = EXCLUDED.rating,
            trailer_url = EXCLUDED.trailer_url,
            status = EXCLUDED.status,
            studio = EXCLUDED.studio,
            release_date = EXCLUDED.release_date,
            duration = EXCLUDED.duration,
            season = EXCLUDED.season,
            type = EXCLUDED.type,
            total_episodes = EXCLUDED.total_episodes,
            director = EXCLUDED.director,
            casts = EXCLUDED.casts,
            genres = EXCLUDED.genres,
            synopsis = EXCLUDED.synopsis,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(slug)
    .bind(&detail.title)
    .bind(&detail.alternate_titles)
    .bind(&detail.poster)
    .bind(&detail.rating)
    .bind(&detail.trailer_url)
    .bind(&detail.status)
    .bind(&detail.studio)
    .bind(&detail.release_date)
    .bind(&detail.duration)
    .bind(&detail.season)
    .bind(&detail.anime_type)
    .bind(&detail.total_episodes)
    .bind(&detail.director)
    .bind(&detail.casts)
    .bind(&detail.genres)
    .bind(&detail.synopsis)
    .execute(pool)
    .await?;

    Ok(())
}


/// Get anime detail by slug from the database
///
/// Returns None if the anime is not found
pub async fn get_anime_detail(pool: &PgPool, slug: &str) -> RepositoryResult<Option<AnimeDetail>> {
    let row = sqlx::query(
        r#"
        SELECT slug, title, alternate_titles, poster, rating, trailer_url,
               status, studio, release_date, duration, season, type,
               total_episodes, director, casts, genres, synopsis
        FROM anime_details
        WHERE slug = $1
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            // Fetch episodes for this anime
            let episodes = get_episodes(pool, slug).await?;

            Ok(Some(AnimeDetail {
                title: row.get::<String, _>("title"),
                alternate_titles: row.get::<Option<String>, _>("alternate_titles").unwrap_or_default(),
                poster: row.get::<Option<String>, _>("poster").unwrap_or_default(),
                rating: row.get::<Option<String>, _>("rating").unwrap_or_default(),
                trailer_url: row.get::<Option<String>, _>("trailer_url").unwrap_or_default(),
                status: row.get::<Option<String>, _>("status").unwrap_or_default(),
                studio: row.get::<Option<String>, _>("studio").unwrap_or_default(),
                release_date: row.get::<Option<String>, _>("release_date").unwrap_or_default(),
                duration: row.get::<Option<String>, _>("duration").unwrap_or_default(),
                season: row.get::<Option<String>, _>("season").unwrap_or_default(),
                anime_type: row.get::<Option<String>, _>("type").unwrap_or_default(),
                total_episodes: row.get::<Option<String>, _>("total_episodes").unwrap_or_default(),
                director: row.get::<Option<String>, _>("director").unwrap_or_default(),
                casts: row.get::<Option<Vec<String>>, _>("casts").unwrap_or_default(),
                genres: row.get::<Option<Vec<String>>, _>("genres").unwrap_or_default(),
                synopsis: row.get::<Option<String>, _>("synopsis").unwrap_or_default(),
                episodes,
            }))
        }
        None => Ok(None),
    }
}

/// Delete anime detail by slug from the database
///
/// This will also cascade delete associated episodes due to foreign key constraint
pub async fn delete_anime_detail(pool: &PgPool, slug: &str) -> RepositoryResult<bool> {
    let result = sqlx::query("DELETE FROM anime_details WHERE slug = $1")
        .bind(slug)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ============================================================================
// Episodes Repository
// ============================================================================

/// Save episodes for an anime to the database with upsert logic
///
/// Uses ON CONFLICT UPDATE to update existing records based on url
pub async fn save_episodes(
    pool: &PgPool,
    anime_slug: &str,
    episodes: &[Episode],
) -> RepositoryResult<()> {
    for episode in episodes {
        sqlx::query(
            r#"
            INSERT INTO episodes (anime_slug, number, title, url, release_date, updated_at)
            VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)
            ON CONFLICT (url) DO UPDATE SET
                anime_slug = EXCLUDED.anime_slug,
                number = EXCLUDED.number,
                title = EXCLUDED.title,
                release_date = EXCLUDED.release_date,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(anime_slug)
        .bind(&episode.number)
        .bind(&episode.title)
        .bind(&episode.url)
        .bind(&episode.release_date)
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Get all episodes for an anime by slug
pub async fn get_episodes(pool: &PgPool, anime_slug: &str) -> RepositoryResult<Vec<Episode>> {
    let rows = sqlx::query(
        r#"
        SELECT number, title, url, release_date
        FROM episodes
        WHERE anime_slug = $1
        ORDER BY id ASC
        "#,
    )
    .bind(anime_slug)
    .fetch_all(pool)
    .await?;

    let episodes = rows
        .into_iter()
        .map(|row| {
            let url: String = row.get::<String, _>("url");
            Episode {
                slug: extract_slug_from_url(&url),
                number: row.get::<Option<String>, _>("number").unwrap_or_default(),
                title: row.get::<Option<String>, _>("title").unwrap_or_default(),
                url,
                release_date: row.get::<Option<String>, _>("release_date").unwrap_or_default(),
            }
        })
        .collect();

    Ok(episodes)
}

/// Delete all episodes for an anime by slug
pub async fn delete_episodes_by_anime(pool: &PgPool, anime_slug: &str) -> RepositoryResult<u64> {
    let result = sqlx::query("DELETE FROM episodes WHERE anime_slug = $1")
        .bind(anime_slug)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

// ============================================================================
// Video Sources Repository
// ============================================================================

/// Save video sources for an episode to the database
///
/// First deletes existing sources for the episode, then inserts new ones
pub async fn save_video_sources(
    pool: &PgPool,
    episode_url: &str,
    sources: &[VideoSource],
) -> RepositoryResult<()> {
    // Delete existing sources for this episode
    sqlx::query("DELETE FROM video_sources WHERE episode_url = $1")
        .bind(episode_url)
        .execute(pool)
        .await?;

    // Insert new sources
    for source in sources {
        sqlx::query(
            r#"
            INSERT INTO video_sources (episode_url, server, quality, url, updated_at)
            VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(episode_url)
        .bind(&source.server)
        .bind(&source.quality)
        .bind(&source.url)
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Get all video sources for an episode by URL
pub async fn get_video_sources(pool: &PgPool, episode_url: &str) -> RepositoryResult<Vec<VideoSource>> {
    let rows = sqlx::query(
        r#"
        SELECT server, quality, url
        FROM video_sources
        WHERE episode_url = $1
        ORDER BY id ASC
        "#,
    )
    .bind(episode_url)
    .fetch_all(pool)
    .await?;

    let sources = rows
        .into_iter()
        .map(|row| VideoSource {
            server: row.get::<Option<String>, _>("server").unwrap_or_default(),
            quality: row.get::<Option<String>, _>("quality").unwrap_or_default(),
            url: row.get::<Option<String>, _>("url").unwrap_or_default(),
        })
        .collect();

    Ok(sources)
}

/// Delete all video sources for an episode by URL
pub async fn delete_video_sources(pool: &PgPool, episode_url: &str) -> RepositoryResult<u64> {
    let result = sqlx::query("DELETE FROM video_sources WHERE episode_url = $1")
        .bind(episode_url)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}


// ============================================================================
// Batch Operations
// ============================================================================

/// Save anime detail with its episodes in a single transaction
///
/// This ensures atomicity - either both anime detail and episodes are saved, or neither
pub async fn save_anime_detail_with_episodes(
    pool: &PgPool,
    slug: &str,
    detail: &AnimeDetail,
) -> RepositoryResult<()> {
    let mut tx = pool.begin().await?;

    // Save anime detail
    sqlx::query(
        r#"
        INSERT INTO anime_details (
            slug, title, alternate_titles, poster, rating, trailer_url,
            status, studio, release_date, duration, season, type,
            total_episodes, director, casts, genres, synopsis, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, CURRENT_TIMESTAMP)
        ON CONFLICT (slug) DO UPDATE SET
            title = EXCLUDED.title,
            alternate_titles = EXCLUDED.alternate_titles,
            poster = EXCLUDED.poster,
            rating = EXCLUDED.rating,
            trailer_url = EXCLUDED.trailer_url,
            status = EXCLUDED.status,
            studio = EXCLUDED.studio,
            release_date = EXCLUDED.release_date,
            duration = EXCLUDED.duration,
            season = EXCLUDED.season,
            type = EXCLUDED.type,
            total_episodes = EXCLUDED.total_episodes,
            director = EXCLUDED.director,
            casts = EXCLUDED.casts,
            genres = EXCLUDED.genres,
            synopsis = EXCLUDED.synopsis,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(slug)
    .bind(&detail.title)
    .bind(&detail.alternate_titles)
    .bind(&detail.poster)
    .bind(&detail.rating)
    .bind(&detail.trailer_url)
    .bind(&detail.status)
    .bind(&detail.studio)
    .bind(&detail.release_date)
    .bind(&detail.duration)
    .bind(&detail.season)
    .bind(&detail.anime_type)
    .bind(&detail.total_episodes)
    .bind(&detail.director)
    .bind(&detail.casts)
    .bind(&detail.genres)
    .bind(&detail.synopsis)
    .execute(&mut *tx)
    .await?;

    // Save episodes
    for episode in &detail.episodes {
        sqlx::query(
            r#"
            INSERT INTO episodes (anime_slug, number, title, url, release_date, updated_at)
            VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)
            ON CONFLICT (url) DO UPDATE SET
                anime_slug = EXCLUDED.anime_slug,
                number = EXCLUDED.number,
                title = EXCLUDED.title,
                release_date = EXCLUDED.release_date,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(slug)
        .bind(&episode.number)
        .bind(&episode.title)
        .bind(&episode.url)
        .bind(&episode.release_date)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

// ============================================================================
// Cache Layer
// ============================================================================

/// Default cache TTL in milliseconds (1 hour)
pub const DEFAULT_CACHE_TTL_MS: i64 = 3600 * 1000;

/// Check if cached data is still valid (not stale)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `cache_key` - Unique identifier for the cached data (e.g., "updates", "completed", "anime:slug")
/// * `max_age_ms` - Maximum age in milliseconds before cache is considered stale
///
/// # Returns
/// * `Ok(true)` if cache exists and is fresh (less than max_age_ms old)
/// * `Ok(false)` if cache doesn't exist or is stale
pub async fn is_cache_valid(
    pool: &PgPool,
    cache_key: &str,
    max_age_ms: i64,
) -> RepositoryResult<bool> {
    let row = sqlx::query(
        r#"
        SELECT last_fetched
        FROM cache_metadata
        WHERE cache_key = $1
        "#,
    )
    .bind(cache_key)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let last_fetched: DateTime<Utc> = row.get("last_fetched");
            let now = Utc::now();
            let age_ms = (now - last_fetched).num_milliseconds();
            Ok(age_ms < max_age_ms)
        }
        None => Ok(false),
    }
}

/// Update the cache timestamp for a given cache key
///
/// Creates a new cache entry if it doesn't exist, or updates the existing one.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `cache_key` - Unique identifier for the cached data
///
/// # Returns
/// * `Ok(())` on success
pub async fn update_cache_timestamp(pool: &PgPool, cache_key: &str) -> RepositoryResult<()> {
    sqlx::query(
        r#"
        INSERT INTO cache_metadata (cache_key, last_fetched)
        VALUES ($1, CURRENT_TIMESTAMP)
        ON CONFLICT (cache_key) DO UPDATE SET
            last_fetched = CURRENT_TIMESTAMP
        "#,
    )
    .bind(cache_key)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get the last fetched timestamp for a cache key
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `cache_key` - Unique identifier for the cached data
///
/// # Returns
/// * `Ok(Some(timestamp))` if cache entry exists
/// * `Ok(None)` if cache entry doesn't exist
pub async fn get_cache_timestamp(
    pool: &PgPool,
    cache_key: &str,
) -> RepositoryResult<Option<DateTime<Utc>>> {
    let row = sqlx::query(
        r#"
        SELECT last_fetched
        FROM cache_metadata
        WHERE cache_key = $1
        "#,
    )
    .bind(cache_key)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let last_fetched: DateTime<Utc> = row.get("last_fetched");
            Ok(Some(last_fetched))
        }
        None => Ok(None),
    }
}

/// Delete a cache entry
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `cache_key` - Unique identifier for the cached data
///
/// # Returns
/// * `Ok(true)` if entry was deleted
/// * `Ok(false)` if entry didn't exist
pub async fn delete_cache_entry(pool: &PgPool, cache_key: &str) -> RepositoryResult<bool> {
    let result = sqlx::query("DELETE FROM cache_metadata WHERE cache_key = $1")
        .bind(cache_key)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Delete all cache entries
///
/// # Returns
/// * `Ok(count)` - Number of entries deleted
pub async fn delete_all_cache_entries(pool: &PgPool) -> RepositoryResult<u64> {
    let result = sqlx::query("DELETE FROM cache_metadata")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

// ============================================================================
// Crawled Anime Repository
// ============================================================================

/// Save a single crawled anime to the database with upsert logic
///
/// Uses ON CONFLICT UPDATE to update existing records based on slug
pub async fn save_crawled_anime(
    pool: &PgPool,
    anime: &CrawledAnime,
) -> RepositoryResult<()> {
    sqlx::query(
        r#"
        INSERT INTO crawled_anime (
            slug, title, url, thumbnail, status, type, episode_status, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP)
        ON CONFLICT (slug) DO UPDATE SET
            title = EXCLUDED.title,
            url = EXCLUDED.url,
            thumbnail = EXCLUDED.thumbnail,
            status = EXCLUDED.status,
            type = EXCLUDED.type,
            episode_status = EXCLUDED.episode_status,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(&anime.slug)
    .bind(&anime.title)
    .bind(&anime.url)
    .bind(&anime.thumbnail)
    .bind(&anime.status)
    .bind(&anime.anime_type)
    .bind(&anime.episode_status)
    .execute(pool)
    .await?;

    Ok(())
}

/// Save multiple crawled anime to the database with batch upsert for performance
///
/// Uses a transaction to ensure atomicity and ON CONFLICT UPDATE for upsert logic
pub async fn save_crawled_anime_batch(
    pool: &PgPool,
    anime_list: &[CrawledAnime],
) -> RepositoryResult<()> {
    if anime_list.is_empty() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    for anime in anime_list {
        sqlx::query(
            r#"
            INSERT INTO crawled_anime (
                slug, title, url, thumbnail, status, type, episode_status, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP)
            ON CONFLICT (slug) DO UPDATE SET
                title = EXCLUDED.title,
                url = EXCLUDED.url,
                thumbnail = EXCLUDED.thumbnail,
                status = EXCLUDED.status,
                type = EXCLUDED.type,
                episode_status = EXCLUDED.episode_status,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&anime.slug)
        .bind(&anime.title)
        .bind(&anime.url)
        .bind(&anime.thumbnail)
        .bind(&anime.status)
        .bind(&anime.anime_type)
        .bind(&anime.episode_status)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Get the total count of crawled anime in the database
pub async fn get_crawled_anime_count(pool: &PgPool) -> RepositoryResult<i64> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM crawled_anime")
        .fetch_one(pool)
        .await?;

    let count: i64 = row.get("count");
    Ok(count)
}

/// Get a crawled anime by slug
pub async fn get_crawled_anime_by_slug(
    pool: &PgPool,
    slug: &str,
) -> RepositoryResult<Option<CrawledAnimeRecord>> {
    let row = sqlx::query(
        r#"
        SELECT id, slug, title, url, thumbnail, status, type, episode_status, created_at, updated_at
        FROM crawled_anime
        WHERE slug = $1
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let created_at: DateTime<Utc> = row.get("created_at");
            let updated_at: DateTime<Utc> = row.get("updated_at");
            
            Ok(Some(CrawledAnimeRecord {
                id: row.get("id"),
                slug: row.get("slug"),
                title: row.get("title"),
                url: row.get("url"),
                thumbnail: row.get::<Option<String>, _>("thumbnail").unwrap_or_default(),
                status: row.get::<Option<String>, _>("status").unwrap_or_default(),
                anime_type: row.get::<Option<String>, _>("type").unwrap_or_default(),
                episode_status: row.get::<Option<String>, _>("episode_status").unwrap_or_default(),
                created_at: created_at.to_rfc3339(),
                updated_at: updated_at.to_rfc3339(),
            }))
        }
        None => Ok(None),
    }
}

/// Get all crawled anime from the database
pub async fn get_all_crawled_anime(pool: &PgPool) -> RepositoryResult<Vec<CrawledAnimeRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT id, slug, title, url, thumbnail, status, type, episode_status, created_at, updated_at
        FROM crawled_anime
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let anime_list = rows
        .into_iter()
        .map(|row| {
            let created_at: DateTime<Utc> = row.get("created_at");
            let updated_at: DateTime<Utc> = row.get("updated_at");
            
            CrawledAnimeRecord {
                id: row.get("id"),
                slug: row.get("slug"),
                title: row.get("title"),
                url: row.get("url"),
                thumbnail: row.get::<Option<String>, _>("thumbnail").unwrap_or_default(),
                status: row.get::<Option<String>, _>("status").unwrap_or_default(),
                anime_type: row.get::<Option<String>, _>("type").unwrap_or_default(),
                episode_status: row.get::<Option<String>, _>("episode_status").unwrap_or_default(),
                created_at: created_at.to_rfc3339(),
                updated_at: updated_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(anime_list)
}

/// Delete a crawled anime by slug
pub async fn delete_crawled_anime(pool: &PgPool, slug: &str) -> RepositoryResult<bool> {
    let result = sqlx::query("DELETE FROM crawled_anime WHERE slug = $1")
        .bind(slug)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Delete all crawled anime from the database
pub async fn delete_all_crawled_anime(pool: &PgPool) -> RepositoryResult<u64> {
    let result = sqlx::query("DELETE FROM crawled_anime")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test AnimeUpdate
    fn create_test_anime_update(episode_url: &str) -> AnimeUpdate {
        AnimeUpdate {
            slug: "test-series".to_string(),
            title: "Test Episode".to_string(),
            episode_url: episode_url.to_string(),
            thumbnail: "https://example.com/thumb.jpg".to_string(),
            episode_number: "1".to_string(),
            anime_type: "TV".to_string(),
            series_title: "Test Series".to_string(),
            series_url: "https://example.com/series".to_string(),
            status: "Ongoing".to_string(),
            release_info: "2024-01-01".to_string(),
        }
    }

    // Helper function to create a test CompletedAnime
    fn create_test_completed_anime(url: &str) -> CompletedAnime {
        CompletedAnime {
            slug: "test-completed".to_string(),
            title: "Test Completed Anime".to_string(),
            url: url.to_string(),
            thumbnail: "https://example.com/thumb.jpg".to_string(),
            anime_type: "TV".to_string(),
            episode_count: "24".to_string(),
            status: "Completed".to_string(),
            posted_by: "Admin".to_string(),
            posted_at: "2024-01-01".to_string(),
            series_title: "Test Series".to_string(),
            series_url: "https://example.com/series".to_string(),
            genres: vec!["Action".to_string(), "Adventure".to_string()],
            rating: "8.5".to_string(),
        }
    }

    // Helper function to create a test AnimeDetail
    fn create_test_anime_detail() -> AnimeDetail {
        AnimeDetail {
            title: "Test Anime".to_string(),
            alternate_titles: "Test Alt Title".to_string(),
            poster: "https://example.com/poster.jpg".to_string(),
            rating: "8.5".to_string(),
            trailer_url: "https://youtube.com/watch?v=test".to_string(),
            status: "Ongoing".to_string(),
            studio: "Test Studio".to_string(),
            release_date: "2024-01-01".to_string(),
            duration: "24 min".to_string(),
            season: "Winter 2024".to_string(),
            anime_type: "TV".to_string(),
            total_episodes: "24".to_string(),
            director: "Test Director".to_string(),
            casts: vec!["Actor 1".to_string(), "Actor 2".to_string()],
            genres: vec!["Action".to_string(), "Adventure".to_string()],
            synopsis: "Test synopsis".to_string(),
            episodes: vec![
                Episode {
                    slug: "ep1".to_string(),
                    number: "1".to_string(),
                    title: "Episode 1".to_string(),
                    url: "https://example.com/ep1".to_string(),
                    release_date: "2024-01-01".to_string(),
                },
                Episode {
                    slug: "ep2".to_string(),
                    number: "2".to_string(),
                    title: "Episode 2".to_string(),
                    url: "https://example.com/ep2".to_string(),
                    release_date: "2024-01-08".to_string(),
                },
            ],
        }
    }

    // Helper function to create a test VideoSource
    fn create_test_video_source(server: &str, quality: &str) -> VideoSource {
        VideoSource {
            server: server.to_string(),
            quality: quality.to_string(),
            url: format!("https://example.com/video-{}-{}.mp4", server, quality),
        }
    }

    #[test]
    fn test_create_anime_update() {
        let update = create_test_anime_update("https://example.com/ep1");
        assert_eq!(update.title, "Test Episode");
        assert_eq!(update.episode_url, "https://example.com/ep1");
    }

    #[test]
    fn test_create_completed_anime() {
        let anime = create_test_completed_anime("https://example.com/anime1");
        assert_eq!(anime.title, "Test Completed Anime");
        assert_eq!(anime.url, "https://example.com/anime1");
        assert_eq!(anime.genres.len(), 2);
    }

    #[test]
    fn test_create_anime_detail() {
        let detail = create_test_anime_detail();
        assert_eq!(detail.title, "Test Anime");
        assert_eq!(detail.episodes.len(), 2);
        assert_eq!(detail.genres.len(), 2);
        assert_eq!(detail.casts.len(), 2);
    }

    #[test]
    fn test_create_video_source() {
        let source = create_test_video_source("SOKUJA", "720p");
        assert_eq!(source.server, "SOKUJA");
        assert_eq!(source.quality, "720p");
        assert!(source.url.contains("SOKUJA"));
        assert!(source.url.contains("720p"));
    }

    // Integration tests that require a database connection
    // These are marked with #[ignore] and can be run with `cargo test -- --ignored`

    #[tokio::test]
    #[ignore]
    async fn test_anime_updates_crud() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect");

        // Clean up first
        let _ = delete_all_anime_updates(&pool).await;

        // Create
        let updates = vec![
            create_test_anime_update("https://test.com/ep1"),
            create_test_anime_update("https://test.com/ep2"),
        ];
        save_anime_updates(&pool, &updates).await.expect("Failed to save");

        // Read
        let fetched = get_anime_updates(&pool).await.expect("Failed to fetch");
        assert!(fetched.len() >= 2);

        // Update (upsert with same URL)
        let mut updated = create_test_anime_update("https://test.com/ep1");
        updated.title = "Updated Title".to_string();
        save_anime_updates(&pool, &[updated]).await.expect("Failed to update");

        // Verify update
        let fetched = get_anime_updates(&pool).await.expect("Failed to fetch");
        let found = fetched.iter().find(|u| u.episode_url == "https://test.com/ep1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Updated Title");

        // Clean up
        delete_all_anime_updates(&pool).await.expect("Failed to delete");
    }

    #[tokio::test]
    #[ignore]
    async fn test_completed_anime_crud() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect");

        // Clean up first
        let _ = delete_all_completed_anime(&pool).await;

        // Create
        let anime_list = vec![
            create_test_completed_anime("https://test.com/anime1"),
            create_test_completed_anime("https://test.com/anime2"),
        ];
        save_completed_anime(&pool, &anime_list).await.expect("Failed to save");

        // Read
        let fetched = get_completed_anime(&pool).await.expect("Failed to fetch");
        assert!(fetched.len() >= 2);

        // Clean up
        delete_all_completed_anime(&pool).await.expect("Failed to delete");
    }

    #[tokio::test]
    #[ignore]
    async fn test_anime_detail_with_episodes_crud() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect");

        let slug = "test-anime-slug";

        // Clean up first
        let _ = delete_anime_detail(&pool, slug).await;

        // Create with episodes
        let detail = create_test_anime_detail();
        save_anime_detail_with_episodes(&pool, slug, &detail)
            .await
            .expect("Failed to save");

        // Read
        let fetched = get_anime_detail(&pool, slug).await.expect("Failed to fetch");
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.title, "Test Anime");
        assert_eq!(fetched.episodes.len(), 2);

        // Update (upsert)
        let mut updated_detail = create_test_anime_detail();
        updated_detail.title = "Updated Anime Title".to_string();
        save_anime_detail_with_episodes(&pool, slug, &updated_detail)
            .await
            .expect("Failed to update");

        // Verify update
        let fetched = get_anime_detail(&pool, slug).await.expect("Failed to fetch");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().title, "Updated Anime Title");

        // Clean up
        delete_anime_detail(&pool, slug).await.expect("Failed to delete");
    }

    #[tokio::test]
    #[ignore]
    async fn test_video_sources_crud() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect");

        let episode_url = "https://test.com/episode/test-ep";

        // Clean up first
        let _ = delete_video_sources(&pool, episode_url).await;

        // Create
        let sources = vec![
            create_test_video_source("SOKUJA", "480p"),
            create_test_video_source("SOKUJA", "720p"),
            create_test_video_source("SOKUJA", "1080p"),
        ];
        save_video_sources(&pool, episode_url, &sources)
            .await
            .expect("Failed to save");

        // Read
        let fetched = get_video_sources(&pool, episode_url)
            .await
            .expect("Failed to fetch");
        assert_eq!(fetched.len(), 3);

        // Update (replace all sources)
        let new_sources = vec![
            create_test_video_source("NEW_SERVER", "720p"),
        ];
        save_video_sources(&pool, episode_url, &new_sources)
            .await
            .expect("Failed to update");

        // Verify update
        let fetched = get_video_sources(&pool, episode_url)
            .await
            .expect("Failed to fetch");
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].server, "NEW_SERVER");

        // Clean up
        delete_video_sources(&pool, episode_url)
            .await
            .expect("Failed to delete");
    }

    // Cache layer tests

    #[test]
    fn test_default_cache_ttl() {
        // 1 hour in milliseconds
        assert_eq!(DEFAULT_CACHE_TTL_MS, 3600 * 1000);
        assert_eq!(DEFAULT_CACHE_TTL_MS, 3_600_000);
    }

    #[tokio::test]
    #[ignore]
    async fn test_cache_layer_basic_operations() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect");

        let cache_key = "test:cache:basic";

        // Clean up first
        let _ = delete_cache_entry(&pool, cache_key).await;

        // Test 1: Cache should not be valid when entry doesn't exist
        let is_valid = is_cache_valid(&pool, cache_key, DEFAULT_CACHE_TTL_MS)
            .await
            .expect("Failed to check cache");
        assert!(!is_valid, "Cache should not be valid when entry doesn't exist");

        // Test 2: Get timestamp should return None when entry doesn't exist
        let timestamp = get_cache_timestamp(&pool, cache_key)
            .await
            .expect("Failed to get timestamp");
        assert!(timestamp.is_none(), "Timestamp should be None when entry doesn't exist");

        // Test 3: Update cache timestamp (creates new entry)
        update_cache_timestamp(&pool, cache_key)
            .await
            .expect("Failed to update cache timestamp");

        // Test 4: Cache should now be valid (just created)
        let is_valid = is_cache_valid(&pool, cache_key, DEFAULT_CACHE_TTL_MS)
            .await
            .expect("Failed to check cache");
        assert!(is_valid, "Cache should be valid immediately after creation");

        // Test 5: Get timestamp should return Some value
        let timestamp = get_cache_timestamp(&pool, cache_key)
            .await
            .expect("Failed to get timestamp");
        assert!(timestamp.is_some(), "Timestamp should exist after creation");

        // Test 6: Delete cache entry
        let deleted = delete_cache_entry(&pool, cache_key)
            .await
            .expect("Failed to delete cache entry");
        assert!(deleted, "Should have deleted the cache entry");

        // Test 7: Cache should not be valid after deletion
        let is_valid = is_cache_valid(&pool, cache_key, DEFAULT_CACHE_TTL_MS)
            .await
            .expect("Failed to check cache");
        assert!(!is_valid, "Cache should not be valid after deletion");
    }

    #[tokio::test]
    #[ignore]
    async fn test_cache_freshness_with_short_ttl() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect");

        let cache_key = "test:cache:freshness";

        // Clean up first
        let _ = delete_cache_entry(&pool, cache_key).await;

        // Create cache entry
        update_cache_timestamp(&pool, cache_key)
            .await
            .expect("Failed to update cache timestamp");

        // Test with very short TTL (1ms) - should be stale immediately
        // Note: We use a small delay to ensure the cache is stale
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let is_valid = is_cache_valid(&pool, cache_key, 1)
            .await
            .expect("Failed to check cache");
        assert!(!is_valid, "Cache should be stale with 1ms TTL after 10ms");

        // Test with long TTL (1 hour) - should still be fresh
        let is_valid = is_cache_valid(&pool, cache_key, DEFAULT_CACHE_TTL_MS)
            .await
            .expect("Failed to check cache");
        assert!(is_valid, "Cache should be fresh with 1 hour TTL");

        // Clean up
        delete_cache_entry(&pool, cache_key)
            .await
            .expect("Failed to delete cache entry");
    }

    #[tokio::test]
    #[ignore]
    async fn test_cache_upsert_behavior() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect");

        let cache_key = "test:cache:upsert";

        // Clean up first
        let _ = delete_cache_entry(&pool, cache_key).await;

        // Create initial cache entry
        update_cache_timestamp(&pool, cache_key)
            .await
            .expect("Failed to create cache entry");

        let first_timestamp = get_cache_timestamp(&pool, cache_key)
            .await
            .expect("Failed to get timestamp")
            .expect("Timestamp should exist");

        // Wait a bit and update again
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        update_cache_timestamp(&pool, cache_key)
            .await
            .expect("Failed to update cache entry");

        let second_timestamp = get_cache_timestamp(&pool, cache_key)
            .await
            .expect("Failed to get timestamp")
            .expect("Timestamp should exist");

        // Second timestamp should be newer
        assert!(
            second_timestamp > first_timestamp,
            "Second timestamp should be newer than first"
        );

        // Clean up
        delete_cache_entry(&pool, cache_key)
            .await
            .expect("Failed to delete cache entry");
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_all_cache_entries() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect");

        // Create multiple cache entries
        let keys = vec!["test:cache:all:1", "test:cache:all:2", "test:cache:all:3"];
        
        for key in &keys {
            update_cache_timestamp(&pool, key)
                .await
                .expect("Failed to create cache entry");
        }

        // Verify all entries exist
        for key in &keys {
            let is_valid = is_cache_valid(&pool, key, DEFAULT_CACHE_TTL_MS)
                .await
                .expect("Failed to check cache");
            assert!(is_valid, "Cache entry should exist for {}", key);
        }

        // Delete all cache entries
        let deleted_count = delete_all_cache_entries(&pool)
            .await
            .expect("Failed to delete all cache entries");
        assert!(deleted_count >= 3, "Should have deleted at least 3 entries");

        // Verify all entries are gone
        for key in &keys {
            let is_valid = is_cache_valid(&pool, key, DEFAULT_CACHE_TTL_MS)
                .await
                .expect("Failed to check cache");
            assert!(!is_valid, "Cache entry should not exist for {} after deletion", key);
        }
    }

    // Crawled Anime Repository Tests

    // Helper function to create a test CrawledAnime
    fn create_test_crawled_anime(slug: &str) -> CrawledAnime {
        CrawledAnime {
            slug: slug.to_string(),
            title: format!("Test Anime {}", slug),
            url: format!("https://example.com/anime/{}/", slug),
            thumbnail: format!("https://example.com/thumb/{}.jpg", slug),
            status: "Ongoing".to_string(),
            anime_type: "TV".to_string(),
            episode_status: "12/24".to_string(),
        }
    }

    #[test]
    fn test_create_crawled_anime() {
        let anime = create_test_crawled_anime("test-anime");
        assert_eq!(anime.slug, "test-anime");
        assert_eq!(anime.title, "Test Anime test-anime");
        assert!(anime.url.contains("test-anime"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_crawled_anime_single_save() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect");

        let slug = "test-single-crawled";

        // Clean up first
        let _ = delete_crawled_anime(&pool, slug).await;

        // Create
        let anime = create_test_crawled_anime(slug);
        save_crawled_anime(&pool, &anime).await.expect("Failed to save");

        // Read
        let fetched = get_crawled_anime_by_slug(&pool, slug)
            .await
            .expect("Failed to fetch");
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.slug, slug);
        assert_eq!(fetched.title, anime.title);

        // Clean up
        delete_crawled_anime(&pool, slug).await.expect("Failed to delete");
    }

    #[tokio::test]
    #[ignore]
    async fn test_crawled_anime_batch_save() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect");

        let slugs = vec!["test-batch-1", "test-batch-2", "test-batch-3"];

        // Clean up first
        for slug in &slugs {
            let _ = delete_crawled_anime(&pool, slug).await;
        }

        // Create batch
        let anime_list: Vec<CrawledAnime> = slugs.iter()
            .map(|s| create_test_crawled_anime(s))
            .collect();
        
        save_crawled_anime_batch(&pool, &anime_list)
            .await
            .expect("Failed to save batch");

        // Verify all were saved
        for slug in &slugs {
            let fetched = get_crawled_anime_by_slug(&pool, slug)
                .await
                .expect("Failed to fetch");
            assert!(fetched.is_some(), "Should find anime with slug {}", slug);
        }

        // Clean up
        for slug in &slugs {
            delete_crawled_anime(&pool, slug).await.expect("Failed to delete");
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_crawled_anime_upsert() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect");

        let slug = "test-upsert-crawled";

        // Clean up first
        let _ = delete_crawled_anime(&pool, slug).await;

        // Create initial
        let anime = create_test_crawled_anime(slug);
        save_crawled_anime(&pool, &anime).await.expect("Failed to save");

        // Update with same slug
        let mut updated_anime = create_test_crawled_anime(slug);
        updated_anime.title = "Updated Title".to_string();
        updated_anime.status = "Completed".to_string();
        save_crawled_anime(&pool, &updated_anime).await.expect("Failed to upsert");

        // Verify update
        let fetched = get_crawled_anime_by_slug(&pool, slug)
            .await
            .expect("Failed to fetch")
            .expect("Should find anime");
        assert_eq!(fetched.title, "Updated Title");
        assert_eq!(fetched.status, "Completed");

        // Verify count is still 1 (not duplicated)
        let count_before = get_crawled_anime_count(&pool).await.expect("Failed to count");
        save_crawled_anime(&pool, &updated_anime).await.expect("Failed to upsert again");
        let count_after = get_crawled_anime_count(&pool).await.expect("Failed to count");
        
        // Count should not increase (upsert, not insert)
        assert_eq!(count_before, count_after, "Count should not increase on upsert");

        // Clean up
        delete_crawled_anime(&pool, slug).await.expect("Failed to delete");
    }

    #[tokio::test]
    #[ignore]
    async fn test_crawled_anime_count() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect");

        // Get initial count
        let initial_count = get_crawled_anime_count(&pool).await.expect("Failed to count");

        // Add some anime
        let slugs = vec!["test-count-1", "test-count-2"];
        for slug in &slugs {
            let _ = delete_crawled_anime(&pool, slug).await;
            let anime = create_test_crawled_anime(slug);
            save_crawled_anime(&pool, &anime).await.expect("Failed to save");
        }

        // Verify count increased
        let new_count = get_crawled_anime_count(&pool).await.expect("Failed to count");
        assert_eq!(new_count, initial_count + 2);

        // Clean up
        for slug in &slugs {
            delete_crawled_anime(&pool, slug).await.expect("Failed to delete");
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_crawled_anime_empty_batch() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.expect("Failed to connect");

        // Saving empty batch should succeed without error
        let empty_list: Vec<CrawledAnime> = vec![];
        save_crawled_anime_batch(&pool, &empty_list)
            .await
            .expect("Empty batch save should succeed");
    }
}

