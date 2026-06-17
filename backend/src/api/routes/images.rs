use crate::AppState;
use crate::api::constants::{ALLOWED_EXTENSIONS, UPVOTES_PER_UPLOAD};
use crate::api::libraries::db::get_upload_allowed;
use crate::api::utils::{get_extension_from_filename, get_extension_from_mime};
use crate::types::db::DbImage;
use crate::types::images::{
    GetLeaderboardQueryParams, ImageCountResponse, ImageUploadRequest, ImageUploadResponse,
    LeaderboardOrder,
};
use actix_multipart::form::MultipartForm;
use actix_multipart::form::text::Text;
use actix_web::cookie::Cookie;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::{Error, HttpRequest, HttpResponse, Responder, get, post, web};
use log::{debug, error, info, warn};
use mime::IMAGE;
use sanitize_filename::sanitize;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::sqlite::SqliteQueryResult;
use std::fs::{File, Permissions};
use std::hash::Hasher;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tokio::fs;
use twox_hash::XxHash64;
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

#[get("leaderboard")]
pub async fn get_leaderboard(
    query: web::Query<GetLeaderboardQueryParams>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    info!("{:?}", query);

    let order_by = match query.order_by {
        None | Some(LeaderboardOrder::Time) => "created_at",
        Some(LeaderboardOrder::Karma) => "karma",
    };
    let order_direction = match query.ascending {
        None | Some(false) => "DESC",
        Some(true) => "ASC",
    };

    // Execute the query and fetch rows
    let entries: Vec<DbImage> = sqlx::query_as(
        format!(
            "SELECT * FROM images \
             WHERE is_public is TRUE
             ORDER BY {} {}
             LIMIT 100",
            order_by, order_direction
        )
        .as_str(),
    )
    .fetch_all(&app_state.db_pool)
    .await
    .map_err(|e| {
        error!("Database error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to fetch images")
    })?;

    Ok(HttpResponse::Ok().json(entries))
}

// Image upload endpoint
#[post("/upload")]
pub async fn upload_images(
    MultipartForm(form): MultipartForm<ImageUploadRequest>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    info!("Starting image upload processing");

    // Check if image upload is allowed
    if !get_upload_allowed(&app_state.db_pool).await.unwrap_or(false) {
        warn!("Image upload not allowed");
        return Ok(HttpResponse::Forbidden().json("Image upload is not allowed"));
    }

    let mut responses = Vec::new();

    let mut num_non_duplicates = 0;
    for mut file in form.images {
        // Validate content type
        let content_type = file.content_type.unwrap_or(mime::APPLICATION_OCTET_STREAM);
        if !content_type.type_().eq(&IMAGE) {
            warn!("Invalid content type: {}", content_type);
            return Ok(HttpResponse::BadRequest().json("Only image files are allowed"));
        }

        // Validate file size
        if file.size > app_state.max_file_size as usize {
            warn!(
                "File size {} exceeds limit of {} bytes",
                file.size, app_state.max_file_size
            );
            return Ok(HttpResponse::BadRequest().json("File size exceeds the allowed limit"));
        }

        // Get or generate original filename

        // Determine file extension
        let extension = get_extension_from_mime(&content_type);
        // let original_file_name = file.file_name.unwrap_or_else(|| "image".to_string());
        // let extension = get_extension_from_filename(&original_file_name)
        //     .unwrap_or_else(|| get_extension_from_mime(&content_type));

        // Read file content for hashing
        let mut content = Vec::new();
        file.file.read_to_end(&mut content).map_err(|e| {
            error!("Failed to read file content: {}", e);
            actix_web::error::ErrorInternalServerError(format!("Failed to read file: {}", e))
        })?;

        // Generate hash-based filename
        let mut hasher = XxHash64::default();
        hasher.write(&content);
        let file_id = format!("{:x}", hasher.finish());
        let file_name_with_ext = format!("{}.{}", file_id, extension);
        let file_path = format!("{}/{}", app_state.upload_dir, file_name_with_ext);

        // Check if file already exists
        if Path::new(&file_path).exists() {
            warn!("File already exists: {}", file_name_with_ext);
            responses.push(ImageUploadResponse {
                file_id: file_id.clone(),
                file_name: file_name_with_ext,
                is_public: form.is_public.0,
                _is_new: false,
            });
            num_non_duplicates += 1;
            continue;
        }

        // Persist the file
        match file.file.persist(&file_path) {
            Ok(_) => {
                info!("Successfully saved file: {}", file_name_with_ext);

                // Set file permissions to rw-r--r-- (644)
                let file = File::open(&file_path).map_err(|e| {
                    error!(
                        "Failed to open file {} to set permissions: {}",
                        file_name_with_ext, e
                    );
                    actix_web::error::ErrorInternalServerError(format!(
                        "Failed to set permissions: {}",
                        e
                    ))
                })?;
                file.set_permissions(Permissions::from_mode(0o644))
                    .map_err(|e| {
                        error!(
                            "Failed to set permissions for {}: {}",
                            file_name_with_ext, e
                        );
                        actix_web::error::ErrorInternalServerError(format!(
                            "Failed to set permissions: {}",
                            e
                        ))
                    })?;
            }
            Err(e) => {
                error!("Failed to save file {}: {}", file_name_with_ext, e);
                return Err(actix_web::error::ErrorInternalServerError(format!(
                    "Failed to save file: {}",
                    e
                )));
            }
        }

        // Insert metadata into the database
        let insert_res = sqlx::query("INSERT INTO images (filename, is_public) VALUES ($1, $2)")
            .bind(&file_name_with_ext)
            .bind(form.is_public.0)
            .execute(&app_state.db_pool)
            .await;

        // Delete image if database insert was not successful
        if let Err(e) = insert_res {
            error!("Failed to persist file on DB {}: {}", file_name_with_ext, e);
            if let Err(e) = fs::remove_file(file_path).await {
                error!(
                    "Failed to remove orphaned file {}: {}",
                    file_name_with_ext, e
                );
            }
            return Ok(HttpResponse::InternalServerError().json("Database error"));
        };

        responses.push(ImageUploadResponse {
            file_id,
            file_name: file_name_with_ext,
            is_public: form.is_public.0,
            _is_new: true,
        });
    }

    if responses.is_empty() {
        warn!("No valid images uploaded");
        return Ok(HttpResponse::BadRequest().json("No valid images uploaded"));
    }

    if num_non_duplicates != 0 {
        debug!(
            "Found {} duplicates. Upvote token will not be issued",
            num_non_duplicates
        );
        return Ok(HttpResponse::Ok().json(responses));
    }

    // Generate cookie and/or update cache
    let upvote_token = Uuid::new_v4();
    app_state
        .cache
        .insert(format!("upvote.{upvote_token}"), UPVOTES_PER_UPLOAD)
        .await;

    info!("Successfully uploaded {} images", responses.len());
    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("upvote_token", upvote_token.to_string())
                .path("/api/public")
                .secure(true)
                .http_only(true)
                .finish(),
        )
        .json(responses))
}

// Simplified image retrieval endpoint
#[get("/{filename}")]
pub async fn get_image(
    path: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    let filename = path.into_inner();
    debug!("Attempting to retrieve image: {}", filename);

    // Sanitize filename to prevent path traversal
    let sanitized_filename = sanitize(&filename);
    if sanitized_filename.contains("..")
        || sanitized_filename.contains('/')
        || sanitized_filename.contains('\\')
    {
        warn!("Invalid filename detected: {}", sanitized_filename);
        return Ok(HttpResponse::BadRequest().json("Invalid filename"));
    }

    // TODO: query the DB, only return if found and is_public is TRUE (public images)

    // Check if extension is allowed
    let ext = Path::new(&sanitized_filename)
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| ALLOWED_EXTENSIONS.contains(e))
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid file extension"))?;

    // Construct and validate file path
    let file_path = format!("{}/{}", app_state.upload_dir, sanitized_filename);
    if !file_path.starts_with(&app_state.upload_dir) {
        warn!("Invalid file path: {}", file_path);
        return Ok(HttpResponse::BadRequest().json("Invalid file path"));
    }

    // Check if file exists
    if fs::metadata(&file_path).await.is_err() {
        warn!("Image not found: {}", file_path);
        return Ok(HttpResponse::BadRequest().json("Image not found"));
    }

    // Read file
    let content = match fs::read(&file_path).await {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read file {}: {}", file_path, e);
            return Err(actix_web::error::ErrorInternalServerError(e));
        }
    };

    // TODO: return not found if image was created more than 10 mins ago

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

    Ok(HttpResponse::Ok()
        .content_type(mime)
        .append_header(disposition)
        .body(content))
}

#[post("/{id}/upvote")]
pub async fn post_image_upvote(
    req: HttpRequest,
    path: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    let id = path.into_inner();

    // Parse the upvote cookie and check the remaining number of upvotes for this token from cache
    let Some(mut upvote_cookie) = req.cookie("upvote_token") else {
        return Ok(HttpResponse::Forbidden().json(json!("No upvote token")));
    };
    let upvote_token_key = format!("upvote.{}", upvote_cookie.value().to_string());

    let remaining_upvotes = app_state.cache.get(&upvote_token_key).await;
    let mut remaining_upvotes = match remaining_upvotes {
        None | Some(0) => {
            // No upvotes left
            app_state.cache.invalidate(&upvote_token_key).await;
            upvote_cookie.make_removal();
            return Ok(HttpResponse::Forbidden().json(json!("No upvote token")));
        }
        Some(x) => x,
    };

    debug!("Attempting to upvote image: {}", id);
    let rows_affected =
        sqlx::query("UPDATE images SET karma = karma + 1 WHERE id = $1 AND is_public IS TRUE;")
            .bind(&id)
            .execute(&app_state.db_pool)
            .await
            .map_or_else(|e| 0, |r| r.rows_affected());

    if rows_affected == 0 {
        return Ok(HttpResponse::Forbidden().json(json!({ "rowsAffected": rows_affected })));
    };

    // Decrease the number of remaining upvotes
    remaining_upvotes -= 1;

    // Remove the cookie if there are no more upvotes left
    if remaining_upvotes == 0 {
        app_state.cache.invalidate(&upvote_token_key).await;
        upvote_cookie.make_removal();

        Ok(HttpResponse::Ok()
            .cookie(upvote_cookie)
            .json(json!({ "rowsAffected": rows_affected })))
    } else {
        app_state
            .cache
            .insert(upvote_token_key.clone(), remaining_upvotes)
            .await;
        Ok(HttpResponse::Ok().json(json!({ "rowsAffected": rows_affected })))
    }
}
