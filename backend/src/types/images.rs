use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use serde::Serialize;

// Structure for the multipart form
#[derive(Debug, MultipartForm)]
pub struct ImageUploadRequest {
    #[multipart(rename = "images", limit = "1MB")]
    pub images: Vec<TempFile>,
}

// Structure for the response
#[derive(Serialize)]
pub struct ImageUploadResponse {
    pub file_id: String,
    pub file_name: String,
}

// Structure for the count response
#[derive(Serialize)]
pub struct ImageCountResponse {
    pub count: u64,
}
