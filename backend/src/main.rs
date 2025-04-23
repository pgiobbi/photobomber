mod api;
mod types;

use crate::api::constants::DEFAULT_MAX_FILE_SIZE;
use crate::api::extractors::auth_middleware::AuthMiddlewareFactory;
use crate::api::routes::auth::login;
use crate::api::routes::images::{get_image, get_upload_count, upload_images};
use actix_cors::Cors;
use actix_jwt_auth_middleware::{Authority, FromRequest, TokenSigner};
use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::middleware::NormalizePath;
use actix_web::{App, HttpServer, http, web};
use env_logger::Env;
use jwt_compact::alg::{Ed25519, Hs256Key};
use jwt_compact::jwk::KeyType::KeyPair;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::env;
use std::string::ToString;

type Bytes = u64;

#[derive(Clone)]
struct AppState {
    /// Path to the upload directory.
    upload_dir: String,
    /// Max upload file size (bytes).
    max_file_size: Bytes,
}

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

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                upload_dir: upload_dir.clone(),
                max_file_size,
            }))
            .app_data(TempFileConfig::default().directory(upload_dir.to_string()))
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|origin, _req_head| {
                        // Extract the origin string
                        let origin_str = origin.to_str().unwrap();

                        // Allow localhost variations and specific domain
                        let allowed = origin_str.starts_with("http://localhost")
                            || origin_str.starts_with("http://127.0.0.1")
                            || origin_str.starts_with("http://[::1]")
                            || origin_str == "https://photobomber.servebeer.com";

                        info!(
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
                    .service(web::resource("login").route(web::post().to(login))),
            )
            .service(
                web::scope("/api/images")
                    // .wrap(AuthMiddlewareFactory) // TODO: enable
                    .service(get_upload_count)
                    .service(upload_images)
                    .service(get_image),
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
