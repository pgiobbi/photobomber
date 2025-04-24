use crate::types::location::Stage;
use serde::Serialize;
use std::sync::RwLock;

type Bytes = u64;

#[derive(Default, Serialize)]
pub struct LocationState {
    pub location: Option<Stage>,
    pub updated_at: Option<i64>,
}

pub struct AppState {
    /// Path to the upload directory.
    pub upload_dir: String,
    /// Max upload file size (bytes).
    pub max_file_size: Bytes,
    /// Location state (stage, updatedAt)
    pub location_state: RwLock<LocationState>,
}
