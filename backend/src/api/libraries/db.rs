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
    let images_table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='images')",
    )
    .fetch_one(&pool)
    .await
    .context("Failed to check if images table exists")?;

    if !images_table_exists {
        // Initialize the database with the images table
        sqlx::query(
            r#"
            CREATE TABLE images (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                is_public BOOLEAN NOT NULL DEFAULT FALSE,
                karma INTEGER DEFAULT 0,
                content_hash TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX idx_is_public ON images(is_public);
            CREATE INDEX idx_karma ON images(karma);
            CREATE INDEX idx_content_hash ON images(content_hash);
            "#,
        )
        .execute(&pool)
        .await
        .context("Failed to create images table")?;
    }

    // Migration: ensure the `content_hash` column exists on pre-existing databases. Newly
    // created tables already have it (the check below returns true and this is skipped). The
    // hash backs de-duplication now that filenames are timestamp-based instead of hash-based.
    let content_hash_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM pragma_table_info('images') WHERE name = 'content_hash')",
    )
    .fetch_one(&pool)
    .await
    .context("Failed to check if images.content_hash column exists")?;

    if !content_hash_exists {
        sqlx::query(
            r#"
            ALTER TABLE images ADD COLUMN content_hash TEXT;
            CREATE INDEX IF NOT EXISTS idx_content_hash ON images(content_hash);
            "#,
        )
        .execute(&pool)
        .await
        .context("Failed to add content_hash column to images table")?;
    }

    // Check if the database is new by checking if the `params` table exists
    let params_table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='params')",
    )
    .fetch_one(&pool)
    .await
    .context("Failed to check if params table exists")?;

    if !params_table_exists {
        // Initialize the database with the images table
        sqlx::query(
            r#"
            CREATE TABLE params (
                key         TEXT PRIMARY KEY,
                value       TEXT NOT NULL,
                description TEXT,
                updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TRIGGER update_params_timestamp
            AFTER UPDATE ON params
            FOR EACH ROW
            BEGIN
                UPDATE params
                SET updated_at = CURRENT_TIMESTAMP
                WHERE key = OLD.key;
            END;
            INSERT INTO params (key, value, description) VALUES ('upload_allowed', '0', 'whether image upload is allowed');
            "#,
        )
        .execute(&pool)
        .await
        .context("Failed to create params table")?;
    }

    Ok(pool)
}

/// Checks if the back-end allows image upload. If param is not found or is malformed, deny upload.
pub async fn get_upload_allowed(pool: &Pool<Sqlite>) -> Result<bool> {
    let result: Option<(String,)> =
        sqlx::query_as("SELECT value FROM params WHERE key = 'upload_allowed'")
            .fetch_optional(pool)
            .await
            .context("Failed to fetch upload_allowed parameter")?;

    match result {
        Some((value,)) => match value.as_str() {
            "1" => Ok(true),
            _ => Ok(false),
        },
        None => Ok(false),
    }
}
