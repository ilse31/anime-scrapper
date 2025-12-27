//! User routes for the Anime Scraper API
//!
//! This module contains HTTP route handlers for user-specific endpoints:
//! - POST /api/favorites - Add anime to favorites
//! - GET /api/favorites - Get user's favorites
//! - DELETE /api/favorites/:slug - Remove from favorites
//! - POST /api/subscriptions - Subscribe to anime
//! - GET /api/subscriptions - Get user's subscriptions
//! - DELETE /api/subscriptions/:slug - Unsubscribe
//! - POST /api/history - Record watched episode
//! - GET /api/history - Get watch history
//! - DELETE /api/history/:slug - Remove from history

use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use tracing::{error, info};
use utoipa::ToSchema;

use crate::auth::Auth;
use crate::db::{
    add_favorite, add_subscription, add_to_history, get_favorites, get_history, get_subscriptions,
    remove_favorite, remove_from_history, remove_subscription, RepositoryError,
};
use crate::models::{ApiError, ApiResponse, UserFavorite, UserHistory, UserSubscription};
use crate::routes::AppState;

// ============================================================================
// Request Bodies
// ============================================================================

/// Request body for adding a favorite
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddFavoriteRequest {
    /// Anime slug identifier
    pub anime_slug: String,
    /// Anime title for display
    pub anime_title: String,
    /// Thumbnail image URL
    pub thumbnail: String,
}

/// Request body for adding a subscription
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddSubscriptionRequest {
    /// Anime slug identifier
    pub anime_slug: String,
    /// Anime title for display
    pub anime_title: String,
    /// Thumbnail image URL
    pub thumbnail: String,
}

/// Request body for adding to history
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddHistoryRequest {
    /// Episode slug identifier
    pub episode_slug: String,
    /// Parent anime slug
    pub anime_slug: String,
    /// Episode title for display
    pub episode_title: String,
    /// Anime title for display
    pub anime_title: String,
    /// Thumbnail image URL
    pub thumbnail: String,
}


/// POST /api/favorites - Add an anime to user's favorites
///
/// Requires authentication via JWT token in Authorization header.
///
/// # Request Body
/// - animeSlug: Unique identifier for the anime (required)
/// - animeTitle: Anime title for display (required)
/// - thumbnail: Thumbnail image URL (required)
///
/// # Responses
/// - 200: Favorite added successfully
/// - 400: Invalid request body
/// - 401: Not authenticated
/// - 409: Anime already in favorites
/// - 500: Internal server error
#[utoipa::path(
    post,
    path = "/api/favorites",
    tag = "user",
    request_body = AddFavoriteRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Favorite added successfully", body = ApiResponse<UserFavorite>),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 401, description = "Not authenticated", body = ApiError),
        (status = 409, description = "Already in favorites", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn add_favorite_handler(
    data: web::Data<AppState>,
    auth: Auth,
    body: web::Json<AddFavoriteRequest>,
) -> impl Responder {
    let pool = data.db.pool();

    // Validate required fields
    if body.anime_slug.is_empty() {
        return HttpResponse::BadRequest().json(ApiError::new("Anime slug is required"));
    }

    if body.anime_title.is_empty() {
        return HttpResponse::BadRequest().json(ApiError::new("Anime title is required"));
    }

    match add_favorite(
        pool,
        auth.user_id,
        &body.anime_slug,
        &body.anime_title,
        &body.thumbnail,
    )
    .await
    {
        Ok(favorite) => {
            info!("User {} added favorite: {}", auth.user_id, body.anime_slug);
            HttpResponse::Ok().json(ApiResponse::new(favorite))
        }
        Err(RepositoryError::Conflict(msg)) => HttpResponse::Conflict().json(ApiError::new(msg)),
        Err(e) => {
            error!("Failed to add favorite: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to add favorite"))
        }
    }
}

/// GET /api/favorites - Get user's favorite anime list
///
/// Requires authentication via JWT token in Authorization header.
///
/// # Responses
/// - 200: Returns list of favorites
/// - 401: Not authenticated
/// - 500: Internal server error
#[utoipa::path(
    get,
    path = "/api/favorites",
    tag = "user",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Favorites retrieved successfully", body = ApiResponse<Vec<UserFavorite>>),
        (status = 401, description = "Not authenticated", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_favorites_handler(data: web::Data<AppState>, auth: Auth) -> impl Responder {
    let pool = data.db.pool();

    match get_favorites(pool, auth.user_id).await {
        Ok(favorites) => HttpResponse::Ok().json(ApiResponse::new(favorites)),
        Err(e) => {
            error!("Failed to get favorites: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to get favorites"))
        }
    }
}

/// DELETE /api/favorites/{slug} - Remove an anime from user's favorites
///
/// Requires authentication via JWT token in Authorization header.
///
/// # Path Parameters
/// - slug: Anime slug to remove from favorites
///
/// # Responses
/// - 200: Favorite removed successfully
/// - 401: Not authenticated
/// - 404: Favorite not found
/// - 500: Internal server error
#[utoipa::path(
    delete,
    path = "/api/favorites/{slug}",
    tag = "user",
    params(
        ("slug" = String, Path, description = "Anime slug to remove from favorites")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Favorite removed successfully", body = ApiResponse<String>),
        (status = 401, description = "Not authenticated", body = ApiError),
        (status = 404, description = "Favorite not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn remove_favorite_handler(
    data: web::Data<AppState>,
    auth: Auth,
    path: web::Path<String>,
) -> impl Responder {
    let pool = data.db.pool();
    let anime_slug = path.into_inner();

    match remove_favorite(pool, auth.user_id, &anime_slug).await {
        Ok(true) => {
            info!("User {} removed favorite: {}", auth.user_id, anime_slug);
            HttpResponse::Ok().json(ApiResponse::new(
                "Favorite removed successfully".to_string(),
            ))
        }
        Ok(false) => HttpResponse::NotFound().json(ApiError::new("Favorite not found")),
        Err(e) => {
            error!("Failed to remove favorite: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to remove favorite"))
        }
    }
}


/// POST /api/subscriptions - Subscribe to an anime series
///
/// Requires authentication via JWT token in Authorization header.
///
/// # Request Body
/// - animeSlug: Unique identifier for the anime (required)
/// - animeTitle: Anime title for display (required)
/// - thumbnail: Thumbnail image URL (required)
///
/// # Responses
/// - 200: Subscription added successfully
/// - 400: Invalid request body
/// - 401: Not authenticated
/// - 409: Already subscribed
/// - 500: Internal server error
#[utoipa::path(
    post,
    path = "/api/subscriptions",
    tag = "user",
    request_body = AddSubscriptionRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Subscription added successfully", body = ApiResponse<UserSubscription>),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 401, description = "Not authenticated", body = ApiError),
        (status = 409, description = "Already subscribed", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn add_subscription_handler(
    data: web::Data<AppState>,
    auth: Auth,
    body: web::Json<AddSubscriptionRequest>,
) -> impl Responder {
    let pool = data.db.pool();

    // Validate required fields
    if body.anime_slug.is_empty() {
        return HttpResponse::BadRequest().json(ApiError::new("Anime slug is required"));
    }

    if body.anime_title.is_empty() {
        return HttpResponse::BadRequest().json(ApiError::new("Anime title is required"));
    }

    match add_subscription(
        pool,
        auth.user_id,
        &body.anime_slug,
        &body.anime_title,
        &body.thumbnail,
    )
    .await
    {
        Ok(subscription) => {
            info!("User {} subscribed to: {}", auth.user_id, body.anime_slug);
            HttpResponse::Ok().json(ApiResponse::new(subscription))
        }
        Err(RepositoryError::Conflict(msg)) => HttpResponse::Conflict().json(ApiError::new(msg)),
        Err(e) => {
            error!("Failed to add subscription: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to add subscription"))
        }
    }
}

/// GET /api/subscriptions - Get user's subscriptions
///
/// Requires authentication via JWT token in Authorization header.
///
/// # Responses
/// - 200: Returns list of subscriptions
/// - 401: Not authenticated
/// - 500: Internal server error
#[utoipa::path(
    get,
    path = "/api/subscriptions",
    tag = "user",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Subscriptions retrieved successfully", body = ApiResponse<Vec<UserSubscription>>),
        (status = 401, description = "Not authenticated", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_subscriptions_handler(data: web::Data<AppState>, auth: Auth) -> impl Responder {
    let pool = data.db.pool();

    match get_subscriptions(pool, auth.user_id).await {
        Ok(subscriptions) => HttpResponse::Ok().json(ApiResponse::new(subscriptions)),
        Err(e) => {
            error!("Failed to get subscriptions: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to get subscriptions"))
        }
    }
}

/// DELETE /api/subscriptions/{slug} - Unsubscribe from an anime series
///
/// Requires authentication via JWT token in Authorization header.
///
/// # Path Parameters
/// - slug: Anime slug to unsubscribe from
///
/// # Responses
/// - 200: Unsubscribed successfully
/// - 401: Not authenticated
/// - 404: Subscription not found
/// - 500: Internal server error
#[utoipa::path(
    delete,
    path = "/api/subscriptions/{slug}",
    tag = "user",
    params(
        ("slug" = String, Path, description = "Anime slug to unsubscribe from")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Unsubscribed successfully", body = ApiResponse<String>),
        (status = 401, description = "Not authenticated", body = ApiError),
        (status = 404, description = "Subscription not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn remove_subscription_handler(
    data: web::Data<AppState>,
    auth: Auth,
    path: web::Path<String>,
) -> impl Responder {
    let pool = data.db.pool();
    let anime_slug = path.into_inner();

    match remove_subscription(pool, auth.user_id, &anime_slug).await {
        Ok(true) => {
            info!("User {} unsubscribed from: {}", auth.user_id, anime_slug);
            HttpResponse::Ok().json(ApiResponse::new("Unsubscribed successfully".to_string()))
        }
        Ok(false) => HttpResponse::NotFound().json(ApiError::new("Subscription not found")),
        Err(e) => {
            error!("Failed to remove subscription: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to remove subscription"))
        }
    }
}


/// POST /api/history - Record a watched episode
///
/// Requires authentication via JWT token in Authorization header.
/// If the episode already exists in history, updates the watched_at timestamp.
///
/// # Request Body
/// - episodeSlug: Unique identifier for the episode (required)
/// - animeSlug: Parent anime slug (required)
/// - episodeTitle: Episode title for display (required)
/// - animeTitle: Anime title for display (required)
/// - thumbnail: Thumbnail image URL (required)
///
/// # Responses
/// - 200: History entry added/updated successfully
/// - 400: Invalid request body
/// - 401: Not authenticated
/// - 500: Internal server error
#[utoipa::path(
    post,
    path = "/api/history",
    tag = "user",
    request_body = AddHistoryRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "History entry added successfully", body = ApiResponse<UserHistory>),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 401, description = "Not authenticated", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn add_history_handler(
    data: web::Data<AppState>,
    auth: Auth,
    body: web::Json<AddHistoryRequest>,
) -> impl Responder {
    let pool = data.db.pool();

    // Validate required fields
    if body.episode_slug.is_empty() {
        return HttpResponse::BadRequest().json(ApiError::new("Episode slug is required"));
    }

    if body.anime_slug.is_empty() {
        return HttpResponse::BadRequest().json(ApiError::new("Anime slug is required"));
    }

    match add_to_history(
        pool,
        auth.user_id,
        &body.episode_slug,
        &body.anime_slug,
        &body.episode_title,
        &body.anime_title,
        &body.thumbnail,
    )
    .await
    {
        Ok(history) => {
            info!(
                "User {} recorded history: {}",
                auth.user_id, body.episode_slug
            );
            HttpResponse::Ok().json(ApiResponse::new(history))
        }
        Err(e) => {
            error!("Failed to add history: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to record watch history"))
        }
    }
}

/// GET /api/history - Get user's watch history
///
/// Requires authentication via JWT token in Authorization header.
/// Returns history sorted by most recently watched first.
///
/// # Responses
/// - 200: Returns list of history entries
/// - 401: Not authenticated
/// - 500: Internal server error
#[utoipa::path(
    get,
    path = "/api/history",
    tag = "user",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "History retrieved successfully", body = ApiResponse<Vec<UserHistory>>),
        (status = 401, description = "Not authenticated", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_history_handler(data: web::Data<AppState>, auth: Auth) -> impl Responder {
    let pool = data.db.pool();

    match get_history(pool, auth.user_id).await {
        Ok(history) => HttpResponse::Ok().json(ApiResponse::new(history)),
        Err(e) => {
            error!("Failed to get history: {}", e);
            HttpResponse::InternalServerError().json(ApiError::new("Failed to get watch history"))
        }
    }
}

/// DELETE /api/history/{slug} - Remove an episode from user's watch history
///
/// Requires authentication via JWT token in Authorization header.
///
/// # Path Parameters
/// - slug: Episode slug to remove from history
///
/// # Responses
/// - 200: History entry removed successfully
/// - 401: Not authenticated
/// - 404: History entry not found
/// - 500: Internal server error
#[utoipa::path(
    delete,
    path = "/api/history/{slug}",
    tag = "user",
    params(
        ("slug" = String, Path, description = "Episode slug to remove from history")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "History entry removed successfully", body = ApiResponse<String>),
        (status = 401, description = "Not authenticated", body = ApiError),
        (status = 404, description = "History entry not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn remove_history_handler(
    data: web::Data<AppState>,
    auth: Auth,
    path: web::Path<String>,
) -> impl Responder {
    let pool = data.db.pool();
    let episode_slug = path.into_inner();

    match remove_from_history(pool, auth.user_id, &episode_slug).await {
        Ok(true) => {
            info!("User {} removed history: {}", auth.user_id, episode_slug);
            HttpResponse::Ok().json(ApiResponse::new(
                "History entry removed successfully".to_string(),
            ))
        }
        Ok(false) => HttpResponse::NotFound().json(ApiError::new("History entry not found")),
        Err(e) => {
            error!("Failed to remove history: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::new("Failed to remove history entry"))
        }
    }
}

/// Configure user routes (favorites, subscriptions, history)
pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            // Favorites
            .route("/favorites", web::post().to(add_favorite_handler))
            .route("/favorites", web::get().to(get_favorites_handler))
            .route(
                "/favorites/{slug}",
                web::delete().to(remove_favorite_handler),
            )
            // Subscriptions
            .route("/subscriptions", web::post().to(add_subscription_handler))
            .route("/subscriptions", web::get().to(get_subscriptions_handler))
            .route(
                "/subscriptions/{slug}",
                web::delete().to(remove_subscription_handler),
            )
            // History
            .route("/history", web::post().to(add_history_handler))
            .route("/history", web::get().to(get_history_handler))
            .route("/history/{slug}", web::delete().to(remove_history_handler)),
    );
}
