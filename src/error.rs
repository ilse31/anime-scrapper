//! Global error handling module for the Anime Scraper API
//!
//! This module provides a unified error type that handles all application errors
//! and converts them to appropriate HTTP responses with consistent JSON structure.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;

use crate::auth::AuthError;
use crate::db::DbError;
use crate::models::ApiError;
use crate::scraper::ScraperError;

/// Application-wide error type that unifies all error sources
#[derive(Debug, Error)]
pub enum AppError {
    /// Scraping-related errors (network, HTTP, parsing)
    #[error("Scraping error: {0}")]
    Scraping(#[from] ScraperError),

    /// Database-related errors
    #[error("Database error: {0}")]
    Database(#[from] DbError),

    /// Authentication-related errors
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    /// Validation errors (bad request)
    #[error("Validation error: {0}")]
    Validation(String),

    /// Resource not found errors
    #[error("Not found: {0}")]
    NotFound(String),

    /// Conflict errors (e.g., duplicate resource)
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Internal server errors
    #[error("Internal error: {0}")]
    Internal(String),

    /// SQLx database errors (direct)
    #[error("Database query error: {0}")]
    SqlxError(#[from] sqlx::Error),
}

impl AppError {
    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        AppError::Validation(msg.into())
    }

    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        AppError::NotFound(msg.into())
    }

    /// Create a conflict error
    pub fn conflict(msg: impl Into<String>) -> Self {
        AppError::Conflict(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        AppError::Internal(msg.into())
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 400 Bad Request - Validation errors
            AppError::Validation(_) => StatusCode::BAD_REQUEST,

            // 401 Unauthorized - Authentication errors
            AppError::Auth(auth_err) => match auth_err {
                AuthError::InvalidCredentials
                | AuthError::TokenExpired
                | AuthError::InvalidToken
                | AuthError::MissingAuthHeader
                | AuthError::InvalidAuthHeaderFormat
                | AuthError::TokenVerificationError(_) => StatusCode::UNAUTHORIZED,
                // Other auth errors are internal
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },

            // 404 Not Found
            AppError::NotFound(_) => StatusCode::NOT_FOUND,

            // 409 Conflict
            AppError::Conflict(_) => StatusCode::CONFLICT,

            // 500 Internal Server Error - Scraping, Database, Internal errors
            AppError::Scraping(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            AppError::Validation(msg) => msg.clone(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::Conflict(msg) => msg.clone(),
            AppError::Internal(msg) => msg.clone(),

            AppError::Auth(auth_err) => match auth_err {
                AuthError::InvalidCredentials => "Invalid email or password".to_string(),
                AuthError::TokenExpired => "Token has expired, please login again".to_string(),
                AuthError::InvalidToken => "Invalid authentication token".to_string(),
                AuthError::MissingAuthHeader => "Authorization header is required".to_string(),
                AuthError::InvalidAuthHeaderFormat => {
                    "Invalid authorization header format, expected 'Bearer <token>'".to_string()
                }
                AuthError::TokenVerificationError(_) => "Invalid authentication token".to_string(),
                AuthError::UserNotFound => "User not found".to_string(),
                AuthError::HashingError(_) => "Authentication processing error".to_string(),
                AuthError::TokenGenerationError(_) => {
                    "Failed to generate authentication token".to_string()
                }
                AuthError::GoogleOAuthError(msg) => {
                    format!("Google authentication failed: {}", msg)
                }
            },

            AppError::Scraping(scraper_err) => match scraper_err {
                ScraperError::NetworkError(msg) => format!("Failed to connect to server: {}", msg),
                ScraperError::HttpError(status) => {
                    format!("Server returned error status: {}", status)
                }
                ScraperError::ResponseError(msg) => format!("Failed to read response: {}", msg),
                ScraperError::RateLimited => {
                    "Server is rate limiting requests, please try again later".to_string()
                }
            },

            AppError::Database(db_err) => match db_err {
                DbError::ConnectionError(_) => "Database connection error".to_string(),
                DbError::HealthCheckError(_) => "Database health check failed".to_string(),
            },

            AppError::SqlxError(_) => "Database operation failed".to_string(),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        self.status_code()
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let error_response = ApiError::new(self.user_message());

        HttpResponse::build(status).json(error_response)
    }
}

/// Result type alias for operations that can fail with AppError
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_status_code() {
        let error = AppError::validation("Invalid input");
        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_not_found_error_status_code() {
        let error = AppError::not_found("Resource not found");
        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_conflict_error_status_code() {
        let error = AppError::conflict("Resource already exists");
        assert_eq!(error.status_code(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_internal_error_status_code() {
        let error = AppError::internal("Something went wrong");
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_auth_error_unauthorized() {
        let error = AppError::Auth(AuthError::InvalidCredentials);
        assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);

        let error = AppError::Auth(AuthError::TokenExpired);
        assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);

        let error = AppError::Auth(AuthError::MissingAuthHeader);
        assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_scraper_error_internal() {
        let error = AppError::Scraping(ScraperError::NetworkError("timeout".to_string()));
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);

        let error = AppError::Scraping(ScraperError::HttpError(503));
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_validation_error_message() {
        let error = AppError::validation("Email is required");
        assert_eq!(error.user_message(), "Email is required");
    }

    #[test]
    fn test_not_found_error_message() {
        let error = AppError::not_found("Anime not found");
        assert_eq!(error.user_message(), "Anime not found");
    }

    #[test]
    fn test_conflict_error_message() {
        let error = AppError::conflict("Email already registered");
        assert_eq!(error.user_message(), "Email already registered");
    }

    #[test]
    fn test_auth_error_user_messages() {
        let error = AppError::Auth(AuthError::InvalidCredentials);
        assert_eq!(error.user_message(), "Invalid email or password");

        let error = AppError::Auth(AuthError::TokenExpired);
        assert_eq!(
            error.user_message(),
            "Token has expired, please login again"
        );

        let error = AppError::Auth(AuthError::MissingAuthHeader);
        assert_eq!(error.user_message(), "Authorization header is required");
    }

    #[test]
    fn test_scraper_error_user_messages() {
        let error =
            AppError::Scraping(ScraperError::NetworkError("connection refused".to_string()));
        assert!(error.user_message().contains("Failed to connect"));

        let error = AppError::Scraping(ScraperError::HttpError(500));
        assert!(error.user_message().contains("500"));

        let error = AppError::Scraping(ScraperError::RateLimited);
        assert!(error.user_message().contains("rate limiting"));
    }

    #[test]
    fn test_error_display() {
        let error = AppError::validation("test error");
        assert_eq!(format!("{}", error), "Validation error: test error");

        let error = AppError::not_found("anime");
        assert_eq!(format!("{}", error), "Not found: anime");
    }

    #[test]
    fn test_from_scraper_error() {
        let scraper_err = ScraperError::NetworkError("timeout".to_string());
        let app_err: AppError = scraper_err.into();
        assert!(matches!(app_err, AppError::Scraping(_)));
    }

    #[test]
    fn test_from_auth_error() {
        let auth_err = AuthError::InvalidCredentials;
        let app_err: AppError = auth_err.into();
        assert!(matches!(app_err, AppError::Auth(_)));
    }

    #[test]
    fn test_from_db_error() {
        let db_err = DbError::HealthCheckError("test".to_string());
        let app_err: AppError = db_err.into();
        assert!(matches!(app_err, AppError::Database(_)));
    }
}
