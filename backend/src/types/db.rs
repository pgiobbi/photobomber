use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct DbImage {
    pub id: i32,
    pub filename: String,
    pub join_leaderboard: bool,
    pub karma: i32,
    pub created_at: String,
}