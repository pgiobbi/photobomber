use crate::api::libraries::auth::AuthenticatedUser;
use crate::types::db::{DbImage, DbParam};
use crate::types::location::{GetLocationResponse, PostLocationRequest};
use crate::types::state::{AppState, LocationVariant};
use actix_web::{Error, HttpResponse, Responder, get, post, web};
use chrono::Utc;
use log::{error, info};
use std::ops::Deref;
use std::panic::Location;
use std::sync::LockResult;

#[get("")]
pub async fn get_params(app_state: web::Data<AppState>) -> Result<impl Responder, Error> {
    // Execute the query and fetch rows
    let entries: Vec<DbParam> = sqlx::query_as("SELECT * FROM params")
        .fetch_all(&app_state.db_pool)
        .await
        .map_err(|e| {
            error!("Database error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch images")
        })?;
    Ok(HttpResponse::Ok().json(entries))
}

#[post("")]
async fn upsert_param(
    param: web::Json<DbParam>,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO params (key, value, description)
        VALUES ($1, $2, $3)
        ON CONFLICT (key) DO UPDATE
        SET value = $2, description = $3
        "#,
    )
    .bind(&param.key)
    .bind(&param.value)
    .bind(&param.description)
    .execute(&app_state.db_pool)
    .await
    .map_err(|e| {
        error!("Database error during upsert: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to upsert parameter")
    })?;

    if result.rows_affected() == 0 {
        error!("No rows affected during upsert for key: {}", param.key);
        return Err(actix_web::error::ErrorInternalServerError("Upsert failed"));
    }

    Ok(HttpResponse::Ok().json(&param))
}