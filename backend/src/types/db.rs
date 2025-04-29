use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DbImage {
    pub id: i32,
    pub filename: String,
    pub is_public: bool,
    pub karma: i32,
    pub created_at: String,
}