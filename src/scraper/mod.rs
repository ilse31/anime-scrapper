//! Scraper module for fetching HTML content from the target URL
//!
//! This module provides HTTP client functionality with browser-like headers
//! and anti-detection features to fetch HTML content from sokuja.uk.

use rand::Rng;
use reqwest::{Client, StatusCode};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

/// Errors that can occur during scraping operations
#[derive(Error, Debug)]
pub enum ScraperError {
    /// Network-related errors (connection timeout, DNS failure, etc.)
    #[error("Failed to connect to server: {0}")]
    NetworkError(String),

    /// HTTP non-200 status code errors
    #[error("Server returned status {0}")]
    HttpError(u16),

    /// Error reading response body
    #[error("Failed to read response body: {0}")]
    ResponseError(String),

    /// Rate limited by server
    #[error("Rate limited, retry after delay")]
    RateLimited,
}

/// Result of a successful page fetch
#[derive(Debug)]
pub struct ScraperResult {
    /// The HTML content of the page
    pub html: String,
    /// The HTTP status code
    pub status: u16,
}

/// Configuration for anti-detection features
#[derive(Debug, Clone)]
pub struct ScraperConfig {
    /// Minimum delay between requests in milliseconds
    pub min_delay_ms: u64,
    /// Maximum delay between requests in milliseconds
    pub max_delay_ms: u64,
    /// Whether to rotate user agents
    pub rotate_user_agent: bool,
    /// Maximum retries on failure
    pub max_retries: u32,
    /// Base delay for exponential backoff in milliseconds
    pub backoff_base_ms: u64,
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            min_delay_ms: 1000,
            max_delay_ms: 3000,
            rotate_user_agent: true,
            max_retries: 3,
            backoff_base_ms: 1000,
        }
    }
}

/// List of realistic user agents for rotation
const USER_AGENTS: &[&str] = &[
    // Chrome on Windows
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36",
    // Chrome on macOS
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    // Firefox on Windows
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
    // Firefox on macOS
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:121.0) Gecko/20100101 Firefox/121.0",
    // Safari on macOS
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15",
    // Edge on Windows
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0",
];

/// HTTP client for scraping web pages with anti-detection features
pub struct Scraper {
    client: Client,
    config: ScraperConfig,
    request_count: AtomicUsize,
}

impl Default for Scraper {
    fn default() -> Self {
        Self::new()
    }
}

impl Scraper {
    /// Create a new Scraper with default configuration
    pub fn new() -> Self {
        Self::with_config(ScraperConfig::default())
    }

    /// Create a new Scraper with custom configuration
    pub fn with_config(config: ScraperConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            config,
            request_count: AtomicUsize::new(0),
        }
    }

    /// Get a random user agent from the list
    fn get_user_agent(&self) -> &'static str {
        if self.config.rotate_user_agent {
            let idx = rand::thread_rng().gen_range(0..USER_AGENTS.len());
            USER_AGENTS[idx]
        } else {
            USER_AGENTS[0]
        }
    }

    /// Apply random delay between requests
    async fn apply_delay(&self) {
        let delay = rand::thread_rng().gen_range(self.config.min_delay_ms..=self.config.max_delay_ms);
        sleep(Duration::from_millis(delay)).await;
    }

    /// Apply exponential backoff delay
    async fn apply_backoff(&self, attempt: u32) {
        let delay = self.config.backoff_base_ms * 2u64.pow(attempt);
        let jitter = rand::thread_rng().gen_range(0..500);
        sleep(Duration::from_millis(delay + jitter)).await;
    }

    /// Get headers that match the user agent
    fn get_sec_ch_ua(&self, user_agent: &str) -> (&'static str, &'static str, &'static str) {
        if user_agent.contains("Chrome/120") {
            (
                "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"",
                "?0",
                "\"Windows\"",
            )
        } else if user_agent.contains("Chrome/119") {
            (
                "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"119\", \"Google Chrome\";v=\"119\"",
                "?0",
                "\"Windows\"",
            )
        } else if user_agent.contains("Edg/") {
            (
                "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Microsoft Edge\";v=\"120\"",
                "?0",
                "\"Windows\"",
            )
        } else if user_agent.contains("Macintosh") {
            (
                "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"",
                "?0",
                "\"macOS\"",
            )
        } else {
            // Firefox doesn't send Sec-Ch-Ua headers, but we'll use empty strings
            ("", "", "")
        }
    }

    /// Fetch a page from the given URL with anti-detection features
    pub async fn fetch_page(&self, url: &str) -> Result<ScraperResult, ScraperError> {
        // Apply delay before request (except for first request)
        let count = self.request_count.fetch_add(1, Ordering::SeqCst);
        if count > 0 {
            self.apply_delay().await;
        }

        let mut last_error = None;

        for attempt in 0..self.config.max_retries {
            if attempt > 0 {
                self.apply_backoff(attempt).await;
            }

            match self.do_fetch(url).await {
                Ok(result) => return Ok(result),
                Err(ScraperError::RateLimited) => {
                    tracing::warn!("Rate limited on attempt {}, backing off...", attempt + 1);
                    last_error = Some(ScraperError::RateLimited);
                    continue;
                }
                Err(ScraperError::HttpError(status)) if status == 429 || status >= 500 => {
                    tracing::warn!("HTTP {} on attempt {}, retrying...", status, attempt + 1);
                    last_error = Some(ScraperError::HttpError(status));
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap_or(ScraperError::NetworkError("Max retries exceeded".to_string())))
    }

    /// Internal fetch implementation
    async fn do_fetch(&self, url: &str) -> Result<ScraperResult, ScraperError> {
        let user_agent = self.get_user_agent();
        let (sec_ch_ua, sec_ch_ua_mobile, sec_ch_ua_platform) = self.get_sec_ch_ua(user_agent);

        let mut request = self
            .client
            .get(url)
            .header("User-Agent", user_agent)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9,id;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none")
            .header("Sec-Fetch-User", "?1")
            .header("Upgrade-Insecure-Requests", "1");

        // Add Sec-Ch-Ua headers only for Chrome-based browsers
        if !sec_ch_ua.is_empty() {
            request = request
                .header("Sec-Ch-Ua", sec_ch_ua)
                .header("Sec-Ch-Ua-Mobile", sec_ch_ua_mobile)
                .header("Sec-Ch-Ua-Platform", sec_ch_ua_platform);
        }

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                ScraperError::NetworkError("Connection timeout".to_string())
            } else if e.is_connect() {
                ScraperError::NetworkError("Failed to connect to server".to_string())
            } else {
                ScraperError::NetworkError(e.to_string())
            }
        })?;

        let status = response.status();
        let status_code = status.as_u16();

        // Handle rate limiting
        if status_code == 429 {
            return Err(ScraperError::RateLimited);
        }

        if status != StatusCode::OK {
            return Err(ScraperError::HttpError(status_code));
        }

        let html = response
            .text()
            .await
            .map_err(|e| ScraperError::ResponseError(e.to_string()))?;

        Ok(ScraperResult {
            html,
            status: status_code,
        })
    }

    /// Fetch a page without delay (for single requests)
    pub async fn fetch_page_no_delay(&self, url: &str) -> Result<ScraperResult, ScraperError> {
        self.do_fetch(url).await
    }

    /// Reset request counter (useful for new crawl sessions)
    pub fn reset_counter(&self) {
        self.request_count.store(0, Ordering::SeqCst);
    }

    /// Get current request count
    pub fn request_count(&self) -> usize {
        self.request_count.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scraper_creation() {
        let scraper = Scraper::new();
        assert_eq!(scraper.request_count(), 0);
    }

    #[test]
    fn test_scraper_with_config() {
        let config = ScraperConfig {
            min_delay_ms: 500,
            max_delay_ms: 1500,
            rotate_user_agent: false,
            max_retries: 5,
            backoff_base_ms: 2000,
        };
        let scraper = Scraper::with_config(config);
        assert_eq!(scraper.config.min_delay_ms, 500);
        assert_eq!(scraper.config.max_retries, 5);
    }

    #[test]
    fn test_user_agent_rotation() {
        let scraper = Scraper::new();
        let ua1 = scraper.get_user_agent();
        // User agent should be from our list
        assert!(USER_AGENTS.contains(&ua1));
    }

    #[test]
    fn test_sec_ch_ua_headers() {
        let scraper = Scraper::new();
        
        // Chrome Windows
        let (ua, mobile, platform) = scraper.get_sec_ch_ua(USER_AGENTS[0]);
        assert!(ua.contains("Chrome"));
        assert_eq!(mobile, "?0");
        assert_eq!(platform, "\"Windows\"");

        // macOS
        let (_, _, platform) = scraper.get_sec_ch_ua(USER_AGENTS[2]);
        assert_eq!(platform, "\"macOS\"");
    }

    #[test]
    fn test_request_counter() {
        let scraper = Scraper::new();
        assert_eq!(scraper.request_count(), 0);
        scraper.request_count.fetch_add(1, Ordering::SeqCst);
        assert_eq!(scraper.request_count(), 1);
        scraper.reset_counter();
        assert_eq!(scraper.request_count(), 0);
    }

    #[test]
    fn test_default_config() {
        let config = ScraperConfig::default();
        assert_eq!(config.min_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 3000);
        assert!(config.rotate_user_agent);
        assert_eq!(config.max_retries, 3);
    }
}
