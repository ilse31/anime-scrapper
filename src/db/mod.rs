//! Database module for the Anime Scraper API
//!
//! Provides database connection pool management, health check functionality,
//! and repository functions for anime data persistence.

pub mod repository;

pub use repository::*;

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Error as SqlxError;
use std::time::Duration;
use thiserror::Error;

/// Database-related errors
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Failed to connect to database: {0}")]
    ConnectionError(#[from] SqlxError),

    #[error("Database health check failed: {0}")]
    HealthCheckError(String),
}

/// Database connection pool wrapper
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    ///
    /// # Arguments
    /// * `database_url` - PostgreSQL connection string
    ///
    /// # Returns
    /// A new Database instance with an active connection pool
    pub async fn new(database_url: &str) -> Result<Self, DbError> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Get a reference to the underlying connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run database migrations
    ///
    /// # Returns
    /// Ok(()) if migrations succeed, error otherwise
    pub async fn run_migrations(&self) -> Result<(), DbError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| DbError::ConnectionError(SqlxError::Migrate(Box::new(e))))?;
        Ok(())
    }

    /// Check database health by executing a simple query
    ///
    /// # Returns
    /// Ok(()) if the database is healthy, error otherwise
    pub async fn health_check(&self) -> Result<(), DbError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::HealthCheckError(e.to_string()))?;
        Ok(())
    }

    /// Close the database connection pool gracefully
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires a running database
    async fn test_database_connection() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for tests");

        let db = Database::new(&database_url).await;
        assert!(db.is_ok(), "Should connect to database");

        let db = db.unwrap();
        let health = db.health_check().await;
        assert!(health.is_ok(), "Health check should pass");
    }
}
