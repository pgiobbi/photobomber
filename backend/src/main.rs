use actix_cors::Cors;
use actix_multipart::form::{
    MultipartForm,
    tempfile::{TempFile, TempFileConfig},
};
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::middleware::NormalizePath;
use actix_web::{App, Error, HttpResponse, HttpServer, Responder, get, http, post, web};
use env_logger::Env;
use log::{error, info, warn};
use mime::{IMAGE, Mime};
use sanitize_filename::sanitize;
use serde::Serialize;
use std::env;
use std::path::Path;
use std::string::ToString;
use tokio::fs;
use uuid::Uuid;

const DEFAULT_MAX_FILE_SIZE: u64 = 1024 * 1024; // 1MB in bytes

// Whitelist of allowed image extensions
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];

// Structure for the multipart form
#[derive(Debug, MultipartForm)]
struct ImageUploadForm {
    #[multipart(rename = "images", limit = "1MB")]
    images: Vec<TempFile>,
}

// Structure for the response
#[derive(Serialize)]
struct UploadResponse {
    file_id: String,
    file_name: String,
}

// Structure for the count response
#[derive(Serialize)]
struct CountResponse {
    count: u64,
}

type Bytes = u64;

#[derive(Clone)]
struct AppState {
    /// Path to the upload directory.
    upload_dir: String,
    /// Max upload file size (bytes).
    max_file_size: Bytes,
}

// Map MIME type to file extension
fn get_extension_from_mime(mime: &Mime) -> &str {
    match mime.subtype().as_str() {
        "jpeg" => "jpg",
        "png" => "png",
        "gif" => "gif",
        "bmp" => "bmp",
        "webp" => "webp",
        _ => "jpg", // Default to jpg for unknown image types
    }
}

// Extract extension from filename
fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(|ext| ext.to_str())
}

// Image upload endpoint
#[post("/upload")]
async fn upload_images(
    MultipartForm(form): MultipartForm<ImageUploadForm>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    info!("Starting image upload processing");

    let mut responses = Vec::new();

    for file in form.images {
        // Validate content type
        let content_type = file.content_type.unwrap_or(mime::APPLICATION_OCTET_STREAM);
        if !content_type.type_().eq(&IMAGE) {
            warn!("{}", format!("Invalid content type: {}", content_type));
            return Ok(HttpResponse::BadRequest().json("Only image files are allowed"));
        }

        // Validate file size
        if file.size > app_state.max_file_size as usize {
            warn!("{}", format!("File size {} exceeds 1MB limit", file.size));
            return Ok(HttpResponse::BadRequest().json("File size exceeds 1MB limit"));
        }

        // Get or generate filename
        let original_file_name = file
            .file_name
            .unwrap_or_else(|| format!("image_{}", Uuid::new_v4()));

        // Determine file extension
        let extension = get_extension_from_filename(&original_file_name)
            .unwrap_or_else(|| get_extension_from_mime(&content_type));

        // Generate unique file ID
        let file_id = Uuid::new_v4().to_string();

        // Create persistent file path with extension
        let file_name_with_ext = format!("{}.{}", file_id, extension);
        let file_path = format!("{}/{}", app_state.upload_dir, file_name_with_ext);

        // Persist the file
        match file.file.persist(&file_path) {
            Ok(_) => {
                info!(
                    "{}",
                    format!("Successfully saved file: {}", file_name_with_ext)
                );
            }
            Err(e) => {
                error!(
                    "{}",
                    format!("Failed to save file {}: {}", file_name_with_ext, e)
                );
                return Err(actix_web::error::ErrorInternalServerError(format!(
                    "Failed to save file: {}",
                    e
                )));
            }
        }

        responses.push(UploadResponse {
            file_id,
            file_name: file_name_with_ext,
        });
    }

    if responses.is_empty() {
        warn!("No valid images uploaded");
        return Ok(HttpResponse::BadRequest().json("No valid images uploaded"));
    }

    info!(
        "{}",
        format!("Successfully uploaded {} images", responses.len())
    );
    Ok(HttpResponse::Ok().json(responses))
}

// Simplified image retrieval endpoint
#[get("/images/{filename}")]
async fn get_image(
    path: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    let filename = path.into_inner();
    info!("{}", format!("Attempting to retrieve image: {}", filename));

    // Sanitize filename to prevent path traversal
    let sanitized_filename = sanitize(&filename);
    if sanitized_filename.contains("..")
        || sanitized_filename.contains('/')
        || sanitized_filename.contains('\\')
    {
        warn!(
            "{}",
            format!("Invalid filename detected: {}", sanitized_filename)
        );
        return Ok(HttpResponse::BadRequest().json("Invalid filename"));
    }

    // Check if extension is allowed
    let ext = Path::new(&sanitized_filename)
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| ALLOWED_EXTENSIONS.contains(e))
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid file extension"))?;

    // Construct and validate file path
    let file_path = format!("{}/{}", app_state.upload_dir, sanitized_filename);
    if !file_path.starts_with(&app_state.upload_dir) {
        warn!("{}", format!("Invalid file path: {}", file_path));
        return Ok(HttpResponse::BadRequest().json("Invalid file path"));
    }

    // Check if file exists
    if fs::metadata(&file_path).await.is_err() {
        warn!("{}", format!("Image not found: {}", file_path));
        return Ok(HttpResponse::BadRequest().json("Image not found"));
    }

    // Read file
    let content = match fs::read(&file_path).await {
        Ok(content) => content,
        Err(e) => {
            error!("{}", format!("Failed to read file {}: {}", file_path, e));
            return Err(actix_web::error::ErrorInternalServerError(e));
        }
    };

    // Determine MIME type
    let mime = match ext {
        "jpg" | "jpeg" => mime::IMAGE_JPEG,
        "png" => mime::IMAGE_PNG,
        "gif" => mime::IMAGE_GIF,
        _ => mime::APPLICATION_OCTET_STREAM,
    };

    // Set content disposition for safe rendering
    let disposition = ContentDisposition {
        disposition: DispositionType::Inline,
        parameters: vec![DispositionParam::Filename(sanitized_filename)],
    };

    info!("{}", format!("Successfully retrieved image: {}", filename));
    Ok(HttpResponse::Ok()
        .content_type(mime)
        .append_header(disposition)
        .body(content))
}

// Count endpoint
#[get("/count")]
async fn get_upload_count(app_state: web::Data<AppState>) -> Result<impl Responder, Error> {
    info!("Fetching upload count");

    let mut count = 0;
    let mut entries = match fs::read_dir(&app_state.upload_dir).await {
        Ok(entries) => entries,
        Err(e) => {
            error!("{}", format!("Failed to read upload directory: {}", e));
            return Err(actix_web::error::ErrorInternalServerError(e));
        }
    };

    while let Some(entry) = entries.next_entry().await.map_err(|e| {
        error!("{}", format!("Error reading directory entry: {}", e));
        actix_web::error::ErrorInternalServerError(e)
    })? {
        if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
            if ALLOWED_EXTENSIONS.contains(&ext) {
                count += 1;
            }
        }
    }

    info!("{}", format!("Retrieved upload count: {}", count));
    Ok(HttpResponse::Ok().json(CountResponse { count }))
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
                web::scope("/api")
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
