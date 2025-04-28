use crate::api::constants::{ADMIN_ID, PUBLIC_ID};
use crate::api::extractors::auth::{
    generate_and_set_auth_cookies, generate_and_set_auth_removal_cookies,
};
use crate::api::libraries::auth::AuthenticatedUser;
use crate::types::auth::LoginRequest;
use crate::types::state::AppState;
use actix_web::{HttpResponse, Responder, web, get, post};
use serde_json::json;

#[post("")]
pub async fn post_login(
    body: web::Json<LoginRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    // 0 for public users, 1 for admins
    let is_admin = app_state.credential_state.is_admin(&body.0);
    let user_id = match is_admin {
        Ok(true) => ADMIN_ID,
        Ok(false) => PUBLIC_ID,
        // Wrong credentials!
        Err(_) => return HttpResponse::Unauthorized().json("Unauthorized"),
    };
    let mut response = generate_and_set_auth_cookies(user_id, is_admin.unwrap())
        .unwrap_or_else(|_| HttpResponse::InternalServerError());

    response.json(json!({
        "access_token": null,
        "refresh_token": null,
        "user_id": user_id,
    }))
}

#[post("")]
pub async fn post_logout() -> impl Responder {
    let mut response = generate_and_set_auth_removal_cookies()
        .unwrap_or_else(|_| HttpResponse::InternalServerError());
    response.json(json!({}))
}

#[get("")]
pub async fn get_profile(user: AuthenticatedUser) -> impl Responder {
    HttpResponse::Ok().json(user)
}
