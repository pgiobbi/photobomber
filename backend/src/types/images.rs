use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use serde::Serialize;

// Structure for the multipart form
#[derive(Debug, MultipartForm)]
pub struct ImageUploadRequest {
    #[multipart(rename = "images", limit = "1MB")]
    pub images: Vec<TempFile>,
    #[multipart(rename = "joinLeaderboard")]
    pub join_leaderboard: Text<bool>,
}

// Structure for the response
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageUploadResponse {
    pub file_id: String,
    pub file_name: String,
    pub join_leaderboard: bool,
}

// Structure for the count response
#[derive(Serialize)]
pub struct ImageCountResponse {
    pub count: u64,
}
