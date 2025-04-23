use crate::api::extractors::auth::generate_and_set_auth_cookies;
use actix_web::{HttpResponse, Responder};
use serde_json::json;

pub async fn login() -> impl Responder {
    let user_id = 0;
    let mut response = generate_and_set_auth_cookies(user_id)
        .unwrap_or_else(|_| HttpResponse::InternalServerError());

    response.json(json!({
        "access_token": null,
        "refresh_token": null,
        "user_id": user_id,
    }))
}
