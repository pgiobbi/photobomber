mod api;
mod types;

use crate::api::constants::DEFAULT_MAX_FILE_SIZE;
use crate::api::extractors::auth_middleware::AuthMiddlewareFactory;
use crate::api::routes::auth::{get_profile, login, logout};
use crate::api::routes::images::{get_image, get_upload_count, upload_images};
use crate::api::routes::location::{get_location, post_location};
use crate::api::routes::stages::get_stages;
use crate::types::stages::TomorrowlandStage;
use crate::types::state::{AppState, CredentialState, LocationState};
use actix_cors::Cors;
use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::middleware::NormalizePath;
use actix_web::{App, HttpServer, http, web};
use env_logger::Env;
use jwt_compact::alg::{Ed25519, Hs256Key};
use jwt_compact::jwk::KeyType::KeyPair;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::env;
use std::env::join_paths;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::ops::Index;
use std::path::Path;
use std::string::ToString;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting application");

    // Create upload directory
    let upload_dir = env::var("UPLOAD_DIR").unwrap_or("./uploads".to_string());
    match std::fs::create_dir_all(upload_dir.clone()) {
        Ok(_) => {}
        Err(e) => {
            error!("{}", format!("Failed to create upload directory: {}", e));
            return Err(e);
        }
    }

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

    let app_state = web::Data::new(AppState {
        upload_dir: upload_dir.clone(),
        max_file_size,
        location_state: RwLock::new(LocationState::default()),
        credential_state: CredentialState::from_env(),
        stages,
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
                web::scope("/api/auth")
                    .service(web::resource("login").route(web::post().to(login)))
                    .service(web::resource("logout").route(web::post().to(logout)))
                    .service(
                        web::resource("profile")
                            .wrap(AuthMiddlewareFactory)
                            .route(web::get().to(get_profile)),
                    ),
            )
            .service(
                web::scope("/api/images")
                    .wrap(AuthMiddlewareFactory)
                    .service(get_upload_count)
                    .service(upload_images)
                    .service(get_image),
            )
            .service(
                web::scope("/api/location")
                    .wrap(AuthMiddlewareFactory)
                    .service(get_location)
                    .service(post_location),
            )
            .service(
                web::scope("/api/stages")
                    .wrap(AuthMiddlewareFactory)
                    .service(get_stages),
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
    .await
}
