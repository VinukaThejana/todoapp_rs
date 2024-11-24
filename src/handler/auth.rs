use crate::model::user::ReAuthUserReq;
use crate::token::claims::Claims;
use crate::token::cookies::{CookieManager, CookieParams};
use crate::token::service::{create_token, factory};
use crate::token::traits::Token;
use crate::token::types::access::Access;
use crate::token::types::params::TokenParams;
use crate::token::types::reauth::Reauth;
use crate::token::types::refresh::Refresh;
use crate::token::types::response::TokenResponse;
use crate::token::{constants, TokenType};
use crate::{
    config::{state::AppState, ENV},
    database,
    error::AppError,
    model::user::{CreateUserReq, LoginUserReq},
};
use anyhow::anyhow;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, HeaderValue};
use axum::Extension;
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use validator::Validate;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserReq>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    database::user::create(payload, &state.db)
        .await
        .map_err(AppError::from_db_error)?;

    Ok(Json(json!({
        "status": "ok",
    })))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginUserReq>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let user = database::user::find_by_email(&payload.email, &state.db)
        .await
        .map_err(AppError::from_db_error)?
        .ok_or_else(|| {
            AppError::NotFound(anyhow!("User with email {} not found", &payload.email))
        })?;

    if !bcrypt::verify(&payload.password, &user.password).map_err(|err| {
        AppError::Other(anyhow::Error::new(err).context("Failed to verify the password"))
    })? {
        return Err(AppError::IncorrectCredentials(anyhow!(
            "Incorrect password"
        )));
    };

    let tokens = factory(state.clone(), &user).await?;
    let rjti = tokens.refresh().rjti().to_string();

    tokio::spawn(async move {
        let _ = database::session::delete_expired(&user.id, &state.db)
            .await
            .map_err(|err| {
                log::error!(
                    "{}",
                    anyhow::Error::new(err).context("Failed to delete expired sessions")
                );
            });
        let _ = database::session::create(user.id, rjti, ENV.refresh_token_expiration, &state.db)
            .await
            .map_err(|err| {
                log::error!(
                    "{}",
                    anyhow::Error::new(err).context("Failed to create a new session")
                )
            });
    });

    let refresh_cookie = CookieManager::create(
        constants::REFRESH_TOKEN_COOKIE_NAME,
        tokens.refresh().token(),
        CookieParams::default()
            .with_age(ENV.refresh_token_expiration)
            .with_http_only(true),
    )
    .to_string();
    let session_cookie = CookieManager::create(
        constants::SESSION_TOKEN_COOKIE_NAME,
        tokens.session(),
        CookieParams::default()
            .with_age(ENV.session_token_expiration)
            .with_http_only(false),
    )
    .to_string();

    let mut headers = HeaderMap::new();

    headers.append(
        "X-New-Access-Token",
        HeaderValue::from_str(tokens.access()).unwrap(),
    );
    headers.append(SET_COOKIE, HeaderValue::from_str(&refresh_cookie).unwrap());
    headers.append(SET_COOKIE, HeaderValue::from_str(&session_cookie).unwrap());

    Ok((
        headers,
        Json(json!({
            "status": "ok",
        })),
    ))
}

pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let claims = Refresh::default(state.clone())
        .verify(
            CookieManager::get(&headers, constants::REFRESH_TOKEN_COOKIE_NAME)
                .ok_or_else(|| AppError::Unauthorized(anyhow!("Refresh token not found")))?
                .value()
                .to_owned(),
            TokenType::Refresh,
        )
        .await
        .map_err(AppError::from_token_error)?;

    let access_token = Access::new(state.clone(), claims.sub)
        .refresh(claims.rjti)
        .await
        .map_err(AppError::from_token_error)?;

    let mut headers = HeaderMap::new();

    headers.append(
        "X-New-Access-Token",
        HeaderValue::from_str(&access_token).unwrap(),
    );

    Ok((
        headers,
        Json(json!({
            "status": "ok"
        })),
    ))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let refresh = Refresh::default(state.clone());

    let claims = refresh
        .verify(
            CookieManager::get(&headers, constants::REFRESH_TOKEN_COOKIE_NAME)
                .ok_or_else(|| AppError::Unauthorized(anyhow!("Refresh token not found")))?
                .value()
                .to_owned(),
            TokenType::Refresh,
        )
        .await
        .map_err(AppError::from_token_error)?;
    refresh
        .delete(claims.rjti())
        .await
        .map_err(AppError::from_token_error)?;

    let refresh_cookie = CookieManager::delete(
        constants::REFRESH_TOKEN_COOKIE_NAME,
        CookieParams::default(),
    )
    .to_string();

    let session_cookie = CookieManager::delete(
        constants::SESSION_TOKEN_COOKIE_NAME,
        CookieParams::default(),
    )
    .to_string();

    let mut headers = HeaderMap::new();

    headers.append(SET_COOKIE, HeaderValue::from_str(&refresh_cookie).unwrap());
    headers.append(SET_COOKIE, HeaderValue::from_str(&session_cookie).unwrap());

    Ok((
        headers,
        Json(json!({
            "status": "ok"
        })),
    ))
}

pub async fn reauth(
    State(state): State<AppState>,
    Json(payload): Json<ReAuthUserReq>,
    Extension(user_id): Extension<String>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let user = database::user::find_by_id(user_id, &state.db)
        .await
        .map_err(AppError::from_db_error)?
        .ok_or_else(|| AppError::NotFound(anyhow!("User not found")))?;

    if !bcrypt::verify(&payload.password, &user.password).map_err(|err| {
        AppError::Other(anyhow::Error::new(err).context("Failed to verify the password"))
    })? {
        return Err(AppError::IncorrectCredentials(anyhow!(
            "Incorrect password"
        )));
    }

    let reauth_token = create_token(Reauth::new(state.clone(), user.id), TokenParams::default())
        .await
        .map(|token| {
            let TokenResponse::Reauth(token) = token else {
                unreachable!("Reauth token is expected");
            };
            token
        })?;

    let mut headers = HeaderMap::new();

    headers.append(
        "X-New-Reauth-Token",
        HeaderValue::from_str(&reauth_token).unwrap(),
    );

    Ok((
        headers,
        Json(json!({
            "status": "ok"
        })),
    ))
}
