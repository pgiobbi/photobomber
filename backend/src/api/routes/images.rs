use crate::AppState;
use crate::api::constants::ALLOWED_EXTENSIONS;
use crate::api::utils::{get_extension_from_filename, get_extension_from_mime};
use crate::types::images::{ImageCountResponse, ImageUploadRequest, ImageUploadResponse};
use actix_multipart::form::MultipartForm;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::{Error, HttpResponse, Responder, get, post, web};
use log::{error, info, warn};
use mime::IMAGE;
use sanitize_filename::sanitize;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;

// Count endpoint
#[get("/count")]
pub async fn get_upload_count(app_state: web::Data<AppState>) -> Result<impl Responder, Error> {
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
    Ok(HttpResponse::Ok().json(ImageCountResponse { count }))
}

// Image upload endpoint
#[post("/upload")]
pub async fn upload_images(
    MultipartForm(form): MultipartForm<ImageUploadRequest>,
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

        responses.push(ImageUploadResponse {
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
#[get("/{filename}")]
pub async fn get_image(
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
