use crate::api::libraries::auth::{
    generate_tokens, get_access_token_duration, get_refresh_token_duration,
};
use actix_web::error::{ErrorUnauthorized, HttpError};
use actix_web::{
    Error, HttpResponse, HttpResponseBuilder,
    cookie::{Cookie, time::Duration as CookieDuration},
    dev::ServiceRequest,
};

/// Generates access/refresh tokens and sets them in an empty response.
pub(crate) fn generate_and_set_auth_cookies(user_id: i64) -> Result<HttpResponseBuilder, ()> {
    let mut res = HttpResponse::Ok();
    let (access_tk, refresh_tk) = generate_tokens(user_id).unwrap();

    if let Ok((access_cookie, refresh_cookie)) = build_auth_cookies(&access_tk, &refresh_tk) {
        res.cookie(access_cookie);
        res.cookie(refresh_cookie);
    } else {
        return Err(());
    }
    Ok(res)
}

/// Resets the access/refresh cookies and sets them in an empty response.
pub(crate) fn generate_and_set_auth_removal_cookies() -> Result<HttpResponseBuilder, ()> {
    let mut res = HttpResponse::Ok();
    if let Ok((access_cookie, refresh_cookie)) = build_auth_removal_cookies() {
        res.cookie(access_cookie);
        res.cookie(refresh_cookie);
    } else {
        return Err(());
    }
    Ok(res)
}

/// Extracts access/refresh tokens from cookies.
pub(crate) fn extract_tokens(req: &ServiceRequest) -> Result<(String, String), Error> {
    let access_token = req
        .cookie("access_token")
        .ok_or_else(|| ErrorUnauthorized("No access token"))?
        // .ok_or_else(|| ApiError::from_code(ApiErrorCode::Unauthorized))? // TODO
        .value()
        .to_string();

    let refresh_token = req
        .cookie("refresh_token")
        .ok_or_else(|| ErrorUnauthorized("No refresh token"))?
        // .ok_or_else(|| ApiError::from_code(ApiErrorCode::Unauthorized))? // TODO
        .value()
        .to_string();

    Ok((access_token, refresh_token))
}

/// Sets the access/refresh cookies in a response
pub(crate) fn set_auth_cookies<B>(
    res: &mut HttpResponse<B>,
    access_token: &str,
    refresh_token: &str,
) -> Result<(), ()> {
    match build_auth_cookies(access_token, refresh_token) {
        Ok((access_cookie, refresh_cookie)) => {
            if let Err(_) = res.add_cookie(&access_cookie) {
                return Err(());
            }
            if let Err(_) = res.add_cookie(&refresh_cookie) {
                return Err(());
            }
            Ok(())
        }
        Err(_) => Err(()),
    }
}

/// Builds the auth cookies. These can be set in a response.
pub(crate) fn build_auth_cookies<'a>(
    access_token: &'a str,
    refresh_token: &'a str,
) -> Result<(Cookie<'a>, Cookie<'a>), HttpError> {
    let access_cookie = Cookie::build("access_token", access_token.to_owned())
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(CookieDuration::minutes(
            get_access_token_duration().num_minutes(),
        ))
        .finish();

    let refresh_cookie = Cookie::build("refresh_token", refresh_token.to_owned())
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(CookieDuration::minutes(
            get_refresh_token_duration().num_minutes(),
        ))
        .finish();

    Ok((access_cookie, refresh_cookie))
}

/// Builds the auth removal cookies. These can be set in a response.
pub(crate) fn build_auth_removal_cookies<'a>() -> Result<(Cookie<'a>, Cookie<'a>), HttpError> {
    let mut access_cookie = Cookie::build("access_token", "")
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(CookieDuration::minutes(0))
        .finish();
    access_cookie.make_removal();

    let mut refresh_cookie = Cookie::build("refresh_token", "")
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(CookieDuration::minutes(0))
        .finish();
    refresh_cookie.make_removal();

    Ok((access_cookie, refresh_cookie))
}
