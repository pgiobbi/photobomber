use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use serde::{Deserialize, Serialize};

// Structure for the multipart form
#[derive(Debug, MultipartForm)]
pub struct ImageUploadRequest {
    // Keep this >= the handler's max_file_size (DEFAULT_MAX_FILE_SIZE, 6MB) and <= the nginx
    // proxy body cap (12M). The handler enforces the real per-file limit and returns a clean
    // 400; a smaller limit here makes the MultipartForm extractor reject larger files with a
    // bare 400 before the handler ever runs.
    #[multipart(rename = "images", limit = "12MB")]
    pub images: Vec<TempFile>,
    #[multipart(rename = "isPublic")]
    pub is_public: Text<bool>,
}

// Structure for the response
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageUploadResponse {
    pub file_id: String,
    pub file_name: String,
    pub is_public: bool,
    /// Internal parameter to track whether the image was already present in the system.
    #[serde(skip_serializing)]
    pub _is_new: bool,
}

// Structure for the count response
#[derive(Serialize)]
pub struct ImageCountResponse {
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardOrder {
    Karma,
    Time,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct GetLeaderboardQueryParams {
    pub order_by: Option<LeaderboardOrder>,
    pub ascending: Option<bool>,
}
