mod api;
mod types;

use crate::api::constants::DEFAULT_MAX_FILE_SIZE;
use crate::api::extractors::auth_middleware::AuthMiddlewareFactory;
use crate::api::libraries::db::connect_or_initialize_db;
use crate::api::routes::auth::{get_profile, post_login, post_logout};
use crate::api::routes::images::{get_image, get_leaderboard, get_upload_count, post_image_upvote, upload_images};
use crate::api::routes::location::{get_location, post_location};
use crate::api::routes::stages::get_stages;
use crate::types::stages::TomorrowlandStage;
use crate::types::state::{AppState, CredentialState, LocationState};
use actix_cors::Cors;
use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::middleware::NormalizePath;
use actix_web::{App, HttpServer, http, web};
use anyhow::{Context, Error, Result};
use env_logger::Env;
use jwt_compact::alg::{Ed25519, Hs256Key};
use jwt_compact::jwk::KeyType::KeyPair;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use std::env;
use std::env::join_paths;
use std::fs::File;
use std::io::BufReader;
use std::ops::Index;
use std::path::Path;
use std::string::ToString;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting application");

    // Create upload directory
    let upload_dir = env::var("UPLOAD_DIR").unwrap_or("./uploads".to_string());
    std::fs::create_dir_all(upload_dir.clone()).context("UPLOAD_DIR could not be created")?;

    let max_file_size = env::var("MAX_FILE_SIZE")
        .unwrap_or(DEFAULT_MAX_FILE_SIZE.to_string())
        .parse::<u64>()
        .unwrap_or(DEFAULT_MAX_FILE_SIZE);

    info!("UPLOAD_DIR    {:>16}: {}", "", &upload_dir);
    info!("MAX_FILE_SIZE {:>16}: {}", "", &max_file_size);

    // Load and parse tomorrowland stages
    let config_dir = env::var("CONFIG_DIR").expect("CONFIG_DIR not set");
    let stages_path = Path::new(&config_dir).join("stages.json");
    let stages = {
        let file = File::open(stages_path)?;
        let reader = BufReader::new(file);

        serde_json::from_value(
            serde_json::from_reader::<_, serde_json::Value>(reader)?
                .get("stages")
                .unwrap()
                .clone(),
        )?
    };

    // Initialize db connection
    let db_path = Path::new(&config_dir).join("photobomb.sqlite");
    let pool = connect_or_initialize_db(db_path)
        .await
        .expect("Failed to initialize database");

    let app_state = web::Data::new(AppState {
        upload_dir: upload_dir.clone(),
        max_file_size,
        location_state: RwLock::new(LocationState::default()),
        credential_state: CredentialState::from_env(),
        stages,
        db_pool: pool,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .app_data(TempFileConfig::default().directory(upload_dir.to_string()))
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|origin, _req_head| {
                        // Extract the origin string
                        let origin_str = origin.to_str().unwrap();

                        let cors_allow_all = env::var("PHOTOBOMBER_CORS_ALLOW_ALL")
                            .unwrap_or("false".to_string())
                            .eq("true");

                        // Allow localhost variations and specific domain
                        let allowed = origin_str.starts_with("http://localhost")
                            || origin_str.starts_with("http://127.0.0.1")
                            || origin_str.starts_with("http://[::1]")
                            || origin_str == "https://photobomber.servebeer.com"
                            || cors_allow_all;

                        debug!(
                            "{}",
                            format!(
                                "CORS check for origin {}: {}",
                                origin_str,
                                if allowed { "allowed" } else { "denied" }
                            )
                        );
                        allowed
                    })
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .wrap(NormalizePath::trim())
            .service(
                // Public (w/o auth) endpoints. For both admin and public users
                web::scope("/api/auth")
                    .service(web::scope("login").service(post_login))
                    .service(web::scope("logout").service(post_logout)),
            )
            .service(
                // Public (w/ auth) endpoints. Scoping is required to avoid cookie conflicts, since
                // public cookies are generated with `Path=/api/public`
                web::scope("/api/public")
                    .wrap(AuthMiddlewareFactory)
                    .service(
                        web::scope("images")
                            .service(get_upload_count)
                            .service(upload_images)
                            .service(get_leaderboard)
                            .service(post_image_upvote)
                            .service(get_image), // parametric endpoint, must be last
                    )
                    .service(web::scope("location").service(get_location)),
            )
            .service(
                // Admin (w/ auth) endpoints. Scoping is required to avoid cookie conflicts, since
                // admin cookies are generated with `Path=/api/admin`
                web::scope("/api/admin")
                    .wrap(AuthMiddlewareFactory)
                    .service(web::scope("stages").service(get_stages))
                    .service(
                        web::scope("location")
                            .service(post_location)
                            .service(get_location),
                    )
                    .service(web::scope("profile").service(get_profile)),
            )
    })
    .bind(("0.0.0.0", 3002))
    .map(|res| {
        info!("Server started on port 3002");
        res
    })
    .map_err(|e| {
        error!("{}", format!("Failed to bind server: {}", e));
        e
    })?
    .run()
    .await?;

    Ok(())
}
