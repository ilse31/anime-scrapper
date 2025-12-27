//! API Routes module for the Anime Scraper API
//!
//! This module contains all HTTP route handlers for the public API endpoints.

use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use tracing::{error, info, warn};
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::config::Config;
use crate::constants::endpoints;
use crate::db::{
    get_anime_detail, get_anime_updates, get_completed_anime,
    is_cache_valid, save_anime_detail_with_episodes, save_anime_updates, save_completed_anime,
    save_crawled_anime_batch, save_video_sources, update_cache_timestamp,
    Database, DEFAULT_CACHE_TTL_MS,
};
use crate::models::{
    AnimeListFilters, AnimeListResponse, ApiError, ApiResponse, CrawledAnime, CrawledAnimeRecord,
    CrawlerData, CrawlerResponse, AuthResponse, AuthData, User, UserFavorite, UserHistory,
    UserSubscription, RegisterRequest, LoginRequest, GoogleAuthRequest,
};
use crate::parser::{
    parse_anime_detail, parse_anime_list, parse_anime_updates, parse_completed_anime,
    parse_episode_detail, parse_search_results, AnimeDetail, AnimeListItem, AnimeUpdate,
    CompletedAnime, Episode, EpisodeDetail, SearchResult, VideoSource,
};
use crate::scraper::Scraper;

/// Application state shared across handlers
pub struct AppState {
    pub db: Database,
    pub config: Config,
}

/// Cache keys for different data types
mod cache_keys {
    pub const UPDATES: &str = "updates";
    pub const COMPLETED: &str = "completed";
    
    pub fn anime_detail(slug: &str) -> String {
        format!("anime:{}", slug)
    }
}

/// GET /api/updates - Get latest anime updates
///
/// Returns cached data if fresh (< 1 hour old), otherwise scrapes fresh data.
#[utoipa::path(
    get,
    path = "/api/updates",
    tag = "anime",
    responses(
        (status = 200, description = "Latest anime updates retrieved successfully", body = Vec<AnimeUpdate>),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_updates(data: web::Data<AppState>) -> impl Responder {
    let pool = data.db.pool();
    
    match is_cache_valid(pool, cache_keys::UPDATES, DEFAULT_CACHE_TTL_MS).await {
        Ok(true) => {
            info!("Returning cached anime updates");
            match get_anime_updates(pool).await {
                Ok(updates) if !updates.is_empty() => {
                    HttpResponse::Ok().json(ApiResponse::new(updates))
                }
                Ok(_) => {
                    info!("Cache valid but database empty, scraping fresh data");
                    scrape_and_return_updates(&data).await
                }
                Err(e) => {
                    error!("Failed to get cached anime updates: {}", e);
                    HttpResponse::InternalServerError()
                        .json(ApiError::new(format!("Database error: {}", e)))
                }
            }
        }
        Ok(false) => {
            info!("Cache stale, scraping fresh anime updates");
            scrape_and_return_updates(&data).await
        }
        Err(e) => {
            error!("Failed to check cache validity: {}", e);
            scrape_and_return_updates(&data).await
        }
    }
}

/// Helper function to scrape and return anime updates
async fn scrape_and_return_updates(data: &web::Data<AppState>) -> HttpResponse {
    let pool = data.db.pool();
    let scraper = Scraper::new();
    let url = endpoints::home(&data.config.base_url);
    info!("Fetching URL: {}", url);
    
    match scraper.fetch_page(&url).await {
        Ok(result) => {
            info!("Fetched {} bytes of HTML", result.html.len());
            
            let updates = parse_anime_updates(&result.html);
            info!("Parsed {} anime updates", updates.len());

            if let Err(e) = save_anime_updates(pool, &updates).await {
                error!("Failed to save anime updates: {}", e);
            }

            if let Err(e) = update_cache_timestamp(pool, cache_keys::UPDATES).await {
                error!("Failed to update cache timestamp: {}", e);
            }
            
            HttpResponse::Ok().json(ApiResponse::new(updates))
        }
        Err(e) => {
            error!("Failed to scrape anime updates: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new(format!("Failed to fetch data: {}", e)))
        }
    }
}

/// GET /api/completed - Get completed anime list
///
/// Returns cached data if fresh (< 1 hour old), otherwise scrapes fresh data.
#[utoipa::path(
    get,
    path = "/api/completed",
    tag = "anime",
    responses(
        (status = 200, description = "Completed anime list retrieved successfully", body = Vec<CompletedAnime>),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_completed(data: web::Data<AppState>) -> impl Responder {
    let pool = data.db.pool();
    
    match is_cache_valid(pool, cache_keys::COMPLETED, DEFAULT_CACHE_TTL_MS).await {
        Ok(true) => {
            info!("Returning cached completed anime");
            match get_completed_anime(pool).await {
                Ok(completed) if !completed.is_empty() => {
                    HttpResponse::Ok().json(ApiResponse::new(completed))
                }
                Ok(_) => {
                    info!("Cache valid but database empty, scraping fresh data");
                    scrape_and_return_completed(&data).await
                }
                Err(e) => {
                    error!("Failed to get cached completed anime: {}", e);
                    HttpResponse::InternalServerError()
                        .json(ApiError::new(format!("Database error: {}", e)))
                }
            }
        }
        Ok(false) => {
            info!("Cache stale, scraping fresh completed anime");
            scrape_and_return_completed(&data).await
        }
        Err(e) => {
            error!("Failed to check cache validity: {}", e);
            scrape_and_return_completed(&data).await
        }
    }
}

/// Helper function to scrape and return completed anime
async fn scrape_and_return_completed(data: &web::Data<AppState>) -> HttpResponse {
    let pool = data.db.pool();
    let scraper = Scraper::new();

    match scraper.fetch_page(&endpoints::home(&data.config.base_url)).await {
        Ok(result) => {
            let completed = parse_completed_anime(&result.html);
            info!("Parsed {} completed anime", completed.len());

            if let Err(e) = save_completed_anime(pool, &completed).await {
                error!("Failed to save completed anime: {}", e);
            }

            if let Err(e) = update_cache_timestamp(pool, cache_keys::COMPLETED).await {
                error!("Failed to update cache timestamp: {}", e);
            }
            
            HttpResponse::Ok().json(ApiResponse::new(completed))
        }
        Err(e) => {
            error!("Failed to scrape completed anime: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new(format!("Failed to fetch data: {}", e)))
        }
    }
}

/// Query parameters for search endpoint
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct SearchQuery {
    /// Search keyword
    pub q: Option<String>,
}

/// GET /api/search - Search for anime
///
/// Query parameter: q (required) - search keyword
#[utoipa::path(
    get,
    path = "/api/search",
    tag = "anime",
    params(SearchQuery),
    responses(
        (status = 200, description = "Search results retrieved successfully", body = Vec<SearchResult>),
        (status = 400, description = "Bad request - search query is required", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn search_anime(
    data: web::Data<AppState>,
    query: web::Query<SearchQuery>,
) -> impl Responder {
    let keyword = match &query.q {
        Some(q) if !q.trim().is_empty() => q.trim(),
        _ => {
            return HttpResponse::BadRequest()
                .json(ApiError::new("Search query is required"));
        }
    };

    info!("Searching for anime: {}", keyword);
    let scraper = Scraper::new();

    match scraper.fetch_page(&endpoints::search(&data.config.base_url, keyword)).await {
        Ok(result) => {
            let results = parse_search_results(&result.html);
            HttpResponse::Ok().json(ApiResponse::new(results))
        }
        Err(e) => {
            error!("Failed to search anime: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new(format!("Failed to fetch data: {}", e)))
        }
    }
}

/// Query parameters for anime list endpoint
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct AnimeListQuery {
    /// Page number (default: 1)
    pub page: Option<u32>,
    /// Anime type filter (TV, OVA, Movie, etc.)
    #[serde(rename = "type")]
    pub anime_type: Option<String>,
    /// Status filter (Ongoing, Completed, etc.)
    pub status: Option<String>,
    /// Sort order (title, titlereverse, update, latest, popular, rating)
    pub order: Option<String>,
}

/// GET /api/anime/list - Get anime list with filters
///
/// Query parameters:
/// - page: Page number (default: 1)
/// - type: Anime type filter (TV, OVA, Movie, etc.)
/// - status: Status filter (Ongoing, Completed, etc.)
/// - order: Sort order (title, titlereverse, update, latest, popular, rating)
#[utoipa::path(
    get,
    path = "/api/anime/list",
    tag = "anime",
    params(AnimeListQuery),
    responses(
        (status = 200, description = "Anime list retrieved successfully", body = AnimeListResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_anime_list(
    data: web::Data<AppState>,
    query: web::Query<AnimeListQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let anime_type = query.anime_type.as_deref().unwrap_or("");
    let status = query.status.as_deref().unwrap_or("");
    let order = query.order.as_deref().unwrap_or("");

    info!(
        "Fetching anime list: page={}, type={}, status={}, order={}",
        page, anime_type, status, order
    );

    let scraper = Scraper::new();
    let url = endpoints::anime_list(&data.config.base_url, page, anime_type, status, order);
    
    match scraper.fetch_page(&url).await {
        Ok(result) => {
            let items = parse_anime_list(&result.html);
            
            let response = AnimeListResponse {
                items,
                page: page as i32,
                filters: AnimeListFilters {
                    anime_type: anime_type.to_string(),
                    status: status.to_string(),
                    order: order.to_string(),
                },
            };
            
            HttpResponse::Ok().json(ApiResponse::new(response))
        }
        Err(e) => {
            error!("Failed to fetch anime list: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new(format!("Failed to fetch data: {}", e)))
        }
    }
}

/// GET /api/anime/{slug} - Get anime detail with episodes
///
/// Returns cached data if fresh (< 1 hour old), otherwise scrapes fresh data.
#[utoipa::path(
    get,
    path = "/api/anime/{slug}",
    tag = "anime",
    params(
        ("slug" = String, Path, description = "Anime slug identifier")
    ),
    responses(
        (status = 200, description = "Anime detail retrieved successfully", body = AnimeDetail),
        (status = 404, description = "Anime not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_anime_by_slug(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let slug = path.into_inner();
    let pool = data.db.pool();
    let cache_key = cache_keys::anime_detail(&slug);
    
    match is_cache_valid(pool, &cache_key, DEFAULT_CACHE_TTL_MS).await {
        Ok(true) => {
            info!("Returning cached anime detail for: {}", slug);
            match get_anime_detail(pool, &slug).await {
                Ok(Some(detail)) => HttpResponse::Ok().json(ApiResponse::new(detail)),
                Ok(None) => {
                    scrape_and_save_anime_detail(&data, &slug).await
                }
                Err(e) => {
                    error!("Failed to get cached anime detail: {}", e);
                    HttpResponse::InternalServerError()
                        .json(ApiError::new(format!("Database error: {}", e)))
                }
            }
        }
        Ok(false) => {
            scrape_and_save_anime_detail(&data, &slug).await
        }
        Err(e) => {
            error!("Failed to check cache validity: {}", e);
            scrape_anime_detail_only(&data, &slug).await
        }
    }
}

/// Helper function to scrape and save anime detail
async fn scrape_and_save_anime_detail(data: &web::Data<AppState>, slug: &str) -> HttpResponse {
    info!("Scraping fresh anime detail for: {}", slug);
    let scraper = Scraper::new();
    let pool = data.db.pool();
    let cache_key = cache_keys::anime_detail(slug);

    match scraper.fetch_page(&endpoints::anime(&data.config.base_url, slug)).await {
        Ok(result) => {
            let detail = parse_anime_detail(&result.html);

            if detail.title.is_empty() {
                return HttpResponse::NotFound()
                    .json(ApiError::new("Anime not found"));
            }

            if let Err(e) = save_anime_detail_with_episodes(pool, slug, &detail).await {
                error!("Failed to save anime detail: {}", e);
            }

            if let Err(e) = update_cache_timestamp(pool, &cache_key).await {
                error!("Failed to update cache timestamp: {}", e);
            }
            
            HttpResponse::Ok().json(ApiResponse::new(detail))
        }
        Err(e) => {
            error!("Failed to scrape anime detail: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new(format!("Failed to fetch data: {}", e)))
        }
    }
}

/// Helper function to scrape anime detail without saving (fallback)
async fn scrape_anime_detail_only(data: &web::Data<AppState>, slug: &str) -> HttpResponse {
    let scraper = Scraper::new();

    match scraper.fetch_page(&endpoints::anime(&data.config.base_url, slug)).await {
        Ok(result) => {
            let detail = parse_anime_detail(&result.html);
            
            if detail.title.is_empty() {
                return HttpResponse::NotFound()
                    .json(ApiError::new("Anime not found"));
            }
            
            HttpResponse::Ok().json(ApiResponse::new(detail))
        }
        Err(e) => {
            error!("Failed to scrape anime detail: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new(format!("Failed to fetch data: {}", e)))
        }
    }
}

/// GET /api/episode/{slug} - Get episode video sources
///
/// Scrapes the episode page and returns video sources.
#[utoipa::path(
    get,
    path = "/api/episode/{slug}",
    tag = "anime",
    params(
        ("slug" = String, Path, description = "Episode slug identifier")
    ),
    responses(
        (status = 200, description = "Episode detail with video sources retrieved successfully", body = EpisodeDetail),
        (status = 404, description = "Episode not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_episode_by_slug(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let slug = path.into_inner();
    let pool = data.db.pool();
    
    info!("Fetching episode: {}", slug);
    let scraper = Scraper::new();
    let url = endpoints::episode(&data.config.base_url, &slug);
    
    match scraper.fetch_page(&url).await {
        Ok(result) => {
            let episode_detail = parse_episode_detail(&result.html);

            if episode_detail.title.is_empty() && episode_detail.sources.is_empty() {
                return HttpResponse::NotFound()
                    .json(ApiError::new("Episode not found"));
            }

            if !episode_detail.sources.is_empty() {
                if let Err(e) = save_video_sources(pool, &url, &episode_detail.sources).await {
                    error!("Failed to save video sources: {}", e);
                }
            }
            
            HttpResponse::Ok().json(ApiResponse::new(episode_detail))
        }
        Err(e) => {
            error!("Failed to fetch episode: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new(format!("Failed to fetch data: {}", e)))
        }
    }
}

/// Helper function to extract slug from URL
fn extract_slug_from_url(url: &str) -> String {
    url.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string()
}

/// POST /api/crawler/run - Start bulk crawling all anime pages
///
/// Iterates through all anime list pages, scrapes metadata, anime details,
/// episodes, and video sources. Saves everything to the database.
#[utoipa::path(
    post,
    path = "/api/crawler/run",
    tag = "crawler",
    responses(
        (status = 200, description = "Crawler completed successfully", body = CrawlerResponse),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn run_crawler(data: web::Data<AppState>) -> impl Responder {
    info!("Starting bulk crawler");
    let pool = data.db.pool();
    let scraper = Scraper::new();
    
    let mut total_crawled: i32 = 0;
    let mut total_episodes: i32 = 0;
    let mut total_video_sources: i32 = 0;
    let mut pages_processed: i32 = 0;
    let mut errors: Vec<String> = Vec::new();
    
    let mut page: u32 = 1;
    
    loop {
        info!("Crawling page {}", page);
        let url = endpoints::anime_list(&data.config.base_url, page, "", "", "");

        let anime_list = match scraper.fetch_page(&url).await {
            Ok(result) => {
                let items = parse_anime_list(&result.html);
                if items.is_empty() {
                    info!("No more anime found on page {}, stopping crawler", page);
                    break;
                }
                items
            }
            Err(e) => {
                let error_msg = format!("Failed to fetch page {}: {}", page, e);
                error!("{}", error_msg);
                errors.push(error_msg);
                page += 1;
                if page > 1000 {
                    break;
                }
                continue;
            }
        };
        
        pages_processed += 1;

        let crawled_anime: Vec<CrawledAnime> = anime_list
            .iter()
            .map(|item| CrawledAnime {
                slug: extract_slug_from_url(&item.url),
                title: item.title.clone(),
                url: item.url.clone(),
                thumbnail: item.thumbnail.clone(),
                status: item.status.clone(),
                anime_type: item.anime_type.clone(),
                episode_status: item.episode_status.clone(),
            })
            .collect();

        if let Err(e) = save_crawled_anime_batch(pool, &crawled_anime).await {
            let error_msg = format!("Failed to save crawled anime batch on page {}: {}", page, e);
            error!("{}", error_msg);
            errors.push(error_msg);
        } else {
            total_crawled += crawled_anime.len() as i32;
        }

        for anime in &crawled_anime {
            let slug = &anime.slug;

            let detail = match scraper.fetch_page(&endpoints::anime(&data.config.base_url, slug)).await {
                Ok(result) => {
                    let detail = parse_anime_detail(&result.html);
                    if detail.title.is_empty() {
                        warn!("Empty anime detail for slug: {}", slug);
                        continue;
                    }
                    detail
                }
                Err(e) => {
                    let error_msg = format!("Failed to fetch anime detail for {}: {}", slug, e);
                    warn!("{}", error_msg);
                    errors.push(error_msg);
                    continue;
                }
            };

            if let Err(e) = save_anime_detail_with_episodes(pool, slug, &detail).await {
                let error_msg = format!("Failed to save anime detail for {}: {}", slug, e);
                warn!("{}", error_msg);
                errors.push(error_msg);
            } else {
                total_episodes += detail.episodes.len() as i32;
            }

            for episode in &detail.episodes {
                let episode_slug = extract_slug_from_url(&episode.url);
                let episode_url = endpoints::episode(&data.config.base_url, &episode_slug);
                
                match scraper.fetch_page(&episode_url).await {
                    Ok(result) => {
                        let episode_detail = parse_episode_detail(&result.html);
                        
                        if !episode_detail.sources.is_empty() {
                            if let Err(e) = save_video_sources(pool, &episode.url, &episode_detail.sources).await {
                                let error_msg = format!("Failed to save video sources for {}: {}", episode_slug, e);
                                warn!("{}", error_msg);
                                errors.push(error_msg);
                            } else {
                                total_video_sources += episode_detail.sources.len() as i32;
                            }
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to fetch episode {}: {}", episode_slug, e);
                        warn!("{}", error_msg);
                        errors.push(error_msg);
                        continue;
                    }
                }
            }
        }
        
        page += 1;

        if page > 1000 {
            info!("Reached page limit (1000), stopping crawler");
            break;
        }
    }
    
    info!(
        "Crawler completed: {} anime, {} episodes, {} video sources, {} pages",
        total_crawled, total_episodes, total_video_sources, pages_processed
    );
    
    HttpResponse::Ok().json(CrawlerResponse::new(
        total_crawled,
        total_episodes,
        total_video_sources,
        pages_processed,
        errors,
    ))
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Anime Scraper API",
        version = "0.1.0",
        description = "API for scraping and accessing anime data from sokuja.uk",
        contact(
            name = "API Support",
            url = "https://github.com/yourusername/anime-scraper"
        ),
        license(
            name = "MIT"
        )
    ),
    paths(
        get_updates,
        get_completed,
        search_anime,
        get_anime_list,
        get_anime_by_slug,
        get_episode_by_slug,
        run_crawler
    ),
    components(
        schemas(
            AnimeUpdate,
            SearchResult,
            AnimeListItem,
            Episode,
            VideoSource,
            EpisodeDetail,
            AnimeDetail,
            CompletedAnime,
            UserFavorite,
            UserSubscription,
            UserHistory,
            User,
            RegisterRequest,
            LoginRequest,
            GoogleAuthRequest,
            AuthResponse,
            AuthData,
            ApiError,
            AnimeListResponse,
            AnimeListFilters,
            CrawledAnime,
            CrawledAnimeRecord,
            CrawlerResponse,
            CrawlerData,
            SearchQuery,
            AnimeListQuery
        )
    ),
    tags(
        (name = "anime", description = "Anime data endpoints"),
        (name = "crawler", description = "Bulk crawling operations")
    )
)]
pub struct ApiDoc;

/// Configure API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/updates", web::get().to(get_updates))
            .route("/completed", web::get().to(get_completed))
            .route("/search", web::get().to(search_anime))
            .route("/anime/list", web::get().to(get_anime_list))
            .route("/anime/{slug}", web::get().to(get_anime_by_slug))
            .route("/episode/{slug}", web::get().to(get_episode_by_slug))
            .route("/crawler/run", web::post().to(run_crawler))
    );
}
