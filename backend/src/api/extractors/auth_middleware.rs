use crate::api::extractors::auth::{extract_tokens, set_auth_cookies};
use crate::api::libraries::auth::{
    TokenRefresh, generate_tokens, should_refresh_token, validate_access_token,
    validate_refresh_token,
};
use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthMiddleware<S> {
    service: Arc<S>,
}

#[derive(Debug)]
pub struct AuthMiddlewareFactory;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Arc::new(service),
        }))
    }
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = Arc::clone(&self.service);

        Box::pin(async move {
            // Extract tokens from cookies
            let (access_token, refresh_token) = match extract_tokens(&req) {
                Ok(tokens) => tokens,
                Err(e) => return Err(e),
            };

            // Validate access token
            match validate_access_token(&access_token) {
                Ok(user) => {
                    // Check if token needs refresh
                    if should_refresh_token(&user.claims) {
                        match generate_tokens(user.user_id) {
                            Ok((new_access, new_refresh)) => {
                                // Set new cookies in response
                                req.extensions_mut().insert(TokenRefresh {
                                    access_token: new_access,
                                    refresh_token: new_refresh,
                                });
                            }
                            Err(e) => return Err(Error::from(e)),
                        }
                    }

                    // Add user to request extensions
                    req.extensions_mut().insert(user);
                }
                Err(_) => {
                    // Try to use refresh token
                    match validate_refresh_token(&refresh_token) {
                        Ok(user) => match generate_tokens(user.user_id) {
                            Ok((new_access, new_refresh)) => {
                                req.extensions_mut().insert(TokenRefresh {
                                    access_token: new_access,
                                    refresh_token: new_refresh,
                                });
                                req.extensions_mut().insert(user);
                            }
                            Err(e) => return Err(Error::from(e)),
                        },
                        Err(e) => return Err(Error::from(e)),
                    }
                }
            }

            let res = svc.call(req).await?;

            // Ensure the req borrow ends before calling `res.into_parts`
            let token_opt = res.request().extensions().get::<TokenRefresh>().cloned();

            if let Some(token) = token_opt {
                let (req, mut res) = res.into_parts();
                let is_admin = req.path().starts_with("/api/admin");
                let _ = set_auth_cookies(
                    &mut res,
                    &token.access_token,
                    &token.refresh_token,
                    is_admin,
                );
                Ok(ServiceResponse::new(req, res))
            } else {
                Ok(res)
            }
        })
    }
}
