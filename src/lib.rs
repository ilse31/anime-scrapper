//! Anime Scraper API Library
//!
//! This library provides functionality for scraping anime data from sokuja.uk
//! and exposing it through REST API endpoints.

pub mod auth;
pub mod config;
pub mod constants;
pub mod db;
pub mod email;
pub mod error;
pub mod models;
pub mod parser;
pub mod routes;
pub mod scraper;
