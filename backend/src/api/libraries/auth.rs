use crate::api::constants::APP_NAME;
use actix_web::ResponseError;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::Debug;
use strum::Display;

const ACCESS_TOKEN_TYPE: &str = "AccessToken";
const REFRESH_TOKEN_TYPE: &str = "RefreshToken";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub token_type: String,
    pub sub: String, // RFC 7519 wants strings :(
}

#[derive(Debug, Display)]
pub enum TokenError {
    Expired,
    Invalid,
    WrongType,
    ParseError,
    ConfigError(&'static str),
}

impl ResponseError for TokenError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub claims: TokenClaims,
}

impl AuthenticatedUser {
    pub fn is_admin(&self) -> bool {
        self.user_id == 1
    }
}

#[derive(Clone)]
pub struct TokenRefresh {
    pub access_token: String,
    pub refresh_token: String,
}

/// Validates an access token.
pub fn validate_access_token(token: &str) -> Result<AuthenticatedUser, TokenError> {
    validate_token(token, ACCESS_TOKEN_TYPE)
}

/// Validates a refresh token.
pub fn validate_refresh_token(token: &str) -> Result<AuthenticatedUser, TokenError> {
    validate_token(token, REFRESH_TOKEN_TYPE)
}

/// Returns the access token duration in minutes
pub fn get_access_token_duration() -> Duration {
    Duration::minutes(get_duration("FPX_ACCESS_TOKEN_DURATION_MINUTES").unwrap_or(15))
}

/// Returns the refresh token duration in minutes
pub fn get_refresh_token_duration() -> Duration {
    Duration::minutes(get_duration("FPX_REFRESH_TOKEN_DURATION_MINUTES").unwrap_or(10080))
}

/// Checks whether to refresh the access token (less than 20% to expiration)
pub fn should_refresh_token(claims: &TokenClaims) -> bool {
    let duration = get_access_token_duration();
    let now_seconds = Utc::now().timestamp();
    let progress = (now_seconds - claims.iat) as f64 / duration.num_seconds() as f64;
    progress > 0.8
}

/// Generates an access token.
pub fn generate_access_token(user_id: i64) -> Result<String, TokenError> {
    generate_token(
        user_id,
        ACCESS_TOKEN_TYPE,
        get_access_token_duration().num_minutes(),
    )
}

/// Generates a refresh token.
pub fn generate_refresh_token(user_id: i64) -> Result<String, TokenError> {
    generate_token(
        user_id,
        REFRESH_TOKEN_TYPE,
        get_refresh_token_duration().num_minutes(),
    )
}

/// Generates new access/refresh tokens.
pub fn generate_tokens(user_id: i64) -> Result<(String, String), TokenError> {
    Ok((
        generate_access_token(user_id)?,
        generate_refresh_token(user_id)?,
    ))
}

/// Returns the access/refresh token duration.
pub fn get_duration(env_var: &str) -> Result<i64, TokenError> {
    env::var(env_var)
        .map_err(|_| TokenError::ConfigError("Duration not configured"))
        .and_then(|val| {
            val.parse::<i64>()
                .map_err(|_| TokenError::ConfigError("Invalid duration format"))
        })
}

/// Validates an access/refresh token.
fn validate_token(token: &str, token_type: &str) -> Result<AuthenticatedUser, TokenError> {
    let jwt_secret = get_jwt_secret()?;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[APP_NAME]);
    validation.set_required_spec_claims(&["exp", "iat", "iss", "sub"]);

    let token_data = match decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => data,
        Err(_) => return Err(TokenError::Invalid),
    };

    if token_data.claims.token_type != token_type {
        return Err(TokenError::WrongType);
    }

    Ok(AuthenticatedUser {
        claims: token_data.claims.clone(),
        user_id: token_data
            .claims
            .sub
            .parse::<i64>()
            .map_err(|_| TokenError::ParseError)?,
    })
}

/// Returns the jwt secret.
fn get_jwt_secret() -> Result<String, TokenError> {
    Ok(env::var("FPX_JWT_SECRET").unwrap_or("your-jwt-secret-here".to_string()))
}

/// Internal function that generates an access/refresh token.
fn generate_token(
    user_id: i64,
    token_type: &str,
    duration_minutes: i64,
) -> Result<String, TokenError> {
    let jwt_secret = get_jwt_secret()?;
    let now = Utc::now();
    let exp = (now + Duration::minutes(duration_minutes)).timestamp();

    let claims = TokenClaims {
        exp,
        iat: now.timestamp(),
        iss: APP_NAME.to_string(),
        token_type: token_type.to_string(),
        sub: user_id.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| TokenError::Invalid)
}
