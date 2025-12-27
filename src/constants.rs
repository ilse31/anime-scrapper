//! Constants module for the Anime Scraper API
//!
//! Contains endpoint URL builders that use the base URL from configuration.

/// URL builder functions for all endpoints
pub mod endpoints {
    /// Home page URL
    pub fn home(base_url: &str) -> String {
        base_url.to_string()
    }

    /// Search URL with query parameter
    pub fn search(base_url: &str, query: &str) -> String {
        format!("{}/?s={}", base_url, urlencoding::encode(query))
    }

    /// Anime list URL with filters
    pub fn anime_list(base_url: &str, page: u32, type_filter: &str, status: &str, order: &str) -> String {
        format!(
            "{}/anime/?page={}&status={}&type={}&order={}",
            base_url, page, status, type_filter, order
        )
    }

    /// Anime detail page URL
    pub fn anime(base_url: &str, slug: &str) -> String {
        format!("{}/anime/{}/", base_url, slug)
    }

    /// Episode page URL
    pub fn episode(base_url: &str, slug: &str) -> String {
        format!("{}/{}/", base_url, slug)
    }
}

/// Filter options for anime list
pub mod filters {
    /// Available anime types
    pub const ANIME_TYPES: &[&str] = &[
        "",
        "TV",
        "OVA",
        "Movie",
        "Live Action",
        "Special",
        "BD",
        "ONA",
        "Music",
    ];

    /// Available anime statuses
    pub const ANIME_STATUS: &[&str] = &[
        "",
        "Ongoing",
        "Completed",
        "Upcoming",
        "Hiatus",
    ];

    /// Available sort orders
    /// Maps to: Default, A-Z, Z-A, Latest Update, Latest Added, Popular, Rating
    pub const ANIME_ORDER: &[&str] = &[
        "",
        "title",
        "titlereverse",
        "update",
        "latest",
        "popular",
        "rating",
    ];
}
