use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use std::path::PathBuf;

/// Connects to an existing SQLite database at the given path or creates and initializes it
/// if it doesn't exist.
pub async fn connect_or_initialize_db(db_path: PathBuf) -> Result<Pool<Sqlite>> {
    let options = SqliteConnectOptions::new()
        .create_if_missing(true)
        .filename(db_path);

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .context("Failed to connect to SQLite database")?;

    // Check if the database is new by checking if the `images` table exists
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='images')",
    )
    .fetch_one(&pool)
    .await
    .context("Failed to check if images table exists")?;

    if !table_exists {
        // Initialize the database with the images table
        sqlx::query(
            r#"
            CREATE TABLE images (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                join_leaderboard BOOLEAN NOT NULL DEFAULT FALSE,
                karma INTEGER DEFAULT 0,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX idx_join_leaderboard ON images(join_leaderboard);
            CREATE INDEX idx_karma ON images(karma);
            "#,
        )
        .execute(&pool)
        .await
        .context("Failed to create images table")?;
    }

    Ok(pool)
}
