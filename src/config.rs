//! Configuration module for the Anime Scraper API
//!
//! Handles loading environment variables and application configuration.

use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    /// Database connection URL
    pub database_url: String,
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// JWT secret key for token signing
    pub jwt_secret: String,
    /// Google OAuth client ID
    pub google_client_id: Option<String>,
    /// Base URL for anime scraper source
    pub base_url: String,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// # Panics
    /// Panics if required environment variables are not set
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a valid number"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            google_client_id: env::var("GOOGLE_CLIENT_ID").ok(),
            base_url: env::var("BASE_URL")
                .unwrap_or_else(|_| "https://x3.sokuja.uk".to_string()),
        }
    }
}
