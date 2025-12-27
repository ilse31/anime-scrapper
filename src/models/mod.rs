//! Data models for the Anime Scraper API
//!
//! This module contains all data structures used throughout the application,
//! including user-related models, API responses, and crawler data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Re-export parser models for convenience
pub use crate::parser::{
    AnimeDetail, AnimeListItem, AnimeUpdate, CompletedAnime, Episode, EpisodeDetail, SearchResult,
    VideoSource,
};

/// Represents a user's favorite anime
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserFavorite {
    /// Unique identifier for the anime
    pub anime_slug: String,
    /// Anime title for display
    pub anime_title: String,
    /// Thumbnail image URL
    pub thumbnail: String,
    /// ISO timestamp when added to favorites
    pub created_at: String,
}

/// Represents a user's subscription to an anime series
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserSubscription {
    /// Unique identifier for the anime
    pub anime_slug: String,
    /// Anime title for display
    pub anime_title: String,
    /// Thumbnail image URL
    pub thumbnail: String,
    /// ISO timestamp when subscribed
    pub created_at: String,
}

/// Represents a user's watch history entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserHistory {
    /// Unique identifier for the episode
    pub episode_slug: String,
    /// Parent anime slug
    pub anime_slug: String,
    /// Episode title for display
    pub episode_title: String,
    /// Anime title for display
    pub anime_title: String,
    /// Thumbnail image URL
    pub thumbnail: String,
    /// ISO timestamp of last watch
    pub watched_at: String,
}

/// Represents a user account
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// User ID
    pub id: i32,
    /// User email address
    pub email: String,
    /// User display name (optional)
    pub name: Option<String>,
    /// User avatar URL (optional)
    pub avatar: Option<String>,
    /// ISO timestamp when account was created
    pub created_at: String,
}

/// Request body for user registration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    /// User email address
    pub email: String,
    /// User password
    pub password: String,
    /// Optional display name
    pub name: Option<String>,
}

/// Request body for user login
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    /// User email address
    pub email: String,
    /// User password
    pub password: String,
}

/// Request body for Google OAuth authentication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GoogleAuthRequest {
    /// Google ID token from client
    pub id_token: String,
}

/// Response for authentication endpoints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Response data containing user and token
    pub data: AuthData,
    /// ISO timestamp of the response
    pub timestamp: String,
}

/// Authentication data containing user info and JWT token
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthData {
    /// User information
    pub user: User,
    /// JWT access token
    pub token: String,
}

/// Generic API response wrapper for successful responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    /// Whether the operation was successful (always true for this type)
    pub success: bool,
    /// The response payload
    pub data: T,
    /// ISO timestamp of when data was fetched
    pub timestamp: String,
}

impl<T> ApiResponse<T> {
    /// Create a new successful API response with the current timestamp
    pub fn new(data: T) -> Self {
        Self {
            success: true,
            data,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// Create a new successful API response with a custom timestamp
    pub fn with_timestamp(data: T, timestamp: DateTime<Utc>) -> Self {
        Self {
            success: true,
            data,
            timestamp: timestamp.to_rfc3339(),
        }
    }
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    /// Whether the operation was successful (always false for errors)
    pub success: bool,
    /// Error message describing what went wrong
    pub error: String,
    /// ISO timestamp of when the error occurred
    pub timestamp: String,
}

impl ApiError {
    /// Create a new API error response with the current timestamp
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            success: false,
            error: error.into(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// Create a new API error response with a custom timestamp
    pub fn with_timestamp(error: impl Into<String>, timestamp: DateTime<Utc>) -> Self {
        Self {
            success: false,
            error: error.into(),
            timestamp: timestamp.to_rfc3339(),
        }
    }
}

/// Response wrapper for anime list endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnimeListResponse {
    /// List of anime items
    pub items: Vec<AnimeListItem>,
    /// Current page number
    pub page: i32,
    /// Applied filters
    pub filters: AnimeListFilters,
}

/// Filters applied to anime list query
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnimeListFilters {
    /// Type filter (TV, OVA, Movie, etc.)
    #[serde(rename = "type")]
    pub anime_type: String,
    /// Status filter (Ongoing, Completed, etc.)
    pub status: String,
    /// Sort order
    pub order: String,
}

/// Represents a crawled anime entry from bulk crawler
/// Same fields as AnimeListItem for consistency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CrawledAnime {
    /// Extracted from URL (e.g., "one-piece-subtitle-indonesia")
    pub slug: String,
    /// Anime title from h2[itemprop="headline"]
    pub title: String,
    /// Full URL to anime page from a[itemprop="url"]
    pub url: String,
    /// Thumbnail image URL from img.ts-post-image
    pub thumbnail: String,
    /// From div.status (Completed, Ongoing, etc.)
    pub status: String,
    /// From div.typez (TV, ONA, Movie, etc.)
    #[serde(rename = "type")]
    pub anime_type: String,
    /// From span.epx (episode count or status text)
    pub episode_status: String,
}

/// Database representation of CrawledAnime with timestamps
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CrawledAnimeRecord {
    /// Database ID
    pub id: i32,
    /// Extracted from URL (e.g., "one-piece-subtitle-indonesia")
    pub slug: String,
    /// Anime title
    pub title: String,
    /// Full URL to anime page
    pub url: String,
    /// Thumbnail image URL
    pub thumbnail: String,
    /// Completed, Ongoing, etc.
    pub status: String,
    /// TV, ONA, Movie, etc.
    #[serde(rename = "type")]
    pub anime_type: String,
    /// Episode count or status text
    pub episode_status: String,
    /// ISO timestamp when created
    pub created_at: String,
    /// ISO timestamp when last updated
    pub updated_at: String,
}

/// Response for the bulk crawler endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CrawlerResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Crawler result data
    pub data: CrawlerData,
    /// ISO timestamp of the response
    pub timestamp: String,
}

/// Data returned by the crawler endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CrawlerData {
    /// Total anime saved/updated
    pub total_crawled: i32,
    /// Total episodes saved/updated
    pub total_episodes: i32,
    /// Total video sources saved/updated
    pub total_video_sources: i32,
    /// Number of pages crawled
    pub pages_processed: i32,
    /// Any errors encountered during crawling
    pub errors: Vec<String>,
}

impl CrawlerResponse {
    /// Create a new crawler response with the current timestamp
    pub fn new(
        total_crawled: i32,
        total_episodes: i32,
        total_video_sources: i32,
        pages_processed: i32,
        errors: Vec<String>,
    ) -> Self {
        Self {
            success: true,
            data: CrawlerData {
                total_crawled,
                total_episodes,
                total_video_sources,
                pages_processed,
                errors,
            },
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

// ============================================================================
// Email Verification and Password Reset Models
// ============================================================================

/// Request body for forgot password
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ForgotPasswordRequest {
    /// User email address
    pub email: String,
}

/// Request body for password reset
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequest {
    /// Password reset token
    pub token: String,
    /// New password
    pub new_password: String,
}

/// Request body for email verification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyEmailRequest {
    /// Email verification token
    pub token: String,
}

/// Request body for resending verification email
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResendVerificationRequest {
    /// User email address
    pub email: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_favorite_serialization() {
        let favorite = UserFavorite {
            anime_slug: "naruto-shippuden".to_string(),
            anime_title: "Naruto Shippuden".to_string(),
            thumbnail: "https://example.com/naruto.jpg".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&favorite).unwrap();
        assert!(json.contains("\"animeSlug\""));
        assert!(json.contains("\"animeTitle\""));
        assert!(json.contains("\"thumbnail\""));
        assert!(json.contains("\"createdAt\""));
    }

    #[test]
    fn test_user_subscription_serialization() {
        let subscription = UserSubscription {
            anime_slug: "one-piece".to_string(),
            anime_title: "One Piece".to_string(),
            thumbnail: "https://example.com/onepiece.jpg".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&subscription).unwrap();
        assert!(json.contains("\"animeSlug\""));
        assert!(json.contains("\"animeTitle\""));
        assert!(json.contains("\"thumbnail\""));
        assert!(json.contains("\"createdAt\""));
    }

    #[test]
    fn test_user_history_serialization() {
        let history = UserHistory {
            episode_slug: "naruto-episode-1".to_string(),
            anime_slug: "naruto".to_string(),
            episode_title: "Episode 1".to_string(),
            anime_title: "Naruto".to_string(),
            thumbnail: "https://example.com/naruto-ep1.jpg".to_string(),
            watched_at: "2024-01-01T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&history).unwrap();
        assert!(json.contains("\"episodeSlug\""));
        assert!(json.contains("\"animeSlug\""));
        assert!(json.contains("\"episodeTitle\""));
        assert!(json.contains("\"animeTitle\""));
        assert!(json.contains("\"thumbnail\""));
        assert!(json.contains("\"watchedAt\""));
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: 1,
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            avatar: Some("https://example.com/avatar.jpg".to_string()),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"email\""));
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"avatar\""));
        assert!(json.contains("\"createdAt\""));
    }

    #[test]
    fn test_user_serialization_null_fields() {
        let user = User {
            id: 1,
            email: "test@example.com".to_string(),
            name: None,
            avatar: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"name\":null"));
        assert!(json.contains("\"avatar\":null"));
    }

    #[test]
    fn test_register_request_serialization() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            name: Some("Test User".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"email\""));
        assert!(json.contains("\"password\""));
        assert!(json.contains("\"name\""));
    }

    #[test]
    fn test_login_request_serialization() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"email\""));
        assert!(json.contains("\"password\""));
    }

    #[test]
    fn test_google_auth_request_serialization() {
        let request = GoogleAuthRequest {
            id_token: "google-id-token-123".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"idToken\""));
    }

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse::new(vec!["item1", "item2"]);

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\""));
        assert!(json.contains("\"timestamp\""));
    }

    #[test]
    fn test_api_error_serialization() {
        let error = ApiError::new("Something went wrong");

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"error\":\"Something went wrong\""));
        assert!(json.contains("\"timestamp\""));
    }

    #[test]
    fn test_anime_list_response_serialization() {
        let response = AnimeListResponse {
            items: vec![],
            page: 1,
            filters: AnimeListFilters {
                anime_type: "TV".to_string(),
                status: "Ongoing".to_string(),
                order: "latest".to_string(),
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"items\""));
        assert!(json.contains("\"page\""));
        assert!(json.contains("\"filters\""));
        assert!(json.contains("\"type\":\"TV\""));
        assert!(json.contains("\"status\":\"Ongoing\""));
        assert!(json.contains("\"order\":\"latest\""));
    }

    #[test]
    fn test_crawled_anime_serialization() {
        let anime = CrawledAnime {
            slug: "one-piece".to_string(),
            title: "One Piece".to_string(),
            url: "https://example.com/anime/one-piece/".to_string(),
            thumbnail: "https://example.com/onepiece.jpg".to_string(),
            status: "Ongoing".to_string(),
            anime_type: "TV".to_string(),
            episode_status: "1000+ Episodes".to_string(),
        };

        let json = serde_json::to_string(&anime).unwrap();
        assert!(json.contains("\"slug\""));
        assert!(json.contains("\"title\""));
        assert!(json.contains("\"url\""));
        assert!(json.contains("\"thumbnail\""));
        assert!(json.contains("\"status\""));
        assert!(json.contains("\"type\":\"TV\""));
        assert!(json.contains("\"episodeStatus\""));
    }

    #[test]
    fn test_crawled_anime_record_serialization() {
        let anime = CrawledAnimeRecord {
            id: 1,
            slug: "one-piece".to_string(),
            title: "One Piece".to_string(),
            url: "https://example.com/anime/one-piece/".to_string(),
            thumbnail: "https://example.com/onepiece.jpg".to_string(),
            status: "Ongoing".to_string(),
            anime_type: "TV".to_string(),
            episode_status: "1000+ Episodes".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&anime).unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"slug\""));
        assert!(json.contains("\"title\""));
        assert!(json.contains("\"url\""));
        assert!(json.contains("\"thumbnail\""));
        assert!(json.contains("\"status\""));
        assert!(json.contains("\"type\":\"TV\""));
        assert!(json.contains("\"episodeStatus\""));
        assert!(json.contains("\"createdAt\""));
        assert!(json.contains("\"updatedAt\""));
    }

    #[test]
    fn test_crawler_response_serialization() {
        let response = CrawlerResponse::new(100, 500, 2000, 5, vec!["Error on page 3".to_string()]);

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"totalCrawled\":100"));
        assert!(json.contains("\"totalEpisodes\":500"));
        assert!(json.contains("\"totalVideoSources\":2000"));
        assert!(json.contains("\"pagesProcessed\":5"));
        assert!(json.contains("\"errors\""));
        assert!(json.contains("\"timestamp\""));
    }

    #[test]
    fn test_auth_response_serialization() {
        let response = AuthResponse {
            success: true,
            data: AuthData {
                user: User {
                    id: 1,
                    email: "test@example.com".to_string(),
                    name: Some("Test".to_string()),
                    avatar: None,
                    created_at: "2024-01-01T00:00:00Z".to_string(),
                },
                token: "jwt-token-123".to_string(),
            },
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\""));
        assert!(json.contains("\"user\""));
        assert!(json.contains("\"token\""));
        assert!(json.contains("\"timestamp\""));
    }

    #[test]
    fn test_api_response_new() {
        let response = ApiResponse::new("test data");
        assert!(response.success);
        assert_eq!(response.data, "test data");
        assert!(!response.timestamp.is_empty());
    }

    #[test]
    fn test_api_error_new() {
        let error = ApiError::new("test error");
        assert!(!error.success);
        assert_eq!(error.error, "test error");
        assert!(!error.timestamp.is_empty());
    }

    #[test]
    fn test_crawler_response_new() {
        let response = CrawlerResponse::new(50, 200, 800, 3, vec![]);
        assert!(response.success);
        assert_eq!(response.data.total_crawled, 50);
        assert_eq!(response.data.total_episodes, 200);
        assert_eq!(response.data.total_video_sources, 800);
        assert_eq!(response.data.pages_processed, 3);
        assert!(response.data.errors.is_empty());
        assert!(!response.timestamp.is_empty());
    }

    // Test deserialization

    #[test]
    fn test_user_favorite_deserialization() {
        let json = r#"{
            "animeSlug": "naruto",
            "animeTitle": "Naruto",
            "thumbnail": "https://example.com/naruto.jpg",
            "createdAt": "2024-01-01T00:00:00Z"
        }"#;

        let favorite: UserFavorite = serde_json::from_str(json).unwrap();
        assert_eq!(favorite.anime_slug, "naruto");
        assert_eq!(favorite.anime_title, "Naruto");
        assert_eq!(favorite.thumbnail, "https://example.com/naruto.jpg");
        assert_eq!(favorite.created_at, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_login_request_deserialization() {
        let json = r#"{
            "email": "test@example.com",
            "password": "secret123"
        }"#;

        let request: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.password, "secret123");
    }

    #[test]
    fn test_register_request_deserialization() {
        let json = r#"{
            "email": "test@example.com",
            "password": "secret123",
            "name": "Test User"
        }"#;

        let request: RegisterRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.password, "secret123");
        assert_eq!(request.name, Some("Test User".to_string()));
    }

    #[test]
    fn test_register_request_deserialization_without_name() {
        let json = r#"{
            "email": "test@example.com",
            "password": "secret123"
        }"#;

        let request: RegisterRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.password, "secret123");
        assert_eq!(request.name, None);
    }
}
