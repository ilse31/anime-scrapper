//! Parser module for extracting structured data from HTML
//!
//! This module provides parsing functionality to extract anime data
//! from the HTML content fetched from sokuja.uk.

use base64::{engine::general_purpose::STANDARD, Engine};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

/// Represents an anime update from the latest updates section
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnimeUpdate {
    /// Extracted slug from series URL (e.g., "one-piece-subtitle-indonesia")
    pub slug: String,
    /// Episode title from h2[itemprop="headline"] a
    pub title: String,
    /// Episode URL from a[itemprop="url"]
    pub episode_url: String,
    /// Image from img.ts-post-image
    pub thumbnail: String,
    /// From div.epin (e.g., "24/24")
    pub episode_number: String,
    /// From span.type (TV, OVA, etc.)
    #[serde(rename = "type")]
    pub anime_type: String,
    /// From div.sosev span a
    pub series_title: String,
    /// Series URL from div.sosev span a href
    pub series_url: String,
    /// Completed/Ongoing from status indicator
    pub status: String,
    /// From div.sosev span (date/time info)
    pub release_info: String,
}

/// Represents a search result entry from search results (article.bs)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// Extracted slug from URL (e.g., "one-piece-subtitle-indonesia")
    pub slug: String,
    /// From h2[itemprop="headline"]
    pub title: String,
    /// From a[itemprop="url"]
    pub url: String,
    /// From img.ts-post-image
    pub thumbnail: String,
    /// From div.status
    pub status: String,
    /// From div.typez (TV, ONA, Movie)
    #[serde(rename = "type")]
    pub anime_type: String,
    /// From span.epx (Completed, Ongoing)
    pub episode_status: String,
}

/// Represents an anime list item from the anime list page (article.bs)
/// Same structure as SearchResult - uses same HTML elements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnimeListItem {
    /// Extracted slug from URL (e.g., "one-piece-subtitle-indonesia")
    pub slug: String,
    /// From h2[itemprop="headline"]
    pub title: String,
    /// From a[itemprop="url"]
    pub url: String,
    /// From img.ts-post-image
    pub thumbnail: String,
    /// From div.status
    pub status: String,
    /// From div.typez (TV, ONA, Movie)
    #[serde(rename = "type")]
    pub anime_type: String,
    /// From span.epx (Completed, Ongoing)
    pub episode_status: String,
}

/// Represents an episode entry from the episode list
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    /// Extracted slug from URL (e.g., "naruto-episode-1")
    pub slug: String,
    /// From div.epl-num
    pub number: String,
    /// From div.epl-title
    pub title: String,
    /// From a href
    pub url: String,
    /// From div.epl-date
    pub release_date: String,
}

/// Represents a video source with server, quality, and URL
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VideoSource {
    /// Server name (e.g., "SOKUJA")
    pub server: String,
    /// Quality (e.g., "480p", "720p", "1080p")
    pub quality: String,
    /// Direct video URL from decoded base64
    pub url: String,
}

/// Represents episode detail with video sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeDetail {
    /// Episode title
    pub title: String,
    /// Default video URL from div#embed_holder video source
    pub default_video: String,
    /// All available video sources
    pub sources: Vec<VideoSource>,
}

/// Represents full anime information from detail page
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnimeDetail {
    /// From h1.entry-title
    pub title: String,
    /// From span.alter
    pub alternate_titles: String,
    /// From div.thumb img
    pub poster: String,
    /// From meta[itemprop="ratingValue"]
    pub rating: String,
    /// From a.trailerbutton href
    pub trailer_url: String,
    /// From div.spe span (Status:)
    pub status: String,
    /// From div.spe span (Studio:)
    pub studio: String,
    /// From div.spe span (Tanggal Rilis:)
    pub release_date: String,
    /// From div.spe span (Durasi:)
    pub duration: String,
    /// From div.spe span (Season:)
    pub season: String,
    /// From div.spe span (Tipe:)
    #[serde(rename = "type")]
    pub anime_type: String,
    /// From div.spe span (Total Episode:)
    pub total_episodes: String,
    /// From Director link
    pub director: String,
    /// From a.casts elements
    pub casts: Vec<String>,
    /// From div.genxed a elements
    pub genres: Vec<String>,
    /// From div.desc
    pub synopsis: String,
    /// From div.eplister
    pub episodes: Vec<Episode>,
}

/// Represents a completed anime entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CompletedAnime {
    /// Extracted slug from URL (e.g., "one-piece-subtitle-indonesia")
    pub slug: String,
    /// From h2[itemprop="headline"] a
    pub title: String,
    /// From a[itemprop="url"]
    pub url: String,
    /// From img.ts-post-image
    pub thumbnail: String,
    /// From div.typez (TV, Special, etc.)
    #[serde(rename = "type")]
    pub anime_type: String,
    /// From span.epx
    pub episode_count: String,
    /// From li containing "Status:"
    pub status: String,
    /// From li containing "Dipos Oleh:"
    pub posted_by: String,
    /// From li containing "Dipos pada:"
    pub posted_at: String,
    /// From series link text
    pub series_title: String,
    /// From series link href
    pub series_url: String,
    /// From genre links
    pub genres: Vec<String>,
    /// From span.scr
    pub rating: String,
}

/// Parse anime updates from the home page HTML
///
/// Extracts data from elements matching `article.seventh`
///
/// # Arguments
/// * `html` - The HTML content to parse
///
/// # Returns
/// A vector of `AnimeUpdate` structs
pub fn parse_anime_updates(html: &str) -> Vec<AnimeUpdate> {
    let document = Html::parse_document(html);
    
    // Selector for article.seventh elements
    let article_selector = Selector::parse("article.seventh").unwrap();
    
    // Selectors for individual fields
    let title_selector = Selector::parse("h2[itemprop=\"headline\"] a").unwrap();
    let url_selector = Selector::parse("a[itemprop=\"url\"]").unwrap();
    let thumbnail_selector = Selector::parse("img.ts-post-image").unwrap();
    let episode_number_selector = Selector::parse("div.epin").unwrap();
    let type_selector = Selector::parse("span.type").unwrap();
    let series_selector = Selector::parse("div.sosev span a").unwrap();
    let release_info_selector = Selector::parse("div.sosev span").unwrap();
    let status_selector = Selector::parse("span.status").unwrap();
    
    let mut updates = Vec::new();
    
    for article in document.select(&article_selector) {
        let title = article
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let episode_url = article
            .select(&url_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let thumbnail = article
            .select(&thumbnail_selector)
            .next()
            .and_then(|el| el.value().attr("src").or_else(|| el.value().attr("data-src")))
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let episode_number = article
            .select(&episode_number_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let anime_type = article
            .select(&type_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let (series_title, series_url) = article
            .select(&series_selector)
            .next()
            .map(|el| {
                let text = el.text().collect::<String>().trim().to_string();
                let href = el.value().attr("href").unwrap_or_default().to_string();
                (text, href)
            })
            .unwrap_or_default();
        
        // Extract release info from div.sosev span (the one containing date/time)
        let release_info = article
            .select(&release_info_selector)
            .filter_map(|el| {
                let text = el.text().collect::<String>().trim().to_string();
                // Look for spans that contain date/time info (not the series link)
                if !text.is_empty() && el.select(&Selector::parse("a").unwrap()).next().is_none() {
                    Some(text)
                } else {
                    None
                }
            })
            .next()
            .unwrap_or_default();
        
        let status = article
            .select(&status_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        updates.push(AnimeUpdate {
            slug: extract_slug_from_url(&series_url),
            title,
            episode_url,
            thumbnail,
            episode_number,
            anime_type,
            series_title,
            series_url,
            status,
            release_info,
        });
    }
    
    updates
}

/// Parse completed anime from the HTML
///
/// Extracts data from elements matching `article.stylesix`
///
/// # Arguments
/// * `html` - The HTML content to parse
///
/// # Returns
/// A vector of `CompletedAnime` structs
pub fn parse_completed_anime(html: &str) -> Vec<CompletedAnime> {
    let document = Html::parse_document(html);
    
    // Selector for article.stylesix elements
    let article_selector = Selector::parse("article.stylesix").unwrap();
    
    // Selectors for individual fields
    let title_selector = Selector::parse("h2[itemprop=\"headline\"] a").unwrap();
    let url_selector = Selector::parse("a[itemprop=\"url\"]").unwrap();
    let thumbnail_selector = Selector::parse("img.ts-post-image").unwrap();
    let type_selector = Selector::parse("div.typez").unwrap();
    let episode_count_selector = Selector::parse("span.epx").unwrap();
    let rating_selector = Selector::parse("span.scr").unwrap();
    let genre_selector = Selector::parse("a[rel=\"tag\"]").unwrap();
    let li_selector = Selector::parse("li").unwrap();
    let series_link_selector = Selector::parse("a").unwrap();
    
    let mut completed = Vec::new();
    
    for article in document.select(&article_selector) {
        let title = article
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let url = article
            .select(&url_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let thumbnail = article
            .select(&thumbnail_selector)
            .next()
            .and_then(|el| el.value().attr("src").or_else(|| el.value().attr("data-src")))
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let anime_type = article
            .select(&type_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let episode_count = article
            .select(&episode_count_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let rating = article
            .select(&rating_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        // Extract genres from genre links
        let genres: Vec<String> = article
            .select(&genre_selector)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // Extract status, posted_by, posted_at from list items
        let mut status = String::new();
        let mut posted_by = String::new();
        let mut posted_at = String::new();
        let mut series_title = String::new();
        let mut series_url = String::new();
        
        for li in article.select(&li_selector) {
            let text = li.text().collect::<String>();
            let text_lower = text.to_lowercase();
            
            if text_lower.contains("status:") || text_lower.contains("status :") {
                status = text
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
            } else if text_lower.contains("dipos oleh:") || text_lower.contains("posted by:") {
                posted_by = text
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
            } else if text_lower.contains("dipos pada:") || text_lower.contains("posted at:") || text_lower.contains("posted on:") {
                posted_at = text
                    .split(':')
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join(":")
                    .trim()
                    .to_string();
            }
            
            // Check for series link in list items
            if let Some(link) = li.select(&series_link_selector).next() {
                let href = link.value().attr("href").unwrap_or_default();
                if href.contains("/anime/") && series_url.is_empty() {
                    series_title = link.text().collect::<String>().trim().to_string();
                    series_url = href.to_string();
                }
            }
        }
        
        completed.push(CompletedAnime {
            slug: extract_slug_from_url(&url),
            title,
            url,
            thumbnail,
            anime_type,
            episode_count,
            status,
            posted_by,
            posted_at,
            series_title,
            series_url,
            genres,
            rating,
        });
    }
    
    completed
}

/// Parse search results from the search page HTML
///
/// Extracts data from elements matching `article.bs` inside `div.listupd`
///
/// # Arguments
/// * `html` - The HTML content to parse
///
/// # Returns
/// A vector of `SearchResult` structs. Returns empty array if no results found.
pub fn parse_search_results(html: &str) -> Vec<SearchResult> {
    let document = Html::parse_document(html);
    
    // First try to find div.listupd container, then look for article.bs inside
    let listupd_selector = Selector::parse("div.listupd").unwrap();
    let article_selector = Selector::parse("article.bs").unwrap();
    
    // Selectors for individual fields
    let title_selector = Selector::parse("h2[itemprop=\"headline\"]").unwrap();
    let url_selector = Selector::parse("a[itemprop=\"url\"]").unwrap();
    let thumbnail_selector = Selector::parse("img.ts-post-image").unwrap();
    let status_selector = Selector::parse("div.status").unwrap();
    let type_selector = Selector::parse("div.typez").unwrap();
    let episode_status_selector = Selector::parse("span.epx").unwrap();
    
    let mut results = Vec::new();
    
    // Try to find articles inside div.listupd first
    let articles: Vec<_> = if let Some(listupd) = document.select(&listupd_selector).next() {
        listupd.select(&article_selector).collect()
    } else {
        // Fallback: look for article.bs anywhere in the document
        document.select(&article_selector).collect()
    };
    
    for article in articles {
        let title = article
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let url = article
            .select(&url_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let thumbnail = article
            .select(&thumbnail_selector)
            .next()
            .and_then(|el| el.value().attr("src").or_else(|| el.value().attr("data-src")))
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let status = article
            .select(&status_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let anime_type = article
            .select(&type_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let episode_status = article
            .select(&episode_status_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        results.push(SearchResult {
            slug: extract_slug_from_url(&url),
            title,
            url,
            thumbnail,
            status,
            anime_type,
            episode_status,
        });
    }
    
    results
}

/// Parse anime list from the anime list page HTML
///
/// Extracts data from elements matching `article.bs` inside `div.listupd`
/// Same structure as search results.
///
/// # Arguments
/// * `html` - The HTML content to parse
///
/// # Returns
/// A vector of `AnimeListItem` structs. Returns empty array if no results found.
pub fn parse_anime_list(html: &str) -> Vec<AnimeListItem> {
    let document = Html::parse_document(html);
    
    // First try to find div.listupd container, then look for article.bs inside
    let listupd_selector = Selector::parse("div.listupd").unwrap();
    let article_selector = Selector::parse("article.bs").unwrap();
    
    // Selectors for individual fields
    let title_selector = Selector::parse("h2[itemprop=\"headline\"]").unwrap();
    let url_selector = Selector::parse("a[itemprop=\"url\"]").unwrap();
    let thumbnail_selector = Selector::parse("img.ts-post-image").unwrap();
    let status_selector = Selector::parse("div.status").unwrap();
    let type_selector = Selector::parse("div.typez").unwrap();
    let episode_status_selector = Selector::parse("span.epx").unwrap();
    
    let mut results = Vec::new();
    
    // Try to find articles inside div.listupd first
    let articles: Vec<_> = if let Some(listupd) = document.select(&listupd_selector).next() {
        listupd.select(&article_selector).collect()
    } else {
        // Fallback: look for article.bs anywhere in the document
        document.select(&article_selector).collect()
    };
    
    for article in articles {
        let title = article
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let url = article
            .select(&url_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let thumbnail = article
            .select(&thumbnail_selector)
            .next()
            .and_then(|el| el.value().attr("src").or_else(|| el.value().attr("data-src")))
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let status = article
            .select(&status_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let anime_type = article
            .select(&type_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let episode_status = article
            .select(&episode_status_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        results.push(AnimeListItem {
            slug: extract_slug_from_url(&url),
            title,
            url,
            thumbnail,
            status,
            anime_type,
            episode_status,
        });
    }
    
    results
}

/// Parse anime detail from an anime detail page HTML
///
/// Extracts metadata from `div.bigcontent` and episode list from `div.eplister`
///
/// # Arguments
/// * `html` - The HTML content to parse
///
/// # Returns
/// An `AnimeDetail` struct with all extracted information
pub fn parse_anime_detail(html: &str) -> AnimeDetail {
    let document = Html::parse_document(html);
    
    // Selectors for metadata
    let title_selector = Selector::parse("h1.entry-title").unwrap();
    let alternate_titles_selector = Selector::parse("span.alter").unwrap();
    let poster_selector = Selector::parse("div.thumb img").unwrap();
    let rating_selector = Selector::parse("meta[itemprop=\"ratingValue\"]").unwrap();
    let trailer_selector = Selector::parse("a.trailerbutton").unwrap();
    let spe_span_selector = Selector::parse("div.spe span").unwrap();
    let casts_selector = Selector::parse("a.casts").unwrap();
    let genres_selector = Selector::parse("div.genxed a").unwrap();
    let synopsis_selector = Selector::parse("div.desc").unwrap();
    
    // Episode list selectors
    let episode_list_selector = Selector::parse("div.eplister ul li").unwrap();
    let episode_num_selector = Selector::parse("div.epl-num").unwrap();
    let episode_title_selector = Selector::parse("div.epl-title").unwrap();
    let episode_url_selector = Selector::parse("a").unwrap();
    let episode_date_selector = Selector::parse("div.epl-date").unwrap();
    
    // Extract title
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_default();
    
    // Extract alternate titles
    let alternate_titles = document
        .select(&alternate_titles_selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_default();
    
    // Extract poster image
    let poster = document
        .select(&poster_selector)
        .next()
        .and_then(|el| el.value().attr("src").or_else(|| el.value().attr("data-src")))
        .map(|s| s.to_string())
        .unwrap_or_default();
    
    // Extract rating from meta tag
    let rating = document
        .select(&rating_selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or_default();
    
    // Extract trailer URL
    let trailer_url = document
        .select(&trailer_selector)
        .next()
        .and_then(|el| el.value().attr("href"))
        .map(|s| s.to_string())
        .unwrap_or_default();
    
    // Extract metadata from div.spe span elements
    let mut status = String::new();
    let mut studio = String::new();
    let mut release_date = String::new();
    let mut duration = String::new();
    let mut season = String::new();
    let mut anime_type = String::new();
    let mut total_episodes = String::new();
    let mut director = String::new();
    
    for span in document.select(&spe_span_selector) {
        let text = span.text().collect::<String>();
        let text_lower = text.to_lowercase();
        
        // Extract value after colon
        let extract_value = |text: &str| -> String {
            text.split(':')
                .skip(1)
                .collect::<Vec<_>>()
                .join(":")
                .trim()
                .to_string()
        };
        
        if text_lower.contains("status") {
            status = extract_value(&text);
        } else if text_lower.contains("studio") {
            studio = extract_value(&text);
        } else if text_lower.contains("tanggal rilis") || text_lower.contains("release") || text_lower.contains("released") {
            release_date = extract_value(&text);
        } else if text_lower.contains("durasi") || text_lower.contains("duration") {
            duration = extract_value(&text);
        } else if text_lower.contains("season") {
            season = extract_value(&text);
        } else if text_lower.contains("tipe") || text_lower.contains("type") {
            anime_type = extract_value(&text);
        } else if text_lower.contains("total episode") || text_lower.contains("episodes") {
            total_episodes = extract_value(&text);
        } else if text_lower.contains("director") || text_lower.contains("sutradara") {
            director = extract_value(&text);
        }
    }
    
    // Extract casts
    let casts: Vec<String> = document
        .select(&casts_selector)
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    // Extract genres
    let genres: Vec<String> = document
        .select(&genres_selector)
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    // Extract synopsis
    let synopsis = document
        .select(&synopsis_selector)
        .next()
        .map(|el| {
            // Get all text content, preserving some structure
            el.text()
                .collect::<String>()
                .trim()
                .to_string()
        })
        .unwrap_or_default();
    
    // Extract episodes from div.eplister
    let mut episodes: Vec<Episode> = Vec::new();
    
    for li in document.select(&episode_list_selector) {
        let number = li
            .select(&episode_num_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let ep_title = li
            .select(&episode_title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let url = li
            .select(&episode_url_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let ep_release_date = li
            .select(&episode_date_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        episodes.push(Episode {
            slug: extract_slug_from_url(&url),
            number,
            title: ep_title,
            url,
            release_date: ep_release_date,
        });
    }
    
    AnimeDetail {
        title,
        alternate_titles,
        poster,
        rating,
        trailer_url,
        status,
        studio,
        release_date,
        duration,
        season,
        anime_type,
        total_episodes,
        director,
        casts,
        genres,
        synopsis,
        episodes,
    }
}

/// Parse episode list from HTML
///
/// Extracts episodes from `div.eplister ul li` elements.
/// Episodes are returned in the same order as they appear in the HTML (newest first).
///
/// # Arguments
/// * `html` - The HTML content to parse
///
/// # Returns
/// A vector of `Episode` structs in the order they appear in the HTML
pub fn parse_episode_list(html: &str) -> Vec<Episode> {
    let document = Html::parse_document(html);
    
    // Selectors for episode list
    let episode_list_selector = Selector::parse("div.eplister ul li").unwrap();
    let episode_num_selector = Selector::parse("div.epl-num").unwrap();
    let episode_title_selector = Selector::parse("div.epl-title").unwrap();
    let episode_url_selector = Selector::parse("a").unwrap();
    let episode_date_selector = Selector::parse("div.epl-date").unwrap();
    
    let mut episodes: Vec<Episode> = Vec::new();
    
    for li in document.select(&episode_list_selector) {
        let number = li
            .select(&episode_num_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let title = li
            .select(&episode_title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        let url = li
            .select(&episode_url_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let release_date = li
            .select(&episode_date_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        episodes.push(Episode {
            slug: extract_slug_from_url(&url),
            number,
            title,
            url,
            release_date,
        });
    }
    
    episodes
}

/// Parse video sources from an episode page HTML
///
/// Extracts video sources from `select.mirror option` elements.
/// Decodes base64-encoded values and extracts video URLs.
/// Also extracts the default video URL from `div#embed_holder video source`.
///
/// # Arguments
/// * `html` - The HTML content to parse
///
/// # Returns
/// An `EpisodeDetail` struct with title, default video, and all video sources
pub fn parse_episode_detail(html: &str) -> EpisodeDetail {
    let document = Html::parse_document(html);
    
    // Selectors
    let title_selector = Selector::parse("h1.entry-title").unwrap();
    let default_video_selector = Selector::parse("div#embed_holder video source").unwrap();
    let mirror_option_selector = Selector::parse("select.mirror option").unwrap();
    
    // Extract episode title
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_default();
    
    // Extract default video URL from div#embed_holder video source
    let default_video = document
        .select(&default_video_selector)
        .next()
        .and_then(|el| el.value().attr("src"))
        .map(|s| s.to_string())
        .unwrap_or_default();
    
    // Extract video sources from select.mirror option elements
    let mut sources: Vec<VideoSource> = Vec::new();
    
    for option in document.select(&mirror_option_selector) {
        // Get the base64-encoded value
        let value = match option.value().attr("value") {
            Some(v) if !v.is_empty() => v,
            _ => continue,
        };
        
        // Get the option text for server and quality info
        let option_text = option.text().collect::<String>().trim().to_string();
        
        // Parse server and quality from option text
        // Format is typically "SERVER - QUALITY" or "SERVER QUALITY" or just "SERVER"
        let (server, quality) = parse_server_quality(&option_text);
        
        // Decode base64 value
        let decoded_html = match decode_base64_value(value) {
            Some(html) => html,
            None => continue, // Skip invalid base64
        };
        
        // Extract video URL from decoded HTML
        let video_url = extract_video_url_from_html(&decoded_html);
        
        if !video_url.is_empty() {
            sources.push(VideoSource {
                server,
                quality,
                url: video_url,
            });
        }
    }
    
    EpisodeDetail {
        title,
        default_video,
        sources,
    }
}

/// Parse server name and quality from option text
///
/// Handles formats like:
/// - "SOKUJA - 720p"
/// - "SOKUJA 720p"
/// - "SOKUJA"
/// - "Server Name - 1080p HD"
fn parse_server_quality(text: &str) -> (String, String) {
    let text = text.trim();
    
    // Try to split by " - " first
    if let Some(idx) = text.find(" - ") {
        let server = text[..idx].trim().to_string();
        let quality = text[idx + 3..].trim().to_string();
        return (server, quality);
    }
    
    // Try to find quality pattern (e.g., "480p", "720p", "1080p")
    let quality_patterns = ["1080p", "720p", "480p", "360p", "240p"];
    for pattern in quality_patterns {
        if let Some(idx) = text.to_lowercase().find(&pattern.to_lowercase()) {
            let server = text[..idx].trim().to_string();
            let quality = text[idx..].trim().to_string();
            if !server.is_empty() {
                return (server, quality);
            }
            return (text.to_string(), quality);
        }
    }
    
    // No quality found, entire text is server name
    (text.to_string(), String::new())
}

/// Decode a base64-encoded value
///
/// Returns None if decoding fails (invalid base64)
fn decode_base64_value(value: &str) -> Option<String> {
    // Try to decode the base64 value
    let decoded_bytes = STANDARD.decode(value).ok()?;
    
    // Convert to UTF-8 string
    String::from_utf8(decoded_bytes).ok()
}

/// Extract video URL from decoded HTML content
///
/// Looks for video source elements or iframe src attributes
fn extract_video_url_from_html(html: &str) -> String {
    let document = Html::parse_fragment(html);
    
    // Try to find video source element
    if let Ok(source_selector) = Selector::parse("source") {
        if let Some(source) = document.select(&source_selector).next() {
            if let Some(src) = source.value().attr("src") {
                return src.to_string();
            }
        }
    }
    
    // Try to find video element with src
    if let Ok(video_selector) = Selector::parse("video") {
        if let Some(video) = document.select(&video_selector).next() {
            if let Some(src) = video.value().attr("src") {
                return src.to_string();
            }
        }
    }
    
    // Try to find iframe src
    if let Ok(iframe_selector) = Selector::parse("iframe") {
        if let Some(iframe) = document.select(&iframe_selector).next() {
            if let Some(src) = iframe.value().attr("src") {
                return src.to_string();
            }
        }
    }
    
    // Try to find embed src
    if let Ok(embed_selector) = Selector::parse("embed") {
        if let Some(embed) = document.select(&embed_selector).next() {
            if let Some(src) = embed.value().attr("src") {
                return src.to_string();
            }
        }
    }
    
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_anime_updates_empty_html() {
        let html = "<html><body></body></html>";
        let updates = parse_anime_updates(html);
        assert!(updates.is_empty());
    }

    #[test]
    fn test_parse_anime_updates_with_article() {
        let html = r#"
        <html>
        <body>
            <article class="seventh">
                <h2 itemprop="headline"><a href="/episode-1/">Test Anime Episode 1</a></h2>
                <a itemprop="url" href="/episode-1/"></a>
                <img class="ts-post-image" src="https://example.com/thumb.jpg" />
                <div class="epin">1/12</div>
                <span class="type">TV</span>
                <div class="sosev">
                    <span><a href="/anime/test-anime/">Test Anime</a></span>
                    <span>2 hours ago</span>
                </div>
                <span class="status">Ongoing</span>
            </article>
        </body>
        </html>
        "#;
        
        let updates = parse_anime_updates(html);
        assert_eq!(updates.len(), 1);
        
        let update = &updates[0];
        assert_eq!(update.title, "Test Anime Episode 1");
        assert_eq!(update.episode_url, "/episode-1/");
        assert_eq!(update.thumbnail, "https://example.com/thumb.jpg");
        assert_eq!(update.episode_number, "1/12");
        assert_eq!(update.anime_type, "TV");
        assert_eq!(update.series_title, "Test Anime");
        assert_eq!(update.series_url, "/anime/test-anime/");
        assert_eq!(update.status, "Ongoing");
    }

    #[test]
    fn test_parse_anime_updates_missing_elements() {
        let html = r#"
        <html>
        <body>
            <article class="seventh">
                <h2 itemprop="headline"><a href="/episode-1/">Test Anime</a></h2>
            </article>
        </body>
        </html>
        "#;
        
        let updates = parse_anime_updates(html);
        assert_eq!(updates.len(), 1);
        
        let update = &updates[0];
        assert_eq!(update.title, "Test Anime");
        // Missing elements should default to empty strings
        assert_eq!(update.thumbnail, "");
        assert_eq!(update.episode_number, "");
        assert_eq!(update.anime_type, "");
    }

    #[test]
    fn test_parse_completed_anime_empty_html() {
        let html = "<html><body></body></html>";
        let completed = parse_completed_anime(html);
        assert!(completed.is_empty());
    }

    #[test]
    fn test_parse_completed_anime_with_article() {
        let html = r#"
        <html>
        <body>
            <article class="stylesix">
                <h2 itemprop="headline"><a href="/anime/test-anime/">Test Anime Complete</a></h2>
                <a itemprop="url" href="/anime/test-anime/"></a>
                <img class="ts-post-image" src="https://example.com/thumb.jpg" />
                <div class="typez">TV</div>
                <span class="epx">24 Episodes</span>
                <span class="scr">8.5</span>
                <ul>
                    <li>Status: Completed</li>
                    <li>Dipos Oleh: Admin</li>
                    <li>Dipos pada: 2024-01-01</li>
                    <li><a href="/anime/test-anime/">Test Anime</a></li>
                </ul>
                <a rel="tag" href="/genre/action/">Action</a>
                <a rel="tag" href="/genre/adventure/">Adventure</a>
            </article>
        </body>
        </html>
        "#;
        
        let completed = parse_completed_anime(html);
        assert_eq!(completed.len(), 1);
        
        let anime = &completed[0];
        assert_eq!(anime.title, "Test Anime Complete");
        assert_eq!(anime.url, "/anime/test-anime/");
        assert_eq!(anime.thumbnail, "https://example.com/thumb.jpg");
        assert_eq!(anime.anime_type, "TV");
        assert_eq!(anime.episode_count, "24 Episodes");
        assert_eq!(anime.status, "Completed");
        assert_eq!(anime.posted_by, "Admin");
        assert_eq!(anime.posted_at, "2024-01-01");
        assert_eq!(anime.rating, "8.5");
        assert_eq!(anime.genres, vec!["Action", "Adventure"]);
    }

    #[test]
    fn test_parse_completed_anime_missing_elements() {
        let html = r#"
        <html>
        <body>
            <article class="stylesix">
                <h2 itemprop="headline"><a href="/anime/test/">Test</a></h2>
            </article>
        </body>
        </html>
        "#;
        
        let completed = parse_completed_anime(html);
        assert_eq!(completed.len(), 1);
        
        let anime = &completed[0];
        assert_eq!(anime.title, "Test");
        // Missing elements should default to empty strings/arrays
        assert_eq!(anime.thumbnail, "");
        assert_eq!(anime.anime_type, "");
        assert_eq!(anime.episode_count, "");
        assert!(anime.genres.is_empty());
    }

    #[test]
    fn test_anime_update_serialization() {
        let update = AnimeUpdate {
            slug: "test-series".to_string(),
            title: "Test".to_string(),
            episode_url: "/ep/1".to_string(),
            thumbnail: "https://example.com/img.jpg".to_string(),
            episode_number: "1".to_string(),
            anime_type: "TV".to_string(),
            series_title: "Test Series".to_string(),
            series_url: "/anime/test/".to_string(),
            status: "Ongoing".to_string(),
            release_info: "2 hours ago".to_string(),
        };
        
        let json = serde_json::to_string(&update).unwrap();
        // Verify camelCase serialization
        assert!(json.contains("\"episodeUrl\""));
        assert!(json.contains("\"seriesTitle\""));
        assert!(json.contains("\"releaseInfo\""));
        assert!(json.contains("\"type\"")); // anime_type should serialize as "type"
    }

    #[test]
    fn test_completed_anime_serialization() {
        let anime = CompletedAnime {
            slug: "test".to_string(),
            title: "Test".to_string(),
            url: "/anime/test/".to_string(),
            thumbnail: "https://example.com/img.jpg".to_string(),
            anime_type: "TV".to_string(),
            episode_count: "24".to_string(),
            status: "Completed".to_string(),
            posted_by: "Admin".to_string(),
            posted_at: "2024-01-01".to_string(),
            series_title: "Test".to_string(),
            series_url: "/anime/test/".to_string(),
            genres: vec!["Action".to_string()],
            rating: "8.5".to_string(),
        };
        
        let json = serde_json::to_string(&anime).unwrap();
        // Verify camelCase serialization
        assert!(json.contains("\"episodeCount\""));
        assert!(json.contains("\"postedBy\""));
        assert!(json.contains("\"postedAt\""));
        assert!(json.contains("\"seriesTitle\""));
        assert!(json.contains("\"type\"")); // anime_type should serialize as "type"
    }

    #[test]
    fn test_parse_search_results_empty_html() {
        let html = "<html><body></body></html>";
        let results = parse_search_results(html);
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_search_results_no_listupd() {
        // HTML without div.listupd should return empty results
        let html = r#"
        <html>
        <body>
            <div class="other">
                <article class="bs">
                    <h2 itemprop="headline">Test Anime</h2>
                </article>
            </div>
        </body>
        </html>
        "#;
        
        let results = parse_search_results(html);
        // Should still find article.bs even without div.listupd (fallback behavior)
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_parse_search_results_with_listupd() {
        let html = r#"
        <html>
        <body>
            <div class="listupd">
                <article class="bs">
                    <h2 itemprop="headline">Naruto Shippuden</h2>
                    <a itemprop="url" href="/anime/naruto-shippuden/"></a>
                    <img class="ts-post-image" src="https://example.com/naruto.jpg" />
                    <div class="status">Completed</div>
                    <div class="typez">TV</div>
                    <span class="epx">500 Episodes</span>
                </article>
            </div>
        </body>
        </html>
        "#;
        
        let results = parse_search_results(html);
        assert_eq!(results.len(), 1);
        
        let result = &results[0];
        assert_eq!(result.title, "Naruto Shippuden");
        assert_eq!(result.url, "/anime/naruto-shippuden/");
        assert_eq!(result.thumbnail, "https://example.com/naruto.jpg");
        assert_eq!(result.status, "Completed");
        assert_eq!(result.anime_type, "TV");
        assert_eq!(result.episode_status, "500 Episodes");
    }

    #[test]
    fn test_parse_search_results_multiple_results() {
        let html = r#"
        <html>
        <body>
            <div class="listupd">
                <article class="bs">
                    <h2 itemprop="headline">Anime One</h2>
                    <a itemprop="url" href="/anime/anime-one/"></a>
                    <img class="ts-post-image" src="https://example.com/one.jpg" />
                    <div class="status">Ongoing</div>
                    <div class="typez">TV</div>
                    <span class="epx">12 Episodes</span>
                </article>
                <article class="bs">
                    <h2 itemprop="headline">Anime Two</h2>
                    <a itemprop="url" href="/anime/anime-two/"></a>
                    <img class="ts-post-image" src="https://example.com/two.jpg" />
                    <div class="status">Completed</div>
                    <div class="typez">Movie</div>
                    <span class="epx">1 Episode</span>
                </article>
            </div>
        </body>
        </html>
        "#;
        
        let results = parse_search_results(html);
        assert_eq!(results.len(), 2);
        
        assert_eq!(results[0].title, "Anime One");
        assert_eq!(results[0].anime_type, "TV");
        
        assert_eq!(results[1].title, "Anime Two");
        assert_eq!(results[1].anime_type, "Movie");
    }

    #[test]
    fn test_parse_search_results_missing_elements() {
        let html = r#"
        <html>
        <body>
            <div class="listupd">
                <article class="bs">
                    <h2 itemprop="headline">Minimal Anime</h2>
                </article>
            </div>
        </body>
        </html>
        "#;
        
        let results = parse_search_results(html);
        assert_eq!(results.len(), 1);
        
        let result = &results[0];
        assert_eq!(result.title, "Minimal Anime");
        // Missing elements should default to empty strings
        assert_eq!(result.url, "");
        assert_eq!(result.thumbnail, "");
        assert_eq!(result.status, "");
        assert_eq!(result.anime_type, "");
        assert_eq!(result.episode_status, "");
    }

    #[test]
    fn test_parse_search_results_with_data_src() {
        // Test that data-src attribute is used as fallback for thumbnail
        let html = r#"
        <html>
        <body>
            <div class="listupd">
                <article class="bs">
                    <h2 itemprop="headline">Lazy Load Anime</h2>
                    <a itemprop="url" href="/anime/lazy/"></a>
                    <img class="ts-post-image" data-src="https://example.com/lazy.jpg" />
                    <div class="status">Ongoing</div>
                    <div class="typez">ONA</div>
                    <span class="epx">Ongoing</span>
                </article>
            </div>
        </body>
        </html>
        "#;
        
        let results = parse_search_results(html);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].thumbnail, "https://example.com/lazy.jpg");
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            slug: "test".to_string(),
            title: "Test Anime".to_string(),
            url: "/anime/test/".to_string(),
            thumbnail: "https://example.com/img.jpg".to_string(),
            status: "Ongoing".to_string(),
            anime_type: "TV".to_string(),
            episode_status: "12 Episodes".to_string(),
        };
        
        let json = serde_json::to_string(&result).unwrap();
        // Verify camelCase serialization
        assert!(json.contains("\"title\""));
        assert!(json.contains("\"url\""));
        assert!(json.contains("\"thumbnail\""));
        assert!(json.contains("\"status\""));
        assert!(json.contains("\"type\"")); // anime_type should serialize as "type"
        assert!(json.contains("\"episodeStatus\""));
    }

    // Tests for parse_anime_list function

    #[test]
    fn test_parse_anime_list_empty_html() {
        let html = "<html><body></body></html>";
        let results = parse_anime_list(html);
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_anime_list_with_listupd() {
        let html = r#"
        <html>
        <body>
            <div class="listupd">
                <article class="bs">
                    <h2 itemprop="headline">One Piece</h2>
                    <a itemprop="url" href="/anime/one-piece/"></a>
                    <img class="ts-post-image" src="https://example.com/onepiece.jpg" />
                    <div class="status">Ongoing</div>
                    <div class="typez">TV</div>
                    <span class="epx">1000+ Episodes</span>
                </article>
            </div>
        </body>
        </html>
        "#;
        
        let results = parse_anime_list(html);
        assert_eq!(results.len(), 1);
        
        let item = &results[0];
        assert_eq!(item.title, "One Piece");
        assert_eq!(item.url, "/anime/one-piece/");
        assert_eq!(item.thumbnail, "https://example.com/onepiece.jpg");
        assert_eq!(item.status, "Ongoing");
        assert_eq!(item.anime_type, "TV");
        assert_eq!(item.episode_status, "1000+ Episodes");
    }

    #[test]
    fn test_parse_anime_list_multiple_items() {
        let html = r#"
        <html>
        <body>
            <div class="listupd">
                <article class="bs">
                    <h2 itemprop="headline">Anime A</h2>
                    <a itemprop="url" href="/anime/anime-a/"></a>
                    <img class="ts-post-image" src="https://example.com/a.jpg" />
                    <div class="status">Completed</div>
                    <div class="typez">Movie</div>
                    <span class="epx">1 Episode</span>
                </article>
                <article class="bs">
                    <h2 itemprop="headline">Anime B</h2>
                    <a itemprop="url" href="/anime/anime-b/"></a>
                    <img class="ts-post-image" src="https://example.com/b.jpg" />
                    <div class="status">Ongoing</div>
                    <div class="typez">ONA</div>
                    <span class="epx">24 Episodes</span>
                </article>
                <article class="bs">
                    <h2 itemprop="headline">Anime C</h2>
                    <a itemprop="url" href="/anime/anime-c/"></a>
                    <img class="ts-post-image" src="https://example.com/c.jpg" />
                    <div class="status">Upcoming</div>
                    <div class="typez">TV</div>
                    <span class="epx">TBA</span>
                </article>
            </div>
        </body>
        </html>
        "#;
        
        let results = parse_anime_list(html);
        assert_eq!(results.len(), 3);
        
        assert_eq!(results[0].title, "Anime A");
        assert_eq!(results[0].anime_type, "Movie");
        
        assert_eq!(results[1].title, "Anime B");
        assert_eq!(results[1].anime_type, "ONA");
        
        assert_eq!(results[2].title, "Anime C");
        assert_eq!(results[2].status, "Upcoming");
    }

    #[test]
    fn test_parse_anime_list_missing_elements() {
        let html = r#"
        <html>
        <body>
            <div class="listupd">
                <article class="bs">
                    <h2 itemprop="headline">Minimal Anime</h2>
                </article>
            </div>
        </body>
        </html>
        "#;
        
        let results = parse_anime_list(html);
        assert_eq!(results.len(), 1);
        
        let item = &results[0];
        assert_eq!(item.title, "Minimal Anime");
        // Missing elements should default to empty strings
        assert_eq!(item.url, "");
        assert_eq!(item.thumbnail, "");
        assert_eq!(item.status, "");
        assert_eq!(item.anime_type, "");
        assert_eq!(item.episode_status, "");
    }

    #[test]
    fn test_parse_anime_list_with_data_src() {
        // Test that data-src attribute is used as fallback for thumbnail
        let html = r#"
        <html>
        <body>
            <div class="listupd">
                <article class="bs">
                    <h2 itemprop="headline">Lazy Load Anime</h2>
                    <a itemprop="url" href="/anime/lazy/"></a>
                    <img class="ts-post-image" data-src="https://example.com/lazy.jpg" />
                    <div class="status">Ongoing</div>
                    <div class="typez">TV</div>
                    <span class="epx">12 Episodes</span>
                </article>
            </div>
        </body>
        </html>
        "#;
        
        let results = parse_anime_list(html);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].thumbnail, "https://example.com/lazy.jpg");
    }

    #[test]
    fn test_parse_anime_list_fallback_without_listupd() {
        // Test fallback behavior when div.listupd is not present
        let html = r#"
        <html>
        <body>
            <div class="other-container">
                <article class="bs">
                    <h2 itemprop="headline">Fallback Anime</h2>
                    <a itemprop="url" href="/anime/fallback/"></a>
                    <img class="ts-post-image" src="https://example.com/fallback.jpg" />
                    <div class="status">Completed</div>
                    <div class="typez">Special</div>
                    <span class="epx">3 Episodes</span>
                </article>
            </div>
        </body>
        </html>
        "#;
        
        let results = parse_anime_list(html);
        // Should still find article.bs even without div.listupd (fallback behavior)
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Fallback Anime");
    }

    #[test]
    fn test_anime_list_item_serialization() {
        let item = AnimeListItem {
            slug: "test".to_string(),
            title: "Test Anime".to_string(),
            url: "/anime/test/".to_string(),
            thumbnail: "https://example.com/img.jpg".to_string(),
            status: "Ongoing".to_string(),
            anime_type: "TV".to_string(),
            episode_status: "12 Episodes".to_string(),
        };
        
        let json = serde_json::to_string(&item).unwrap();
        // Verify camelCase serialization
        assert!(json.contains("\"title\""));
        assert!(json.contains("\"url\""));
        assert!(json.contains("\"thumbnail\""));
        assert!(json.contains("\"status\""));
        assert!(json.contains("\"type\"")); // anime_type should serialize as "type"
        assert!(json.contains("\"episodeStatus\""));
    }

    // Tests for parse_anime_detail function

    #[test]
    fn test_parse_anime_detail_empty_html() {
        let html = "<html><body></body></html>";
        let detail = parse_anime_detail(html);
        
        // All fields should be empty strings or empty arrays
        assert_eq!(detail.title, "");
        assert_eq!(detail.alternate_titles, "");
        assert_eq!(detail.poster, "");
        assert_eq!(detail.rating, "");
        assert_eq!(detail.trailer_url, "");
        assert_eq!(detail.status, "");
        assert_eq!(detail.studio, "");
        assert_eq!(detail.release_date, "");
        assert_eq!(detail.duration, "");
        assert_eq!(detail.season, "");
        assert_eq!(detail.anime_type, "");
        assert_eq!(detail.total_episodes, "");
        assert_eq!(detail.director, "");
        assert!(detail.casts.is_empty());
        assert!(detail.genres.is_empty());
        assert_eq!(detail.synopsis, "");
        assert!(detail.episodes.is_empty());
    }

    #[test]
    fn test_parse_anime_detail_full() {
        let html = r#"
        <html>
        <body>
            <h1 class="entry-title">Naruto Shippuden</h1>
            <span class="alter">Naruto: Hurricane Chronicles, ナルト 疾風伝</span>
            <div class="thumb">
                <img src="https://example.com/naruto-poster.jpg" />
            </div>
            <meta itemprop="ratingValue" content="8.7" />
            <a class="trailerbutton" href="https://youtube.com/watch?v=abc123">Watch Trailer</a>
            <div class="spe">
                <span>Status: Completed</span>
                <span>Studio: Pierrot</span>
                <span>Tanggal Rilis: Oct 28, 2007</span>
                <span>Durasi: 23 min per ep</span>
                <span>Season: Fall 2007</span>
                <span>Tipe: TV</span>
                <span>Total Episode: 500</span>
                <span>Director: Hayato Date</span>
            </div>
            <a class="casts">Junko Takeuchi</a>
            <a class="casts">Noriaki Sugiyama</a>
            <a class="casts">Chie Nakamura</a>
            <div class="genxed">
                <a href="/genre/action/">Action</a>
                <a href="/genre/adventure/">Adventure</a>
                <a href="/genre/martial-arts/">Martial Arts</a>
            </div>
            <div class="desc">
                Naruto Uzumaki, is a loud, hyperactive, adolescent ninja who constantly searches for approval and recognition.
            </div>
            <div class="eplister">
                <ul>
                    <li>
                        <a href="/naruto-shippuden-episode-500/">
                            <div class="epl-num">500</div>
                            <div class="epl-title">The Message</div>
                            <div class="epl-date">Mar 23, 2017</div>
                        </a>
                    </li>
                    <li>
                        <a href="/naruto-shippuden-episode-499/">
                            <div class="epl-num">499</div>
                            <div class="epl-title">The Outcome of the Secret Mission</div>
                            <div class="epl-date">Mar 16, 2017</div>
                        </a>
                    </li>
                </ul>
            </div>
        </body>
        </html>
        "#;
        
        let detail = parse_anime_detail(html);
        
        assert_eq!(detail.title, "Naruto Shippuden");
        assert_eq!(detail.alternate_titles, "Naruto: Hurricane Chronicles, ナルト 疾風伝");
        assert_eq!(detail.poster, "https://example.com/naruto-poster.jpg");
        assert_eq!(detail.rating, "8.7");
        assert_eq!(detail.trailer_url, "https://youtube.com/watch?v=abc123");
        assert_eq!(detail.status, "Completed");
        assert_eq!(detail.studio, "Pierrot");
        assert_eq!(detail.release_date, "Oct 28, 2007");
        assert_eq!(detail.duration, "23 min per ep");
        assert_eq!(detail.season, "Fall 2007");
        assert_eq!(detail.anime_type, "TV");
        assert_eq!(detail.total_episodes, "500");
        assert_eq!(detail.director, "Hayato Date");
        assert_eq!(detail.casts, vec!["Junko Takeuchi", "Noriaki Sugiyama", "Chie Nakamura"]);
        assert_eq!(detail.genres, vec!["Action", "Adventure", "Martial Arts"]);
        assert!(detail.synopsis.contains("Naruto Uzumaki"));
        
        // Check episodes
        assert_eq!(detail.episodes.len(), 2);
        assert_eq!(detail.episodes[0].number, "500");
        assert_eq!(detail.episodes[0].title, "The Message");
        assert_eq!(detail.episodes[0].url, "/naruto-shippuden-episode-500/");
        assert_eq!(detail.episodes[0].release_date, "Mar 23, 2017");
        
        assert_eq!(detail.episodes[1].number, "499");
        assert_eq!(detail.episodes[1].title, "The Outcome of the Secret Mission");
    }

    #[test]
    fn test_parse_anime_detail_missing_optional_fields() {
        let html = r#"
        <html>
        <body>
            <h1 class="entry-title">Minimal Anime</h1>
            <div class="thumb">
                <img src="https://example.com/poster.jpg" />
            </div>
            <div class="spe">
                <span>Status: Ongoing</span>
                <span>Tipe: TV</span>
            </div>
        </body>
        </html>
        "#;
        
        let detail = parse_anime_detail(html);
        
        // Required fields should be present
        assert_eq!(detail.title, "Minimal Anime");
        assert_eq!(detail.poster, "https://example.com/poster.jpg");
        assert_eq!(detail.status, "Ongoing");
        assert_eq!(detail.anime_type, "TV");
        
        // Optional fields should be empty strings
        assert_eq!(detail.alternate_titles, "");
        assert_eq!(detail.rating, "");
        assert_eq!(detail.trailer_url, "");
        assert_eq!(detail.studio, "");
        assert_eq!(detail.release_date, "");
        assert_eq!(detail.duration, "");
        assert_eq!(detail.season, "");
        assert_eq!(detail.total_episodes, "");
        assert_eq!(detail.director, "");
        assert!(detail.casts.is_empty());
        assert!(detail.genres.is_empty());
        assert_eq!(detail.synopsis, "");
        assert!(detail.episodes.is_empty());
    }

    #[test]
    fn test_parse_anime_detail_with_data_src_poster() {
        let html = r#"
        <html>
        <body>
            <h1 class="entry-title">Lazy Load Anime</h1>
            <div class="thumb">
                <img data-src="https://example.com/lazy-poster.jpg" />
            </div>
        </body>
        </html>
        "#;
        
        let detail = parse_anime_detail(html);
        assert_eq!(detail.poster, "https://example.com/lazy-poster.jpg");
    }

    #[test]
    fn test_parse_anime_detail_episodes_order() {
        let html = r#"
        <html>
        <body>
            <h1 class="entry-title">Test Anime</h1>
            <div class="eplister">
                <ul>
                    <li>
                        <a href="/ep-3/">
                            <div class="epl-num">3</div>
                            <div class="epl-title">Episode 3</div>
                            <div class="epl-date">Jan 3, 2024</div>
                        </a>
                    </li>
                    <li>
                        <a href="/ep-2/">
                            <div class="epl-num">2</div>
                            <div class="epl-title">Episode 2</div>
                            <div class="epl-date">Jan 2, 2024</div>
                        </a>
                    </li>
                    <li>
                        <a href="/ep-1/">
                            <div class="epl-num">1</div>
                            <div class="epl-title">Episode 1</div>
                            <div class="epl-date">Jan 1, 2024</div>
                        </a>
                    </li>
                </ul>
            </div>
        </body>
        </html>
        "#;
        
        let detail = parse_anime_detail(html);
        
        // Episodes should be in the same order as HTML (newest first)
        assert_eq!(detail.episodes.len(), 3);
        assert_eq!(detail.episodes[0].number, "3");
        assert_eq!(detail.episodes[1].number, "2");
        assert_eq!(detail.episodes[2].number, "1");
    }

    #[test]
    fn test_anime_detail_serialization() {
        let detail = AnimeDetail {
            title: "Test Anime".to_string(),
            alternate_titles: "Alt Title".to_string(),
            poster: "https://example.com/poster.jpg".to_string(),
            rating: "8.5".to_string(),
            trailer_url: "https://youtube.com/watch?v=123".to_string(),
            status: "Ongoing".to_string(),
            studio: "Test Studio".to_string(),
            release_date: "Jan 1, 2024".to_string(),
            duration: "24 min".to_string(),
            season: "Winter 2024".to_string(),
            anime_type: "TV".to_string(),
            total_episodes: "12".to_string(),
            director: "Test Director".to_string(),
            casts: vec!["Actor 1".to_string(), "Actor 2".to_string()],
            genres: vec!["Action".to_string(), "Adventure".to_string()],
            synopsis: "Test synopsis".to_string(),
            episodes: vec![
                Episode {
                    slug: "ep-1".to_string(),
                    number: "1".to_string(),
                    title: "Episode 1".to_string(),
                    url: "/ep-1/".to_string(),
                    release_date: "Jan 1, 2024".to_string(),
                },
            ],
        };
        
        let json = serde_json::to_string(&detail).unwrap();
        
        // Verify camelCase serialization
        assert!(json.contains("\"alternateTitles\""));
        assert!(json.contains("\"trailerUrl\""));
        assert!(json.contains("\"releaseDate\""));
        assert!(json.contains("\"totalEpisodes\""));
        assert!(json.contains("\"type\"")); // anime_type should serialize as "type"
    }

    #[test]
    fn test_episode_serialization() {
        let episode = Episode {
            slug: "episode-1".to_string(),
            number: "1".to_string(),
            title: "First Episode".to_string(),
            url: "/anime/test/episode-1/".to_string(),
            release_date: "Jan 1, 2024".to_string(),
        };
        
        let json = serde_json::to_string(&episode).unwrap();
        
        // Verify camelCase serialization
        assert!(json.contains("\"slug\""));
        assert!(json.contains("\"number\""));
        assert!(json.contains("\"title\""));
        assert!(json.contains("\"url\""));
        assert!(json.contains("\"releaseDate\""));
    }

    // Tests for parse_episode_list function

    #[test]
    fn test_parse_episode_list_empty_html() {
        let html = "<html><body></body></html>";
        let episodes = parse_episode_list(html);
        assert!(episodes.is_empty());
    }

    #[test]
    fn test_parse_episode_list_no_eplister() {
        let html = r#"
        <html>
        <body>
            <div class="other">
                <ul>
                    <li>Some content</li>
                </ul>
            </div>
        </body>
        </html>
        "#;
        
        let episodes = parse_episode_list(html);
        assert!(episodes.is_empty());
    }

    #[test]
    fn test_parse_episode_list_single_episode() {
        let html = r#"
        <html>
        <body>
            <div class="eplister">
                <ul>
                    <li>
                        <a href="/anime-episode-1/">
                            <div class="epl-num">1</div>
                            <div class="epl-title">The Beginning</div>
                            <div class="epl-date">Jan 1, 2024</div>
                        </a>
                    </li>
                </ul>
            </div>
        </body>
        </html>
        "#;
        
        let episodes = parse_episode_list(html);
        assert_eq!(episodes.len(), 1);
        
        let ep = &episodes[0];
        assert_eq!(ep.number, "1");
        assert_eq!(ep.title, "The Beginning");
        assert_eq!(ep.url, "/anime-episode-1/");
        assert_eq!(ep.release_date, "Jan 1, 2024");
    }

    #[test]
    fn test_parse_episode_list_multiple_episodes() {
        let html = r#"
        <html>
        <body>
            <div class="eplister">
                <ul>
                    <li>
                        <a href="/anime-episode-3/">
                            <div class="epl-num">3</div>
                            <div class="epl-title">Episode Three</div>
                            <div class="epl-date">Jan 15, 2024</div>
                        </a>
                    </li>
                    <li>
                        <a href="/anime-episode-2/">
                            <div class="epl-num">2</div>
                            <div class="epl-title">Episode Two</div>
                            <div class="epl-date">Jan 8, 2024</div>
                        </a>
                    </li>
                    <li>
                        <a href="/anime-episode-1/">
                            <div class="epl-num">1</div>
                            <div class="epl-title">Episode One</div>
                            <div class="epl-date">Jan 1, 2024</div>
                        </a>
                    </li>
                </ul>
            </div>
        </body>
        </html>
        "#;
        
        let episodes = parse_episode_list(html);
        assert_eq!(episodes.len(), 3);
        
        // Episodes should maintain order (newest first as they appear in HTML)
        assert_eq!(episodes[0].number, "3");
        assert_eq!(episodes[0].title, "Episode Three");
        
        assert_eq!(episodes[1].number, "2");
        assert_eq!(episodes[1].title, "Episode Two");
        
        assert_eq!(episodes[2].number, "1");
        assert_eq!(episodes[2].title, "Episode One");
    }

    #[test]
    fn test_parse_episode_list_order_preservation() {
        // Test that episodes are returned in the same order as HTML (newest first)
        let html = r#"
        <html>
        <body>
            <div class="eplister">
                <ul>
                    <li>
                        <a href="/ep-10/">
                            <div class="epl-num">10</div>
                            <div class="epl-title">Latest Episode</div>
                            <div class="epl-date">Mar 1, 2024</div>
                        </a>
                    </li>
                    <li>
                        <a href="/ep-9/">
                            <div class="epl-num">9</div>
                            <div class="epl-title">Previous Episode</div>
                            <div class="epl-date">Feb 22, 2024</div>
                        </a>
                    </li>
                    <li>
                        <a href="/ep-8/">
                            <div class="epl-num">8</div>
                            <div class="epl-title">Older Episode</div>
                            <div class="epl-date">Feb 15, 2024</div>
                        </a>
                    </li>
                </ul>
            </div>
        </body>
        </html>
        "#;
        
        let episodes = parse_episode_list(html);
        
        // Verify order is preserved (newest first)
        assert_eq!(episodes.len(), 3);
        assert_eq!(episodes[0].number, "10");
        assert_eq!(episodes[1].number, "9");
        assert_eq!(episodes[2].number, "8");
    }

    #[test]
    fn test_parse_episode_list_missing_elements() {
        let html = r#"
        <html>
        <body>
            <div class="eplister">
                <ul>
                    <li>
                        <a href="/anime-episode-1/">
                            <div class="epl-num">1</div>
                        </a>
                    </li>
                </ul>
            </div>
        </body>
        </html>
        "#;
        
        let episodes = parse_episode_list(html);
        assert_eq!(episodes.len(), 1);
        
        let ep = &episodes[0];
        assert_eq!(ep.number, "1");
        assert_eq!(ep.url, "/anime-episode-1/");
        // Missing elements should default to empty strings
        assert_eq!(ep.title, "");
        assert_eq!(ep.release_date, "");
    }

    #[test]
    fn test_parse_episode_list_empty_eplister() {
        let html = r#"
        <html>
        <body>
            <div class="eplister">
                <ul>
                </ul>
            </div>
        </body>
        </html>
        "#;
        
        let episodes = parse_episode_list(html);
        assert!(episodes.is_empty());
    }

    #[test]
    fn test_parse_episode_list_special_characters() {
        let html = r#"
        <html>
        <body>
            <div class="eplister">
                <ul>
                    <li>
                        <a href="/anime-episode-1/">
                            <div class="epl-num">1</div>
                            <div class="epl-title">Episode 1: The Beginning &amp; The End</div>
                            <div class="epl-date">Jan 1, 2024</div>
                        </a>
                    </li>
                </ul>
            </div>
        </body>
        </html>
        "#;
        
        let episodes = parse_episode_list(html);
        assert_eq!(episodes.len(), 1);
        assert_eq!(episodes[0].title, "Episode 1: The Beginning & The End");
    }

    #[test]
    fn test_parse_episode_list_whitespace_handling() {
        let html = r#"
        <html>
        <body>
            <div class="eplister">
                <ul>
                    <li>
                        <a href="/anime-episode-1/">
                            <div class="epl-num">  1  </div>
                            <div class="epl-title">  Episode Title  </div>
                            <div class="epl-date">  Jan 1, 2024  </div>
                        </a>
                    </li>
                </ul>
            </div>
        </body>
        </html>
        "#;
        
        let episodes = parse_episode_list(html);
        assert_eq!(episodes.len(), 1);
        
        // Whitespace should be trimmed
        assert_eq!(episodes[0].number, "1");
        assert_eq!(episodes[0].title, "Episode Title");
        assert_eq!(episodes[0].release_date, "Jan 1, 2024");
    }

    // Tests for parse_episode_detail (video sources parser)

    #[test]
    fn test_parse_episode_detail_empty_html() {
        let html = "<html><body></body></html>";
        let detail = parse_episode_detail(html);
        
        assert_eq!(detail.title, "");
        assert_eq!(detail.default_video, "");
        assert!(detail.sources.is_empty());
    }

    #[test]
    fn test_parse_episode_detail_with_title() {
        let html = r#"
        <html>
        <body>
            <h1 class="entry-title">Naruto Episode 1</h1>
        </body>
        </html>
        "#;
        
        let detail = parse_episode_detail(html);
        assert_eq!(detail.title, "Naruto Episode 1");
        assert_eq!(detail.default_video, "");
        assert!(detail.sources.is_empty());
    }

    #[test]
    fn test_parse_episode_detail_with_default_video() {
        let html = r#"
        <html>
        <body>
            <h1 class="entry-title">Test Episode</h1>
            <div id="embed_holder">
                <video>
                    <source src="https://example.com/video.mp4" type="video/mp4" />
                </video>
            </div>
        </body>
        </html>
        "#;
        
        let detail = parse_episode_detail(html);
        assert_eq!(detail.title, "Test Episode");
        assert_eq!(detail.default_video, "https://example.com/video.mp4");
    }

    #[test]
    fn test_parse_episode_detail_with_video_sources() {
        // Base64 encode: <source src="https://example.com/720p.mp4" />
        let encoded_720p = base64::engine::general_purpose::STANDARD.encode(
            r#"<source src="https://example.com/720p.mp4" />"#
        );
        // Base64 encode: <source src="https://example.com/480p.mp4" />
        let encoded_480p = base64::engine::general_purpose::STANDARD.encode(
            r#"<source src="https://example.com/480p.mp4" />"#
        );
        
        let html = format!(r#"
        <html>
        <body>
            <h1 class="entry-title">Test Episode</h1>
            <select class="mirror">
                <option value="">Select Server</option>
                <option value="{encoded_720p}">SOKUJA - 720p</option>
                <option value="{encoded_480p}">SOKUJA - 480p</option>
            </select>
        </body>
        </html>
        "#);
        
        let detail = parse_episode_detail(&html);
        assert_eq!(detail.title, "Test Episode");
        assert_eq!(detail.sources.len(), 2);
        
        assert_eq!(detail.sources[0].server, "SOKUJA");
        assert_eq!(detail.sources[0].quality, "720p");
        assert_eq!(detail.sources[0].url, "https://example.com/720p.mp4");
        
        assert_eq!(detail.sources[1].server, "SOKUJA");
        assert_eq!(detail.sources[1].quality, "480p");
        assert_eq!(detail.sources[1].url, "https://example.com/480p.mp4");
    }

    #[test]
    fn test_parse_episode_detail_with_iframe_source() {
        // Base64 encode: <iframe src="https://player.example.com/embed/123" />
        let encoded = base64::engine::general_purpose::STANDARD.encode(
            r#"<iframe src="https://player.example.com/embed/123" />"#
        );
        
        let html = format!(r#"
        <html>
        <body>
            <h1 class="entry-title">Test Episode</h1>
            <select class="mirror">
                <option value="{encoded}">External Player - 1080p</option>
            </select>
        </body>
        </html>
        "#);
        
        let detail = parse_episode_detail(&html);
        assert_eq!(detail.sources.len(), 1);
        assert_eq!(detail.sources[0].server, "External Player");
        assert_eq!(detail.sources[0].quality, "1080p");
        assert_eq!(detail.sources[0].url, "https://player.example.com/embed/123");
    }

    #[test]
    fn test_parse_episode_detail_invalid_base64() {
        let html = r#"
        <html>
        <body>
            <h1 class="entry-title">Test Episode</h1>
            <select class="mirror">
                <option value="not-valid-base64!!!">Invalid Server</option>
                <option value="">Empty Value</option>
            </select>
        </body>
        </html>
        "#;
        
        let detail = parse_episode_detail(html);
        assert_eq!(detail.title, "Test Episode");
        // Invalid base64 should be skipped gracefully
        assert!(detail.sources.is_empty());
    }

    #[test]
    fn test_parse_episode_detail_mixed_valid_invalid() {
        // One valid, one invalid
        let encoded_valid = base64::engine::general_purpose::STANDARD.encode(
            r#"<source src="https://example.com/valid.mp4" />"#
        );
        
        let html = format!(r#"
        <html>
        <body>
            <h1 class="entry-title">Test Episode</h1>
            <select class="mirror">
                <option value="invalid-base64">Bad Server</option>
                <option value="{encoded_valid}">Good Server - 720p</option>
            </select>
        </body>
        </html>
        "#);
        
        let detail = parse_episode_detail(&html);
        // Only valid source should be parsed
        assert_eq!(detail.sources.len(), 1);
        assert_eq!(detail.sources[0].url, "https://example.com/valid.mp4");
    }

    #[test]
    fn test_parse_episode_detail_no_url_in_decoded_html() {
        // Base64 encode HTML without any video URL
        let encoded = base64::engine::general_purpose::STANDARD.encode(
            r#"<div>No video here</div>"#
        );
        
        let html = format!(r#"
        <html>
        <body>
            <h1 class="entry-title">Test Episode</h1>
            <select class="mirror">
                <option value="{encoded}">Empty Server</option>
            </select>
        </body>
        </html>
        "#);
        
        let detail = parse_episode_detail(&html);
        // Source without URL should be skipped
        assert!(detail.sources.is_empty());
    }

    #[test]
    fn test_parse_server_quality_with_dash() {
        let (server, quality) = parse_server_quality("SOKUJA - 720p");
        assert_eq!(server, "SOKUJA");
        assert_eq!(quality, "720p");
    }

    #[test]
    fn test_parse_server_quality_with_space() {
        let (server, quality) = parse_server_quality("SOKUJA 720p");
        assert_eq!(server, "SOKUJA");
        assert_eq!(quality, "720p");
    }

    #[test]
    fn test_parse_server_quality_no_quality() {
        let (server, quality) = parse_server_quality("SOKUJA");
        assert_eq!(server, "SOKUJA");
        assert_eq!(quality, "");
    }

    #[test]
    fn test_parse_server_quality_complex() {
        let (server, quality) = parse_server_quality("Server Name - 1080p HD");
        assert_eq!(server, "Server Name");
        assert_eq!(quality, "1080p HD");
    }

    #[test]
    fn test_parse_server_quality_lowercase() {
        let (server, quality) = parse_server_quality("server 480p");
        assert_eq!(server, "server");
        assert_eq!(quality, "480p");
    }

    #[test]
    fn test_decode_base64_value_valid() {
        let encoded = base64::engine::general_purpose::STANDARD.encode("Hello World");
        let decoded = decode_base64_value(&encoded);
        assert_eq!(decoded, Some("Hello World".to_string()));
    }

    #[test]
    fn test_decode_base64_value_invalid() {
        let decoded = decode_base64_value("not-valid-base64!!!");
        assert!(decoded.is_none());
    }

    #[test]
    fn test_decode_base64_value_empty() {
        let decoded = decode_base64_value("");
        // Empty string is valid base64 that decodes to empty
        assert_eq!(decoded, Some("".to_string()));
    }

    #[test]
    fn test_extract_video_url_from_source() {
        let html = r#"<source src="https://example.com/video.mp4" type="video/mp4" />"#;
        let url = extract_video_url_from_html(html);
        assert_eq!(url, "https://example.com/video.mp4");
    }

    #[test]
    fn test_extract_video_url_from_video() {
        let html = r#"<video src="https://example.com/video.mp4"></video>"#;
        let url = extract_video_url_from_html(html);
        assert_eq!(url, "https://example.com/video.mp4");
    }

    #[test]
    fn test_extract_video_url_from_iframe() {
        let html = r#"<iframe src="https://player.example.com/embed/123"></iframe>"#;
        let url = extract_video_url_from_html(html);
        assert_eq!(url, "https://player.example.com/embed/123");
    }

    #[test]
    fn test_extract_video_url_from_embed() {
        let html = r#"<embed src="https://example.com/player.swf" />"#;
        let url = extract_video_url_from_html(html);
        assert_eq!(url, "https://example.com/player.swf");
    }

    #[test]
    fn test_extract_video_url_no_url() {
        let html = r#"<div>No video here</div>"#;
        let url = extract_video_url_from_html(html);
        assert_eq!(url, "");
    }

    #[test]
    fn test_video_source_serialization() {
        let source = VideoSource {
            server: "SOKUJA".to_string(),
            quality: "720p".to_string(),
            url: "https://example.com/video.mp4".to_string(),
        };
        
        let json = serde_json::to_string(&source).unwrap();
        // Verify camelCase serialization
        assert!(json.contains("\"server\""));
        assert!(json.contains("\"quality\""));
        assert!(json.contains("\"url\""));
    }

    #[test]
    fn test_episode_detail_serialization() {
        let detail = EpisodeDetail {
            title: "Test Episode".to_string(),
            default_video: "https://example.com/default.mp4".to_string(),
            sources: vec![
                VideoSource {
                    server: "SOKUJA".to_string(),
                    quality: "720p".to_string(),
                    url: "https://example.com/720p.mp4".to_string(),
                },
            ],
        };
        
        let json = serde_json::to_string(&detail).unwrap();
        // Verify camelCase serialization
        assert!(json.contains("\"title\""));
        assert!(json.contains("\"defaultVideo\""));
        assert!(json.contains("\"sources\""));
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Generate a random string for HTML content
    fn arbitrary_text() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 ]{0,50}".prop_map(|s| s.trim().to_string())
    }

    /// Generate a random URL-like string
    fn arbitrary_url() -> impl Strategy<Value = String> {
        "[a-z0-9-]{1,20}".prop_map(|s| format!("/anime/{}/", s))
    }

    /// Generate a random image URL
    fn arbitrary_image_url() -> impl Strategy<Value = String> {
        "[a-z0-9]{5,15}".prop_map(|s| format!("https://example.com/{}.jpg", s))
    }

    /// Generate a random anime type
    fn arbitrary_anime_type() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("TV".to_string()),
            Just("OVA".to_string()),
            Just("Movie".to_string()),
            Just("Special".to_string()),
            Just("ONA".to_string()),
        ]
    }

    /// Generate a random status
    fn arbitrary_status() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("Completed".to_string()),
            Just("Ongoing".to_string()),
            Just("Upcoming".to_string()),
        ]
    }

    /// Generate a random rating
    fn arbitrary_rating() -> impl Strategy<Value = String> {
        (1u32..100u32).prop_map(|n| format!("{}.{}", n / 10, n % 10))
    }

    /// Generate a random episode count
    fn arbitrary_episode_count() -> impl Strategy<Value = String> {
        (1u32..100u32).prop_map(|n| format!("{} Episodes", n))
    }

    /// Generate random genres
    fn arbitrary_genres() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec(
            prop_oneof![
                Just("Action".to_string()),
                Just("Adventure".to_string()),
                Just("Comedy".to_string()),
                Just("Drama".to_string()),
                Just("Fantasy".to_string()),
                Just("Romance".to_string()),
            ],
            0..5,
        )
    }

    /// Generate HTML for a completed anime article
    fn generate_completed_anime_html(
        title: &str,
        url: &str,
        thumbnail: &str,
        anime_type: &str,
        episode_count: &str,
        status: &str,
        posted_by: &str,
        posted_at: &str,
        series_title: &str,
        series_url: &str,
        genres: &[String],
        rating: &str,
    ) -> String {
        let genre_html: String = genres
            .iter()
            .map(|g| format!(r#"<a rel="tag" href="/genre/{0}/">{1}</a>"#, g.to_lowercase(), g))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"
            <html>
            <body>
                <article class="stylesix">
                    <h2 itemprop="headline"><a href="{url}">{title}</a></h2>
                    <a itemprop="url" href="{url}"></a>
                    <img class="ts-post-image" src="{thumbnail}" />
                    <div class="typez">{anime_type}</div>
                    <span class="epx">{episode_count}</span>
                    <span class="scr">{rating}</span>
                    <ul>
                        <li>Status: {status}</li>
                        <li>Dipos Oleh: {posted_by}</li>
                        <li>Dipos pada: {posted_at}</li>
                        <li><a href="{series_url}">{series_title}</a></li>
                    </ul>
                    {genre_html}
                </article>
            </body>
            </html>
            "#,
            title = title,
            url = url,
            thumbnail = thumbnail,
            anime_type = anime_type,
            episode_count = episode_count,
            status = status,
            posted_by = posted_by,
            posted_at = posted_at,
            series_title = series_title,
            series_url = series_url,
            rating = rating,
            genre_html = genre_html,
        )
    }

    proptest! {
        /// Property 2: Completed Anime Parsing Completeness
        /// 
        /// For any valid HTML containing `article.stylesix` elements, the parser SHALL extract
        /// all completed anime entries with non-null values for: title, url, thumbnail, type,
        /// episodeCount, status, genres (array), and rating. Missing elements should result
        /// in empty strings or empty arrays.
        /// 
        /// **Validates: Requirements 3.1-3.10**
        #[test]
        fn property_completed_anime_parsing_completeness(
            title in arbitrary_text().prop_filter("non-empty title", |s| !s.is_empty()),
            url in arbitrary_url(),
            thumbnail in arbitrary_image_url(),
            anime_type in arbitrary_anime_type(),
            episode_count in arbitrary_episode_count(),
            status in arbitrary_status(),
            posted_by in arbitrary_text(),
            posted_at in "[0-9]{4}-[0-9]{2}-[0-9]{2}",
            series_title in arbitrary_text(),
            series_url in arbitrary_url(),
            genres in arbitrary_genres(),
            rating in arbitrary_rating(),
        ) {
            let html = generate_completed_anime_html(
                &title,
                &url,
                &thumbnail,
                &anime_type,
                &episode_count,
                &status,
                &posted_by,
                &posted_at,
                &series_title,
                &series_url,
                &genres,
                &rating,
            );

            let result = parse_completed_anime(&html);

            // Should parse exactly one anime
            prop_assert_eq!(result.len(), 1, "Expected exactly one completed anime");

            let anime = &result[0];

            // All fields should be non-null (not Option::None)
            // In Rust, String is never null, so we verify they match expected values
            
            // Required fields should match input
            prop_assert_eq!(&anime.title, &title, "Title mismatch");
            prop_assert_eq!(&anime.url, &url, "URL mismatch");
            prop_assert_eq!(&anime.thumbnail, &thumbnail, "Thumbnail mismatch");
            prop_assert_eq!(&anime.anime_type, &anime_type, "Type mismatch");
            prop_assert_eq!(&anime.episode_count, &episode_count, "Episode count mismatch");
            prop_assert_eq!(&anime.status, &status, "Status mismatch");
            prop_assert_eq!(&anime.rating, &rating, "Rating mismatch");

            // Genres should match (order may vary due to HTML parsing)
            prop_assert_eq!(anime.genres.len(), genres.len(), "Genres count mismatch");
            for genre in &genres {
                prop_assert!(
                    anime.genres.contains(genre),
                    "Missing genre: {}",
                    genre
                );
            }

            // Optional fields should be present (may be empty strings)
            prop_assert!(!anime.posted_by.is_empty() || posted_by.is_empty(), "Posted by should match");
            prop_assert!(!anime.posted_at.is_empty() || posted_at.is_empty(), "Posted at should match");
        }

        /// Property: Empty HTML returns empty array
        /// 
        /// For any HTML without article.stylesix elements, the parser should return an empty array.
        #[test]
        fn property_empty_html_returns_empty_array(
            random_content in "[a-zA-Z0-9 ]{0,100}",
        ) {
            let html = format!("<html><body><div>{}</div></body></html>", random_content);
            let result = parse_completed_anime(&html);
            prop_assert!(result.is_empty(), "Expected empty array for HTML without article.stylesix");
        }

        /// Property: Missing elements default to empty values
        /// 
        /// For any HTML with article.stylesix but missing child elements,
        /// the parser should use empty strings/arrays as defaults.
        #[test]
        fn property_missing_elements_default_to_empty(
            title in arbitrary_text().prop_filter("non-empty title", |s| !s.is_empty()),
        ) {
            // HTML with only title, missing all other elements
            let html = format!(
                r#"
                <html>
                <body>
                    <article class="stylesix">
                        <h2 itemprop="headline"><a href="/anime/test/">{}</a></h2>
                    </article>
                </body>
                </html>
                "#,
                title
            );

            let result = parse_completed_anime(&html);
            prop_assert_eq!(result.len(), 1, "Expected exactly one completed anime");

            let anime = &result[0];
            
            // Title should be parsed
            prop_assert_eq!(&anime.title, &title, "Title should be parsed");
            
            // Missing elements should default to empty strings/arrays
            prop_assert!(anime.thumbnail.is_empty(), "Missing thumbnail should be empty string");
            prop_assert!(anime.anime_type.is_empty(), "Missing type should be empty string");
            prop_assert!(anime.episode_count.is_empty(), "Missing episode count should be empty string");
            prop_assert!(anime.status.is_empty(), "Missing status should be empty string");
            prop_assert!(anime.rating.is_empty(), "Missing rating should be empty string");
            prop_assert!(anime.genres.is_empty(), "Missing genres should be empty array");
        }
    }
}
