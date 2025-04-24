use crate::api::constants::{ADMIN_ID, PUBLIC_ID};
use crate::api::extractors::auth::generate_and_set_auth_cookies;
use crate::api::libraries::auth::AuthenticatedUser;
use crate::types::auth::LoginRequest;
use crate::types::state::AppState;
use actix_web::{HttpResponse, Responder, web};
use serde_json::json;

pub async fn login(
    body: web::Json<LoginRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    // 0 for public users, 1 for admins
    let user_id = match app_state.credential_state.is_admin(&body.0) {
        Ok(true) => ADMIN_ID,
        Ok(false) => PUBLIC_ID,
        // Wrong credentials!
        Err(_) => return HttpResponse::Unauthorized().json("Unauthorized"),
    };
    let mut response = generate_and_set_auth_cookies(user_id)
        .unwrap_or_else(|_| HttpResponse::InternalServerError());

    response.json(json!({
        "access_token": null,
        "refresh_token": null,
        "user_id": user_id,
    }))
}

pub async fn get_profile(user: AuthenticatedUser) -> impl Responder {
    HttpResponse::Ok().json(user)
}
