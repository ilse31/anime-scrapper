//! Repository module for anime data persistence
//!
//! Provides CRUD operations with upsert logic for anime_updates, completed_anime,
//! anime_details, episodes, video_sources, crawled_anime, users, user_favorites,
//! user_subscriptions, and user_history tables.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use thiserror::Error;

use crate::models::{
    CrawledAnime, CrawledAnimeRecord, User, UserFavorite, UserHistory, UserSubscription,
};
use crate::parser::{AnimeDetail, AnimeUpdate, CompletedAnime, Episode, VideoSource};

/// Repository-related errors
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Email already exists")]
    EmailAlreadyExists,
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
pub async fn save_anime_updates(pool: &PgPool, updates: &[AnimeUpdate]) -> RepositoryResult<()> {
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
            let series_url: String = row
                .get::<Option<String>, _>("series_url")
                .unwrap_or_default();
            AnimeUpdate {
                slug: extract_slug_from_url(&series_url),
                title: row.get::<String, _>("title"),
                episode_url: row.get::<String, _>("episode_url"),
                thumbnail: row
                    .get::<Option<String>, _>("thumbnail")
                    .unwrap_or_default(),
                episode_number: row
                    .get::<Option<String>, _>("episode_number")
                    .unwrap_or_default(),
                anime_type: row.get::<Option<String>, _>("type").unwrap_or_default(),
                series_title: row
                    .get::<Option<String>, _>("series_title")
                    .unwrap_or_default(),
                series_url,
                status: row.get::<Option<String>, _>("status").unwrap_or_default(),
                release_info: row
                    .get::<Option<String>, _>("release_info")
                    .unwrap_or_default(),
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
                thumbnail: row
                    .get::<Option<String>, _>("thumbnail")
                    .unwrap_or_default(),
                anime_type: row.get::<Option<String>, _>("type").unwrap_or_default(),
                episode_count: row
                    .get::<Option<String>, _>("episode_count")
                    .unwrap_or_default(),
                status: row.get::<Option<String>, _>("status").unwrap_or_default(),
                posted_by: row
                    .get::<Option<String>, _>("posted_by")
                    .unwrap_or_default(),
                posted_at: row
                    .get::<Option<String>, _>("posted_at")
                    .unwrap_or_default(),
                series_title: row
                    .get::<Option<String>, _>("series_title")
                    .unwrap_or_default(),
                series_url: row
                    .get::<Option<String>, _>("series_url")
                    .unwrap_or_default(),
                genres: row
                    .get::<Option<Vec<String>>, _>("genres")
                    .unwrap_or_default(),
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
                alternate_titles: row
                    .get::<Option<String>, _>("alternate_titles")
                    .unwrap_or_default(),
                poster: row.get::<Option<String>, _>("poster").unwrap_or_default(),
                rating: row.get::<Option<String>, _>("rating").unwrap_or_default(),
                trailer_url: row
                    .get::<Option<String>, _>("trailer_url")
                    .unwrap_or_default(),
                status: row.get::<Option<String>, _>("status").unwrap_or_default(),
                studio: row.get::<Option<String>, _>("studio").unwrap_or_default(),
                release_date: row
                    .get::<Option<String>, _>("release_date")
                    .unwrap_or_default(),
                duration: row.get::<Option<String>, _>("duration").unwrap_or_default(),
                season: row.get::<Option<String>, _>("season").unwrap_or_default(),
                anime_type: row.get::<Option<String>, _>("type").unwrap_or_default(),
                total_episodes: row
                    .get::<Option<String>, _>("total_episodes")
                    .unwrap_or_default(),
                director: row.get::<Option<String>, _>("director").unwrap_or_default(),
                casts: row
                    .get::<Option<Vec<String>>, _>("casts")
                    .unwrap_or_default(),
                genres: row
                    .get::<Option<Vec<String>>, _>("genres")
                    .unwrap_or_default(),
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
                release_date: row
                    .get::<Option<String>, _>("release_date")
                    .unwrap_or_default(),
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
pub async fn get_video_sources(
    pool: &PgPool,
    episode_url: &str,
) -> RepositoryResult<Vec<VideoSource>> {
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
pub async fn save_crawled_anime(pool: &PgPool, anime: &CrawledAnime) -> RepositoryResult<()> {
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
                thumbnail: row
                    .get::<Option<String>, _>("thumbnail")
                    .unwrap_or_default(),
                status: row.get::<Option<String>, _>("status").unwrap_or_default(),
                anime_type: row.get::<Option<String>, _>("type").unwrap_or_default(),
                episode_status: row
                    .get::<Option<String>, _>("episode_status")
                    .unwrap_or_default(),
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
                thumbnail: row
                    .get::<Option<String>, _>("thumbnail")
                    .unwrap_or_default(),
                status: row.get::<Option<String>, _>("status").unwrap_or_default(),
                anime_type: row.get::<Option<String>, _>("type").unwrap_or_default(),
                episode_status: row
                    .get::<Option<String>, _>("episode_status")
                    .unwrap_or_default(),
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

/// Create a new user with email and password
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `email` - User's email address
/// * `password_hash` - Bcrypt hashed password
/// * `name` - Optional display name
///
/// # Returns
/// * `Ok(User)` - The created user
/// * `Err(RepositoryError::EmailAlreadyExists)` - If email is already registered
pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    name: Option<&str>,
) -> RepositoryResult<User> {
    let row = sqlx::query(
        r#"
        INSERT INTO users (email, password_hash, name, created_at, updated_at)
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING id, email, name, avatar, created_at
        "#,
    )
    .bind(email)
    .bind(password_hash)
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("users_email_key") {
                return RepositoryError::EmailAlreadyExists;
            }
        }
        RepositoryError::DatabaseError(e)
    })?;

    let created_at: DateTime<Utc> = row.get("created_at");
    Ok(User {
        id: row.get("id"),
        email: row.get("email"),
        name: row.get("name"),
        avatar: row.get("avatar"),
        created_at: created_at.to_rfc3339(),
    })
}

/// Create a new user with Google OAuth
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `email` - User's email from Google
/// * `google_id` - Google user ID
/// * `name` - User's name from Google
/// * `avatar` - Optional profile picture URL
///
/// # Returns
/// * `Ok(User)` - The created user
/// * `Err(RepositoryError)` - If creation fails
pub async fn create_google_user(
    pool: &PgPool,
    email: &str,
    google_id: &str,
    name: &str,
    avatar: Option<&str>,
) -> RepositoryResult<User> {
    let row = sqlx::query(
        r#"
        INSERT INTO users (email, google_id, name, avatar, created_at, updated_at)
        VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING id, email, name, avatar, created_at
        "#,
    )
    .bind(email)
    .bind(google_id)
    .bind(name)
    .bind(avatar)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("users_email_key") {
                return RepositoryError::EmailAlreadyExists;
            }
        }
        RepositoryError::DatabaseError(e)
    })?;

    let created_at: DateTime<Utc> = row.get("created_at");
    Ok(User {
        id: row.get("id"),
        email: row.get("email"),
        name: row.get("name"),
        avatar: row.get("avatar"),
        created_at: created_at.to_rfc3339(),
    })
}

/// Find a user by email address
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `email` - Email address to search for
///
/// # Returns
/// * `Ok(Some(user, password_hash))` - User found with optional password hash
/// * `Ok(None)` - User not found
pub async fn find_user_by_email(
    pool: &PgPool,
    email: &str,
) -> RepositoryResult<Option<(User, Option<String>)>> {
    let row = sqlx::query(
        r#"
        SELECT id, email, password_hash, name, avatar, created_at
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let created_at: DateTime<Utc> = row.get("created_at");
            let user = User {
                id: row.get("id"),
                email: row.get("email"),
                name: row.get("name"),
                avatar: row.get("avatar"),
                created_at: created_at.to_rfc3339(),
            };
            let password_hash: Option<String> = row.get("password_hash");
            Ok(Some((user, password_hash)))
        }
        None => Ok(None),
    }
}

/// Find a user by Google ID
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `google_id` - Google user ID to search for
///
/// # Returns
/// * `Ok(Some(User))` - User found
/// * `Ok(None)` - User not found
pub async fn find_user_by_google_id(
    pool: &PgPool,
    google_id: &str,
) -> RepositoryResult<Option<User>> {
    let row = sqlx::query(
        r#"
        SELECT id, email, name, avatar, created_at
        FROM users
        WHERE google_id = $1
        "#,
    )
    .bind(google_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let created_at: DateTime<Utc> = row.get("created_at");
            Ok(Some(User {
                id: row.get("id"),
                email: row.get("email"),
                name: row.get("name"),
                avatar: row.get("avatar"),
                created_at: created_at.to_rfc3339(),
            }))
        }
        None => Ok(None),
    }
}

/// Find a user by ID
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID to search for
///
/// # Returns
/// * `Ok(Some(User))` - User found
/// * `Ok(None)` - User not found
pub async fn find_user_by_id(pool: &PgPool, user_id: i32) -> RepositoryResult<Option<User>> {
    let row = sqlx::query(
        r#"
        SELECT id, email, name, avatar, created_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let created_at: DateTime<Utc> = row.get("created_at");
            Ok(Some(User {
                id: row.get("id"),
                email: row.get("email"),
                name: row.get("name"),
                avatar: row.get("avatar"),
                created_at: created_at.to_rfc3339(),
            }))
        }
        None => Ok(None),
    }
}

/// Link a Google account to an existing user
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID to link
/// * `google_id` - Google user ID to link
///
/// # Returns
/// * `Ok(())` - Successfully linked
/// * `Err(RepositoryError)` - If linking fails
pub async fn link_google_account(
    pool: &PgPool,
    user_id: i32,
    google_id: &str,
) -> RepositoryResult<()> {
    sqlx::query(
        r#"
        UPDATE users
        SET google_id = $1, updated_at = CURRENT_TIMESTAMP
        WHERE id = $2
        "#,
    )
    .bind(google_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete a user by ID
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID to delete
///
/// # Returns
/// * `Ok(true)` - User was deleted
/// * `Ok(false)` - User not found
pub async fn delete_user(pool: &PgPool, user_id: i32) -> RepositoryResult<bool> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Add an anime to user's favorites
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
/// * `anime_slug` - Anime slug identifier
/// * `anime_title` - Anime title for display
/// * `thumbnail` - Thumbnail image URL
///
/// # Returns
/// * `Ok(UserFavorite)` - The created favorite
/// * `Err(RepositoryError::Conflict)` - If already favorited
pub async fn add_favorite(
    pool: &PgPool,
    user_id: i32,
    anime_slug: &str,
    anime_title: &str,
    thumbnail: &str,
) -> RepositoryResult<UserFavorite> {
    let row = sqlx::query(
        r#"
        INSERT INTO user_favorites (user_id, anime_slug, anime_title, thumbnail, created_at)
        VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
        RETURNING anime_slug, anime_title, thumbnail, created_at
        "#,
    )
    .bind(user_id)
    .bind(anime_slug)
    .bind(anime_title)
    .bind(thumbnail)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("user_favorites_user_anime_unique") {
                return RepositoryError::Conflict("Anime already in favorites".to_string());
            }
        }
        RepositoryError::DatabaseError(e)
    })?;

    let created_at: DateTime<Utc> = row.get("created_at");
    Ok(UserFavorite {
        anime_slug: row.get("anime_slug"),
        anime_title: row.get("anime_title"),
        thumbnail: row
            .get::<Option<String>, _>("thumbnail")
            .unwrap_or_default(),
        created_at: created_at.to_rfc3339(),
    })
}

/// Get all favorites for a user
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
///
/// # Returns
/// * `Ok(Vec<UserFavorite>)` - List of favorites
pub async fn get_favorites(pool: &PgPool, user_id: i32) -> RepositoryResult<Vec<UserFavorite>> {
    let rows = sqlx::query(
        r#"
        SELECT anime_slug, anime_title, thumbnail, created_at
        FROM user_favorites
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let favorites = rows
        .into_iter()
        .map(|row| {
            let created_at: DateTime<Utc> = row.get("created_at");
            UserFavorite {
                anime_slug: row.get("anime_slug"),
                anime_title: row.get("anime_title"),
                thumbnail: row
                    .get::<Option<String>, _>("thumbnail")
                    .unwrap_or_default(),
                created_at: created_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(favorites)
}

/// Remove an anime from user's favorites
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
/// * `anime_slug` - Anime slug to remove
///
/// # Returns
/// * `Ok(true)` - Favorite was removed
/// * `Ok(false)` - Favorite not found
pub async fn remove_favorite(
    pool: &PgPool,
    user_id: i32,
    anime_slug: &str,
) -> RepositoryResult<bool> {
    let result = sqlx::query("DELETE FROM user_favorites WHERE user_id = $1 AND anime_slug = $2")
        .bind(user_id)
        .bind(anime_slug)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Check if an anime is in user's favorites
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
/// * `anime_slug` - Anime slug to check
///
/// # Returns
/// * `Ok(true)` - Anime is favorited
/// * `Ok(false)` - Anime is not favorited
pub async fn is_favorite(pool: &PgPool, user_id: i32, anime_slug: &str) -> RepositoryResult<bool> {
    let row = sqlx::query("SELECT 1 FROM user_favorites WHERE user_id = $1 AND anime_slug = $2")
        .bind(user_id)
        .bind(anime_slug)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

/// Subscribe to an anime series
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
/// * `anime_slug` - Anime slug identifier
/// * `anime_title` - Anime title for display
/// * `thumbnail` - Thumbnail image URL
///
/// # Returns
/// * `Ok(UserSubscription)` - The created subscription
/// * `Err(RepositoryError::Conflict)` - If already subscribed
pub async fn add_subscription(
    pool: &PgPool,
    user_id: i32,
    anime_slug: &str,
    anime_title: &str,
    thumbnail: &str,
) -> RepositoryResult<UserSubscription> {
    let row = sqlx::query(
        r#"
        INSERT INTO user_subscriptions (user_id, anime_slug, anime_title, thumbnail, created_at)
        VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
        RETURNING anime_slug, anime_title, thumbnail, created_at
        "#,
    )
    .bind(user_id)
    .bind(anime_slug)
    .bind(anime_title)
    .bind(thumbnail)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("user_subscriptions_user_anime_unique") {
                return RepositoryError::Conflict("Already subscribed to this anime".to_string());
            }
        }
        RepositoryError::DatabaseError(e)
    })?;

    let created_at: DateTime<Utc> = row.get("created_at");
    Ok(UserSubscription {
        anime_slug: row.get("anime_slug"),
        anime_title: row.get("anime_title"),
        thumbnail: row
            .get::<Option<String>, _>("thumbnail")
            .unwrap_or_default(),
        created_at: created_at.to_rfc3339(),
    })
}

/// Get all subscriptions for a user
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
///
/// # Returns
/// * `Ok(Vec<UserSubscription>)` - List of subscriptions
pub async fn get_subscriptions(
    pool: &PgPool,
    user_id: i32,
) -> RepositoryResult<Vec<UserSubscription>> {
    let rows = sqlx::query(
        r#"
        SELECT anime_slug, anime_title, thumbnail, created_at
        FROM user_subscriptions
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let subscriptions = rows
        .into_iter()
        .map(|row| {
            let created_at: DateTime<Utc> = row.get("created_at");
            UserSubscription {
                anime_slug: row.get("anime_slug"),
                anime_title: row.get("anime_title"),
                thumbnail: row
                    .get::<Option<String>, _>("thumbnail")
                    .unwrap_or_default(),
                created_at: created_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(subscriptions)
}

/// Unsubscribe from an anime series
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
/// * `anime_slug` - Anime slug to unsubscribe from
///
/// # Returns
/// * `Ok(true)` - Subscription was removed
/// * `Ok(false)` - Subscription not found
pub async fn remove_subscription(
    pool: &PgPool,
    user_id: i32,
    anime_slug: &str,
) -> RepositoryResult<bool> {
    let result =
        sqlx::query("DELETE FROM user_subscriptions WHERE user_id = $1 AND anime_slug = $2")
            .bind(user_id)
            .bind(anime_slug)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// Check if user is subscribed to an anime
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
/// * `anime_slug` - Anime slug to check
///
/// # Returns
/// * `Ok(true)` - User is subscribed
/// * `Ok(false)` - User is not subscribed
pub async fn is_subscribed(
    pool: &PgPool,
    user_id: i32,
    anime_slug: &str,
) -> RepositoryResult<bool> {
    let row =
        sqlx::query("SELECT 1 FROM user_subscriptions WHERE user_id = $1 AND anime_slug = $2")
            .bind(user_id)
            .bind(anime_slug)
            .fetch_optional(pool)
            .await?;
    Ok(row.is_some())
}

/// Add or update an episode in user's watch history
///
/// If the episode already exists in history, updates the watched_at timestamp.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
/// * `episode_slug` - Episode slug identifier
/// * `anime_slug` - Parent anime slug
/// * `episode_title` - Episode title for display
/// * `anime_title` - Anime title for display
/// * `thumbnail` - Thumbnail image URL
///
/// # Returns
/// * `Ok(UserHistory)` - The created/updated history entry
pub async fn add_to_history(
    pool: &PgPool,
    user_id: i32,
    episode_slug: &str,
    anime_slug: &str,
    episode_title: &str,
    anime_title: &str,
    thumbnail: &str,
) -> RepositoryResult<UserHistory> {
    let row = sqlx::query(
        r#"
        INSERT INTO user_history (user_id, episode_slug, anime_slug, episode_title, anime_title, thumbnail, watched_at)
        VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP)
        ON CONFLICT (user_id, episode_slug) DO UPDATE SET
            anime_slug = EXCLUDED.anime_slug,
            episode_title = EXCLUDED.episode_title,
            anime_title = EXCLUDED.anime_title,
            thumbnail = EXCLUDED.thumbnail,
            watched_at = CURRENT_TIMESTAMP
        RETURNING episode_slug, anime_slug, episode_title, anime_title, thumbnail, watched_at
        "#,
    )
    .bind(user_id)
    .bind(episode_slug)
    .bind(anime_slug)
    .bind(episode_title)
    .bind(anime_title)
    .bind(thumbnail)
    .fetch_one(pool)
    .await?;

    let watched_at: DateTime<Utc> = row.get("watched_at");
    Ok(UserHistory {
        episode_slug: row.get("episode_slug"),
        anime_slug: row.get("anime_slug"),
        episode_title: row
            .get::<Option<String>, _>("episode_title")
            .unwrap_or_default(),
        anime_title: row
            .get::<Option<String>, _>("anime_title")
            .unwrap_or_default(),
        thumbnail: row
            .get::<Option<String>, _>("thumbnail")
            .unwrap_or_default(),
        watched_at: watched_at.to_rfc3339(),
    })
}

/// Get user's watch history sorted by most recently watched
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
///
/// # Returns
/// * `Ok(Vec<UserHistory>)` - List of history entries sorted by most recent first
pub async fn get_history(pool: &PgPool, user_id: i32) -> RepositoryResult<Vec<UserHistory>> {
    let rows = sqlx::query(
        r#"
        SELECT episode_slug, anime_slug, episode_title, anime_title, thumbnail, watched_at
        FROM user_history
        WHERE user_id = $1
        ORDER BY watched_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let history = rows
        .into_iter()
        .map(|row| {
            let watched_at: DateTime<Utc> = row.get("watched_at");
            UserHistory {
                episode_slug: row.get("episode_slug"),
                anime_slug: row.get("anime_slug"),
                episode_title: row
                    .get::<Option<String>, _>("episode_title")
                    .unwrap_or_default(),
                anime_title: row
                    .get::<Option<String>, _>("anime_title")
                    .unwrap_or_default(),
                thumbnail: row
                    .get::<Option<String>, _>("thumbnail")
                    .unwrap_or_default(),
                watched_at: watched_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(history)
}

/// Remove an episode from user's watch history
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
/// * `episode_slug` - Episode slug to remove
///
/// # Returns
/// * `Ok(true)` - History entry was removed
/// * `Ok(false)` - History entry not found
pub async fn remove_from_history(
    pool: &PgPool,
    user_id: i32,
    episode_slug: &str,
) -> RepositoryResult<bool> {
    let result = sqlx::query("DELETE FROM user_history WHERE user_id = $1 AND episode_slug = $2")
        .bind(user_id)
        .bind(episode_slug)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Clear all watch history for a user
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
///
/// # Returns
/// * `Ok(count)` - Number of entries deleted
pub async fn clear_history(pool: &PgPool, user_id: i32) -> RepositoryResult<u64> {
    let result = sqlx::query("DELETE FROM user_history WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

// ============================================================================
// Verification Tokens Repository
// ============================================================================

/// Token types for verification
pub const TOKEN_TYPE_EMAIL_VERIFICATION: &str = "email_verification";
pub const TOKEN_TYPE_PASSWORD_RESET: &str = "password_reset";

/// Verification token data
#[derive(Debug, Clone)]
pub struct VerificationToken {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Create a verification token for a user
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
/// * `token` - Unique token string
/// * `token_type` - Type of token (email_verification or password_reset)
/// * `expires_in_hours` - Hours until token expires
///
/// # Returns
/// * `Ok(VerificationToken)` - The created token
pub async fn create_verification_token(
    pool: &PgPool,
    user_id: i32,
    token: &str,
    token_type: &str,
    expires_in_hours: i64,
) -> RepositoryResult<VerificationToken> {
    let expires_at = Utc::now() + chrono::Duration::hours(expires_in_hours);

    let row = sqlx::query(
        r#"
        INSERT INTO verification_tokens (user_id, token, token_type, expires_at, created_at)
        VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
        RETURNING id, user_id, token, token_type, expires_at, used_at, created_at
        "#,
    )
    .bind(user_id)
    .bind(token)
    .bind(token_type)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(VerificationToken {
        id: row.get("id"),
        user_id: row.get("user_id"),
        token: row.get("token"),
        token_type: row.get("token_type"),
        expires_at: row.get("expires_at"),
        used_at: row.get("used_at"),
        created_at: row.get("created_at"),
    })
}

/// Find a verification token by token string
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `token` - Token string to search for
///
/// # Returns
/// * `Ok(Some(VerificationToken))` - Token found
/// * `Ok(None)` - Token not found
pub async fn find_verification_token(
    pool: &PgPool,
    token: &str,
) -> RepositoryResult<Option<VerificationToken>> {
    let row = sqlx::query(
        r#"
        SELECT id, user_id, token, token_type, expires_at, used_at, created_at
        FROM verification_tokens
        WHERE token = $1
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => Ok(Some(VerificationToken {
            id: row.get("id"),
            user_id: row.get("user_id"),
            token: row.get("token"),
            token_type: row.get("token_type"),
            expires_at: row.get("expires_at"),
            used_at: row.get("used_at"),
            created_at: row.get("created_at"),
        })),
        None => Ok(None),
    }
}

/// Mark a verification token as used
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `token` - Token string to mark as used
///
/// # Returns
/// * `Ok(true)` - Token was marked as used
/// * `Ok(false)` - Token not found
pub async fn mark_token_as_used(pool: &PgPool, token: &str) -> RepositoryResult<bool> {
    let result = sqlx::query(
        r#"
        UPDATE verification_tokens
        SET used_at = CURRENT_TIMESTAMP
        WHERE token = $1 AND used_at IS NULL
        "#,
    )
    .bind(token)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Delete expired verification tokens
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Ok(count)` - Number of tokens deleted
pub async fn delete_expired_tokens(pool: &PgPool) -> RepositoryResult<u64> {
    let result = sqlx::query(
        r#"
        DELETE FROM verification_tokens
        WHERE expires_at < CURRENT_TIMESTAMP
        "#,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Delete all verification tokens for a user of a specific type
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
/// * `token_type` - Type of tokens to delete
///
/// # Returns
/// * `Ok(count)` - Number of tokens deleted
pub async fn delete_user_tokens(
    pool: &PgPool,
    user_id: i32,
    token_type: &str,
) -> RepositoryResult<u64> {
    let result = sqlx::query(
        r#"
        DELETE FROM verification_tokens
        WHERE user_id = $1 AND token_type = $2
        "#,
    )
    .bind(user_id)
    .bind(token_type)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Update user's email_verified status
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
/// * `verified` - Whether email is verified
///
/// # Returns
/// * `Ok(true)` - Status was updated
/// * `Ok(false)` - User not found
pub async fn set_email_verified(
    pool: &PgPool,
    user_id: i32,
    verified: bool,
) -> RepositoryResult<bool> {
    let result = sqlx::query(
        r#"
        UPDATE users
        SET email_verified = $1, updated_at = CURRENT_TIMESTAMP
        WHERE id = $2
        "#,
    )
    .bind(verified)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Check if user's email is verified
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
///
/// # Returns
/// * `Ok(Some(bool))` - Email verification status
/// * `Ok(None)` - User not found
pub async fn is_email_verified(pool: &PgPool, user_id: i32) -> RepositoryResult<Option<bool>> {
    let row = sqlx::query(
        r#"
        SELECT email_verified
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => Ok(Some(
            row.get::<Option<bool>, _>("email_verified")
                .unwrap_or(false),
        )),
        None => Ok(None),
    }
}

/// Update user's password
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - User ID
/// * `password_hash` - New bcrypt hashed password
///
/// # Returns
/// * `Ok(true)` - Password was updated
/// * `Ok(false)` - User not found
pub async fn update_user_password(
    pool: &PgPool,
    user_id: i32,
    password_hash: &str,
) -> RepositoryResult<bool> {
    let result = sqlx::query(
        r#"
        UPDATE users
        SET password_hash = $1, updated_at = CURRENT_TIMESTAMP
        WHERE id = $2
        "#,
    )
    .bind(password_hash)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
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
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        // Clean up first
        let _ = delete_all_anime_updates(&pool).await;

        // Create
        let updates = vec![
            create_test_anime_update("https://test.com/ep1"),
            create_test_anime_update("https://test.com/ep2"),
        ];
        save_anime_updates(&pool, &updates)
            .await
            .expect("Failed to save");

        // Read
        let fetched = get_anime_updates(&pool).await.expect("Failed to fetch");
        assert!(fetched.len() >= 2);

        // Update (upsert with same URL)
        let mut updated = create_test_anime_update("https://test.com/ep1");
        updated.title = "Updated Title".to_string();
        save_anime_updates(&pool, &[updated])
            .await
            .expect("Failed to update");

        // Verify update
        let fetched = get_anime_updates(&pool).await.expect("Failed to fetch");
        let found = fetched
            .iter()
            .find(|u| u.episode_url == "https://test.com/ep1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Updated Title");

        // Clean up
        delete_all_anime_updates(&pool)
            .await
            .expect("Failed to delete");
    }

    #[tokio::test]
    #[ignore]
    async fn test_completed_anime_crud() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        // Clean up first
        let _ = delete_all_completed_anime(&pool).await;

        // Create
        let anime_list = vec![
            create_test_completed_anime("https://test.com/anime1"),
            create_test_completed_anime("https://test.com/anime2"),
        ];
        save_completed_anime(&pool, &anime_list)
            .await
            .expect("Failed to save");

        // Read
        let fetched = get_completed_anime(&pool).await.expect("Failed to fetch");
        assert!(fetched.len() >= 2);

        // Clean up
        delete_all_completed_anime(&pool)
            .await
            .expect("Failed to delete");
    }

    #[tokio::test]
    #[ignore]
    async fn test_anime_detail_with_episodes_crud() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let slug = "test-anime-slug";

        // Clean up first
        let _ = delete_anime_detail(&pool, slug).await;

        // Create with episodes
        let detail = create_test_anime_detail();
        save_anime_detail_with_episodes(&pool, slug, &detail)
            .await
            .expect("Failed to save");

        // Read
        let fetched = get_anime_detail(&pool, slug)
            .await
            .expect("Failed to fetch");
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
        let fetched = get_anime_detail(&pool, slug)
            .await
            .expect("Failed to fetch");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().title, "Updated Anime Title");

        // Clean up
        delete_anime_detail(&pool, slug)
            .await
            .expect("Failed to delete");
    }

    #[tokio::test]
    #[ignore]
    async fn test_video_sources_crud() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

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
        let new_sources = vec![create_test_video_source("NEW_SERVER", "720p")];
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
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let cache_key = "test:cache:basic";

        // Clean up first
        let _ = delete_cache_entry(&pool, cache_key).await;

        // Test 1: Cache should not be valid when entry doesn't exist
        let is_valid = is_cache_valid(&pool, cache_key, DEFAULT_CACHE_TTL_MS)
            .await
            .expect("Failed to check cache");
        assert!(
            !is_valid,
            "Cache should not be valid when entry doesn't exist"
        );

        // Test 2: Get timestamp should return None when entry doesn't exist
        let timestamp = get_cache_timestamp(&pool, cache_key)
            .await
            .expect("Failed to get timestamp");
        assert!(
            timestamp.is_none(),
            "Timestamp should be None when entry doesn't exist"
        );

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
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

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
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

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
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

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
            assert!(
                !is_valid,
                "Cache entry should not exist for {} after deletion",
                key
            );
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
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let slug = "test-single-crawled";

        // Clean up first
        let _ = delete_crawled_anime(&pool, slug).await;

        // Create
        let anime = create_test_crawled_anime(slug);
        save_crawled_anime(&pool, &anime)
            .await
            .expect("Failed to save");

        // Read
        let fetched = get_crawled_anime_by_slug(&pool, slug)
            .await
            .expect("Failed to fetch");
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.slug, slug);
        assert_eq!(fetched.title, anime.title);

        // Clean up
        delete_crawled_anime(&pool, slug)
            .await
            .expect("Failed to delete");
    }

    #[tokio::test]
    #[ignore]
    async fn test_crawled_anime_batch_save() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let slugs = vec!["test-batch-1", "test-batch-2", "test-batch-3"];

        // Clean up first
        for slug in &slugs {
            let _ = delete_crawled_anime(&pool, slug).await;
        }

        // Create batch
        let anime_list: Vec<CrawledAnime> =
            slugs.iter().map(|s| create_test_crawled_anime(s)).collect();

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
            delete_crawled_anime(&pool, slug)
                .await
                .expect("Failed to delete");
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_crawled_anime_upsert() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let slug = "test-upsert-crawled";

        // Clean up first
        let _ = delete_crawled_anime(&pool, slug).await;

        // Create initial
        let anime = create_test_crawled_anime(slug);
        save_crawled_anime(&pool, &anime)
            .await
            .expect("Failed to save");

        // Update with same slug
        let mut updated_anime = create_test_crawled_anime(slug);
        updated_anime.title = "Updated Title".to_string();
        updated_anime.status = "Completed".to_string();
        save_crawled_anime(&pool, &updated_anime)
            .await
            .expect("Failed to upsert");

        // Verify update
        let fetched = get_crawled_anime_by_slug(&pool, slug)
            .await
            .expect("Failed to fetch")
            .expect("Should find anime");
        assert_eq!(fetched.title, "Updated Title");
        assert_eq!(fetched.status, "Completed");

        // Verify count is still 1 (not duplicated)
        let count_before = get_crawled_anime_count(&pool)
            .await
            .expect("Failed to count");
        save_crawled_anime(&pool, &updated_anime)
            .await
            .expect("Failed to upsert again");
        let count_after = get_crawled_anime_count(&pool)
            .await
            .expect("Failed to count");

        // Count should not increase (upsert, not insert)
        assert_eq!(
            count_before, count_after,
            "Count should not increase on upsert"
        );

        // Clean up
        delete_crawled_anime(&pool, slug)
            .await
            .expect("Failed to delete");
    }

    #[tokio::test]
    #[ignore]
    async fn test_crawled_anime_count() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        // Get initial count
        let initial_count = get_crawled_anime_count(&pool)
            .await
            .expect("Failed to count");

        // Add some anime
        let slugs = vec!["test-count-1", "test-count-2"];
        for slug in &slugs {
            let _ = delete_crawled_anime(&pool, slug).await;
            let anime = create_test_crawled_anime(slug);
            save_crawled_anime(&pool, &anime)
                .await
                .expect("Failed to save");
        }

        // Verify count increased
        let new_count = get_crawled_anime_count(&pool)
            .await
            .expect("Failed to count");
        assert_eq!(new_count, initial_count + 2);

        // Clean up
        for slug in &slugs {
            delete_crawled_anime(&pool, slug)
                .await
                .expect("Failed to delete");
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_crawled_anime_empty_batch() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        // Saving empty batch should succeed without error
        let empty_list: Vec<CrawledAnime> = vec![];
        save_crawled_anime_batch(&pool, &empty_list)
            .await
            .expect("Empty batch save should succeed");
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_user_with_email_password() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let email = "test_user_crud@example.com";

        // Clean up first
        if let Ok(Some((user, _))) = find_user_by_email(&pool, email).await {
            let _ = delete_user(&pool, user.id).await;
        }

        // Create user
        let user = create_user(&pool, email, "hashed_password", Some("Test User"))
            .await
            .expect("Failed to create user");

        assert_eq!(user.email, email);
        assert_eq!(user.name, Some("Test User".to_string()));
        assert!(user.id > 0);

        // Clean up
        delete_user(&pool, user.id)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_user_duplicate_email() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let email = "test_duplicate@example.com";

        // Clean up first
        if let Ok(Some((user, _))) = find_user_by_email(&pool, email).await {
            let _ = delete_user(&pool, user.id).await;
        }

        // Create first user
        let user = create_user(&pool, email, "hashed_password", None)
            .await
            .expect("Failed to create user");

        // Try to create duplicate
        let result = create_user(&pool, email, "another_password", None).await;
        assert!(matches!(result, Err(RepositoryError::EmailAlreadyExists)));

        // Clean up
        delete_user(&pool, user.id)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_user_by_email() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let email = "test_find_email@example.com";

        // Clean up first
        if let Ok(Some((user, _))) = find_user_by_email(&pool, email).await {
            let _ = delete_user(&pool, user.id).await;
        }

        // Create user
        let created_user = create_user(&pool, email, "hashed_password", Some("Find Me"))
            .await
            .expect("Failed to create user");

        // Find user
        let result = find_user_by_email(&pool, email)
            .await
            .expect("Failed to find user");

        assert!(result.is_some());
        let (found_user, password_hash) = result.unwrap();
        assert_eq!(found_user.email, email);
        assert_eq!(found_user.name, Some("Find Me".to_string()));
        assert_eq!(password_hash, Some("hashed_password".to_string()));

        // Find non-existent user
        let result = find_user_by_email(&pool, "nonexistent@example.com")
            .await
            .expect("Failed to query");
        assert!(result.is_none());

        // Clean up
        delete_user(&pool, created_user.id)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_google_user() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let email = "test_google@example.com";
        let google_id = "google_123456";

        // Clean up first
        if let Ok(Some((user, _))) = find_user_by_email(&pool, email).await {
            let _ = delete_user(&pool, user.id).await;
        }

        // Create Google user
        let user = create_google_user(
            &pool,
            email,
            google_id,
            "Google User",
            Some("https://example.com/avatar.jpg"),
        )
        .await
        .expect("Failed to create Google user");

        assert_eq!(user.email, email);
        assert_eq!(user.name, Some("Google User".to_string()));
        assert_eq!(
            user.avatar,
            Some("https://example.com/avatar.jpg".to_string())
        );

        // Find by Google ID
        let found = find_user_by_google_id(&pool, google_id)
            .await
            .expect("Failed to find user");
        assert!(found.is_some());
        assert_eq!(found.unwrap().email, email);

        // Clean up
        delete_user(&pool, user.id)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_user_by_id() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let email = "test_find_id@example.com";

        // Clean up first
        if let Ok(Some((user, _))) = find_user_by_email(&pool, email).await {
            let _ = delete_user(&pool, user.id).await;
        }

        // Create user
        let created_user = create_user(&pool, email, "hashed_password", None)
            .await
            .expect("Failed to create user");

        // Find by ID
        let found = find_user_by_id(&pool, created_user.id)
            .await
            .expect("Failed to find user");
        assert!(found.is_some());
        assert_eq!(found.unwrap().email, email);

        // Find non-existent ID
        let found = find_user_by_id(&pool, 999999)
            .await
            .expect("Failed to query");
        assert!(found.is_none());

        // Clean up
        delete_user(&pool, created_user.id)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    #[ignore]
    async fn test_link_google_account() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let email = "test_link_google@example.com";
        let google_id = "google_link_123";

        // Clean up first
        if let Ok(Some((user, _))) = find_user_by_email(&pool, email).await {
            let _ = delete_user(&pool, user.id).await;
        }

        // Create user with email/password
        let user = create_user(&pool, email, "hashed_password", None)
            .await
            .expect("Failed to create user");

        // Link Google account
        link_google_account(&pool, user.id, google_id)
            .await
            .expect("Failed to link Google account");

        // Verify link
        let found = find_user_by_google_id(&pool, google_id)
            .await
            .expect("Failed to find user");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, user.id);

        // Clean up
        delete_user(&pool, user.id)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    #[ignore]
    async fn test_favorites_crud() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let email = "test_favorites@example.com";

        // Clean up first
        if let Ok(Some((user, _))) = find_user_by_email(&pool, email).await {
            let _ = delete_user(&pool, user.id).await;
        }

        // Create user
        let user = create_user(&pool, email, "hashed_password", None)
            .await
            .expect("Failed to create user");

        // Add favorite
        let favorite = add_favorite(
            &pool,
            user.id,
            "naruto",
            "Naruto",
            "https://example.com/naruto.jpg",
        )
        .await
        .expect("Failed to add favorite");
        assert_eq!(favorite.anime_slug, "naruto");
        assert_eq!(favorite.anime_title, "Naruto");

        // Check is_favorite
        let is_fav = is_favorite(&pool, user.id, "naruto")
            .await
            .expect("Failed to check favorite");
        assert!(is_fav);

        let is_not_fav = is_favorite(&pool, user.id, "one-piece")
            .await
            .expect("Failed to check favorite");
        assert!(!is_not_fav);

        // Get favorites
        let favorites = get_favorites(&pool, user.id)
            .await
            .expect("Failed to get favorites");
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].anime_slug, "naruto");

        // Try to add duplicate
        let result = add_favorite(
            &pool,
            user.id,
            "naruto",
            "Naruto",
            "https://example.com/naruto.jpg",
        )
        .await;
        assert!(matches!(result, Err(RepositoryError::Conflict(_))));

        // Remove favorite
        let removed = remove_favorite(&pool, user.id, "naruto")
            .await
            .expect("Failed to remove favorite");
        assert!(removed);

        // Verify removed
        let is_fav = is_favorite(&pool, user.id, "naruto")
            .await
            .expect("Failed to check favorite");
        assert!(!is_fav);

        // Clean up
        delete_user(&pool, user.id)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    #[ignore]
    async fn test_subscriptions_crud() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let email = "test_subscriptions@example.com";

        // Clean up first
        if let Ok(Some((user, _))) = find_user_by_email(&pool, email).await {
            let _ = delete_user(&pool, user.id).await;
        }

        // Create user
        let user = create_user(&pool, email, "hashed_password", None)
            .await
            .expect("Failed to create user");

        // Add subscription
        let subscription = add_subscription(
            &pool,
            user.id,
            "one-piece",
            "One Piece",
            "https://example.com/onepiece.jpg",
        )
        .await
        .expect("Failed to add subscription");
        assert_eq!(subscription.anime_slug, "one-piece");
        assert_eq!(subscription.anime_title, "One Piece");

        // Check is_subscribed
        let is_sub = is_subscribed(&pool, user.id, "one-piece")
            .await
            .expect("Failed to check subscription");
        assert!(is_sub);

        let is_not_sub = is_subscribed(&pool, user.id, "naruto")
            .await
            .expect("Failed to check subscription");
        assert!(!is_not_sub);

        // Get subscriptions
        let subscriptions = get_subscriptions(&pool, user.id)
            .await
            .expect("Failed to get subscriptions");
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0].anime_slug, "one-piece");

        // Try to add duplicate
        let result = add_subscription(
            &pool,
            user.id,
            "one-piece",
            "One Piece",
            "https://example.com/onepiece.jpg",
        )
        .await;
        assert!(matches!(result, Err(RepositoryError::Conflict(_))));

        // Remove subscription
        let removed = remove_subscription(&pool, user.id, "one-piece")
            .await
            .expect("Failed to remove subscription");
        assert!(removed);

        // Verify removed
        let is_sub = is_subscribed(&pool, user.id, "one-piece")
            .await
            .expect("Failed to check subscription");
        assert!(!is_sub);

        // Clean up
        delete_user(&pool, user.id)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    #[ignore]
    async fn test_history_crud() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let email = "test_history@example.com";

        // Clean up first
        if let Ok(Some((user, _))) = find_user_by_email(&pool, email).await {
            let _ = delete_user(&pool, user.id).await;
        }

        // Create user
        let user = create_user(&pool, email, "hashed_password", None)
            .await
            .expect("Failed to create user");

        // Add to history
        let history = add_to_history(
            &pool,
            user.id,
            "naruto-ep-1",
            "naruto",
            "Episode 1",
            "Naruto",
            "https://example.com/naruto-ep1.jpg",
        )
        .await
        .expect("Failed to add to history");
        assert_eq!(history.episode_slug, "naruto-ep-1");
        assert_eq!(history.anime_slug, "naruto");

        // Get history
        let history_list = get_history(&pool, user.id)
            .await
            .expect("Failed to get history");
        assert_eq!(history_list.len(), 1);
        assert_eq!(history_list[0].episode_slug, "naruto-ep-1");

        // Remove from history
        let removed = remove_from_history(&pool, user.id, "naruto-ep-1")
            .await
            .expect("Failed to remove from history");
        assert!(removed);

        // Verify removed
        let history_list = get_history(&pool, user.id)
            .await
            .expect("Failed to get history");
        assert!(history_list.is_empty());

        // Clean up
        delete_user(&pool, user.id)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    #[ignore]
    async fn test_history_update_timestamp_on_rewatch() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let email = "test_history_rewatch@example.com";

        // Clean up first
        if let Ok(Some((user, _))) = find_user_by_email(&pool, email).await {
            let _ = delete_user(&pool, user.id).await;
        }

        // Create user
        let user = create_user(&pool, email, "hashed_password", None)
            .await
            .expect("Failed to create user");

        // Add to history first time
        let first_watch = add_to_history(
            &pool,
            user.id,
            "naruto-ep-1",
            "naruto",
            "Episode 1",
            "Naruto",
            "https://example.com/naruto-ep1.jpg",
        )
        .await
        .expect("Failed to add to history");

        // Wait a bit
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Re-watch (add again)
        let second_watch = add_to_history(
            &pool,
            user.id,
            "naruto-ep-1",
            "naruto",
            "Episode 1 - Updated",
            "Naruto",
            "https://example.com/naruto-ep1.jpg",
        )
        .await
        .expect("Failed to update history");

        // Timestamp should be updated
        assert!(second_watch.watched_at > first_watch.watched_at);
        // Title should be updated
        assert_eq!(second_watch.episode_title, "Episode 1 - Updated");

        // Should still be only one entry
        let history_list = get_history(&pool, user.id)
            .await
            .expect("Failed to get history");
        assert_eq!(history_list.len(), 1);

        // Clean up
        delete_user(&pool, user.id)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    #[ignore]
    async fn test_history_sorted_by_most_recent() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let email = "test_history_sort@example.com";

        // Clean up first
        if let Ok(Some((user, _))) = find_user_by_email(&pool, email).await {
            let _ = delete_user(&pool, user.id).await;
        }

        // Create user
        let user = create_user(&pool, email, "hashed_password", None)
            .await
            .expect("Failed to create user");

        // Add multiple episodes
        add_to_history(&pool, user.id, "ep-1", "anime", "Episode 1", "Anime", "")
            .await
            .expect("Failed to add");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        add_to_history(&pool, user.id, "ep-2", "anime", "Episode 2", "Anime", "")
            .await
            .expect("Failed to add");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        add_to_history(&pool, user.id, "ep-3", "anime", "Episode 3", "Anime", "")
            .await
            .expect("Failed to add");

        // Get history - should be sorted by most recent first
        let history_list = get_history(&pool, user.id)
            .await
            .expect("Failed to get history");

        assert_eq!(history_list.len(), 3);
        assert_eq!(history_list[0].episode_slug, "ep-3"); // Most recent
        assert_eq!(history_list[1].episode_slug, "ep-2");
        assert_eq!(history_list[2].episode_slug, "ep-1"); // Oldest

        // Clean up
        delete_user(&pool, user.id)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    #[ignore]
    async fn test_clear_history() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect");

        let email = "test_clear_history@example.com";

        // Clean up first
        if let Ok(Some((user, _))) = find_user_by_email(&pool, email).await {
            let _ = delete_user(&pool, user.id).await;
        }

        // Create user
        let user = create_user(&pool, email, "hashed_password", None)
            .await
            .expect("Failed to create user");

        // Add multiple episodes
        add_to_history(&pool, user.id, "ep-1", "anime", "Episode 1", "Anime", "")
            .await
            .expect("Failed to add");
        add_to_history(&pool, user.id, "ep-2", "anime", "Episode 2", "Anime", "")
            .await
            .expect("Failed to add");

        // Clear history
        let deleted = clear_history(&pool, user.id)
            .await
            .expect("Failed to clear history");
        assert_eq!(deleted, 2);

        // Verify cleared
        let history_list = get_history(&pool, user.id)
            .await
            .expect("Failed to get history");
        assert!(history_list.is_empty());

        // Clean up
        delete_user(&pool, user.id)
            .await
            .expect("Failed to delete user");
    }
}
