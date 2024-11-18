use crate::token::constants;
use crate::token::cookies::{CookieManager, CookieParams};
use crate::token::service::factory;
use crate::{
    config::{state::AppState, ENV},
    database,
    error::AppError,
    model::user::{CreateUserReq, LoginUserReq},
};
use anyhow::anyhow;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, HeaderValue};
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use validator::Validate;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserReq>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let password = bcrypt::hash(payload.password, bcrypt::DEFAULT_COST).map_err(|err| {
        AppError::Other(
            anyhow::Error::new(err)
                .context("Failed to hash the password")
                .context(format!("Failed to create user: {}", payload.email)),
        )
    })?;

    database::user::create(payload.email, payload.name, password, &state.db)
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
    );
    let session_cookie = CookieManager::create(
        constants::SESSION_TOKEN_COOKIE_NAME,
        tokens.session(),
        CookieParams::default()
            .with_age(ENV.session_token_expiration)
            .with_http_only(false),
    );

    let mut headers = HeaderMap::new();

    headers.append(
        "X-New-Access-Token",
        HeaderValue::from_str(tokens.access()).unwrap(),
    );
    headers.append(
        SET_COOKIE,
        HeaderValue::from_str(&refresh_cookie.to_string()).unwrap(),
    );
    headers.append(
        SET_COOKIE,
        HeaderValue::from_str(&session_cookie.to_string()).unwrap(),
    );

    Ok((
        headers,
        Json(json!({
            "status": "ok",
        })),
    ))
}
