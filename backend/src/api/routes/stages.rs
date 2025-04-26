use crate::types::state::AppState;
use actix_web::{HttpResponse, Responder, get, web};

#[get("")]
pub async fn get_stages(app_state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(&app_state.stages)
}
