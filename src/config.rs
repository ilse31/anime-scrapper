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
    /// SMTP configuration for sending emails
    pub smtp: Option<SmtpConfig>,
    /// Frontend URL for email links
    pub frontend_url: String,
}

/// SMTP configuration for email sending
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    /// SMTP server host
    pub host: String,
    /// SMTP server port
    pub port: u16,
    /// SMTP username
    pub username: String,
    /// SMTP password
    pub password: String,
    /// Sender email address
    pub from_email: String,
    /// Sender name
    pub from_name: String,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// # Panics
    /// Panics if required environment variables are not set
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        // Load SMTP config if all required vars are present
        let smtp = match (
            env::var("SMTP_HOST").ok(),
            env::var("SMTP_PORT").ok(),
            env::var("SMTP_USERNAME").ok(),
            env::var("SMTP_PASSWORD").ok(),
            env::var("SMTP_FROM_EMAIL").ok(),
        ) {
            (Some(host), Some(port), Some(username), Some(password), Some(from_email)) => {
                Some(SmtpConfig {
                    host,
                    port: port.parse().unwrap_or(587),
                    username,
                    password,
                    from_email: from_email.clone(),
                    from_name: env::var("SMTP_FROM_NAME")
                        .unwrap_or_else(|_| "Anime Scraper".to_string()),
                })
            }
            _ => None,
        };

        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a valid number"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            google_client_id: env::var("GOOGLE_CLIENT_ID").ok(),
            base_url: env::var("BASE_URL").unwrap_or_else(|_| "https://x3.sokuja.uk".to_string()),
            smtp,
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        }
    }
}
