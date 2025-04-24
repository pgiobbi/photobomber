use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: Option<String>,
    #[serde(skip_serializing)]
    pub password: Option<String>,
}
