use actix_cors::Cors;
use actix_web::middleware::NormalizePath;
use actix_web::{App, Error, HttpResponse, HttpServer, Responder, get, http, post, web};
use serde::Serialize;
use std::path::Path;
use uuid::Uuid;

use actix_multipart::form::{
    MultipartForm,
    tempfile::{TempFile, TempFileConfig},
};
use mime::{IMAGE, Mime};
use tokio::fs;

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB in bytes
const UPLOAD_DIR: &str = "./uploads";

// Whitelist of allowed image extensions
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];

// Structure for the multipart form
#[derive(Debug, MultipartForm)]
struct ImageUploadForm {
    #[multipart(rename = "images", limit = "10MB")]
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
) -> Result<impl Responder, Error> {
    let mut responses = Vec::new();

    for file in form.images {
        // Validate content type
        let content_type = file.content_type.unwrap_or(mime::APPLICATION_OCTET_STREAM);
        if !content_type.type_().eq(&IMAGE) {
            return Ok(HttpResponse::BadRequest().json("Only image files are allowed"));
        }

        // Validate file size
        if file.size > MAX_FILE_SIZE as usize {
            return Ok(HttpResponse::BadRequest().json("File size exceeds 10MB limit"));
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
        let file_path = format!("{}/{}", UPLOAD_DIR, file_name_with_ext);

        // Persist the file
        file.file.persist(&file_path).map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to save file: {}", e))
        })?;

        responses.push(UploadResponse {
            file_id,
            file_name: file_name_with_ext,
        });
    }

    if responses.is_empty() {
        return Ok(HttpResponse::BadRequest().json("No valid images uploaded"));
    }

    Ok(HttpResponse::Ok().json(responses))
}

// Count endpoint
#[get("/count")]
async fn get_upload_count() -> Result<impl Responder, Error> {
    let mut count = 0;
    let mut entries = fs::read_dir(UPLOAD_DIR)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    {
        if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
            if ALLOWED_EXTENSIONS.contains(&ext) {
                count += 1;
            }
        }
    }

    Ok(HttpResponse::Ok().json(CountResponse { count }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create upload directory
    std::fs::create_dir_all(UPLOAD_DIR)?;

    HttpServer::new(|| {
        App::new()
            .app_data(TempFileConfig::default().directory(UPLOAD_DIR))
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|origin, _req_head| {
                        // Extract the origin string
                        let origin_str = origin.to_str().unwrap();

                        // Allow localhost variations (http://localhost, http://127.0.0.1, http://[::1], any port)
                        origin_str.starts_with("http://localhost") ||
                        origin_str.starts_with("http://127.0.0.1") ||
                        origin_str.starts_with("http://[::1]") ||
                        // Allow https://photobomber.servebeer.com
                        origin_str == "https://photobomber.servebeer.com"
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
                    .service(upload_images),
            )
    })
    .bind(("0.0.0.0", 3002))?
    .run()
    .await
}
