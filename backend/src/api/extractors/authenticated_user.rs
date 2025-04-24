use crate::api::libraries::auth::AuthenticatedUser;
use actix_web::{
    Error, FromRequest, HttpMessage, HttpRequest, dev::Payload, error::ErrorUnauthorized,
};
use std::future::{Ready, ready};

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    /// Retrieves the Authenticated user from the request (injected by AuthMiddleware)
    ///
    /// If not found, the user is not authorized
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
            return ready(Ok(user.clone()));
        }
        ready(Err(ErrorUnauthorized("User not authenticated")))
    }
}
