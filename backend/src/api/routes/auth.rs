use crate::api::extractors::auth::generate_and_set_auth_cookies;
use crate::types::auth::LoginRequest;
use crate::types::state::AppState;
use actix_web::{HttpResponse, Responder, web};
use serde_json::json;

pub async fn login(
    body: web::Json<LoginRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    // 0 for public users, 1 for admins
    let is_admin = app_state.credential_state.is_admin(&body.0);
    let user_id = if is_admin { 1 } else { 0 };
    let mut response = generate_and_set_auth_cookies(user_id)
        .unwrap_or_else(|_| HttpResponse::InternalServerError());

    response.json(json!({
        "access_token": null,
        "refresh_token": null,
        "user_id": user_id,
    }))
}
