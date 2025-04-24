use crate::types::auth::LoginRequest;
use crate::types::location::Stage;
use serde::Serialize;
use std::env;
use std::sync::RwLock;

type Bytes = u64;

#[derive(Serialize)]
pub struct CredentialState {
    #[serde(skip_serializing)]
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
}

impl CredentialState {
    pub fn from_env() -> CredentialState {
        CredentialState {
            username: env::var("PHOTOBOMBER_ADMIN_USERNAME").unwrap(),
            password: env::var("PHOTOBOMBER_ADMIN_PASSWORD").unwrap(),
        }
    }

    pub fn is_admin(&self, login_request: &LoginRequest) -> bool {
        match login_request {
            LoginRequest {
                username: Some(username),
                password: Some(password),
            } => self.username.eq(username) && self.password.eq(password),
            _ => false,
        }
    }
}

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
    /// Location state (stage, updatedAt).
    pub location_state: RwLock<LocationState>,
    /// Credential state (Admin credentials).
    pub credential_state: CredentialState,
}
