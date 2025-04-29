use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use serde::{Deserialize, Serialize};

// Structure for the multipart form
#[derive(Debug, MultipartForm)]
pub struct ImageUploadRequest {
    #[multipart(rename = "images", limit = "1MB")]
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
