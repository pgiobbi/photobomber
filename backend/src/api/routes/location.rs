use crate::types::location::{GetLocationResponse, PostLocationRequest};
use crate::types::state::AppState;
use actix_web::{HttpResponse, Responder, get, post, web};
use chrono::Utc;
use log::info;
use std::ops::Deref;
use std::panic::Location;
use std::sync::LockResult;

#[get("")]
pub async fn get_location(app_state: web::Data<AppState>) -> impl Responder {
    let location = app_state.location_state.read().unwrap();
    HttpResponse::Ok().json(GetLocationResponse::from(location.deref()))
}

#[post("")]
pub async fn post_location(
    body: web::Json<PostLocationRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    // TODO: guard: admin only
    info!("{:?}", body.0);
    {
        let mut location_state = app_state.location_state.write().unwrap();
        location_state.location = body.0.location.clone();
        location_state.updated_at = Some(Utc::now().timestamp_millis());
    }
    HttpResponse::Ok().json(GetLocationResponse::from(
        app_state.location_state.read().unwrap().deref(),
    ))
}
