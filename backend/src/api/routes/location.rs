use crate::api::libraries::auth::AuthenticatedUser;
use crate::types::location::{GetLocationResponse, PostLocationRequest};
use crate::types::state::{AppState, LocationVariant};
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
    user: AuthenticatedUser,
    app_state: web::Data<AppState>,
) -> impl Responder {
    info!("{:?} {:?}", body.0, user);
    if !user.is_admin() {
        return HttpResponse::Forbidden().json("Not allowed");
    };
    // Dedicated scope to release the RwLock write ASAP
    {
        let mut location_state = app_state.location_state.write().unwrap();

        // Throw if the Tomorrowland stage is not found in the ones from the Tomorrowland API
        if let Some(LocationVariant::Tomorrowland(selected_stage)) = &body.location {
            if app_state
                .stages
                .iter()
                .find(|&stage| stage.id == selected_stage.id && stage.name == selected_stage.name)
                .is_none()
            {
                return HttpResponse::BadRequest().json("Stage not found");
            }
        }
        location_state.location = body.0.location.clone();
        location_state.updated_at = Some(Utc::now().timestamp_millis());
    }
    HttpResponse::Ok().json(GetLocationResponse::from(
        app_state.location_state.read().unwrap().deref(),
    ))
}
