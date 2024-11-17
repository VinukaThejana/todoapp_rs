use crate::config::env::EnvMode;
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
use cookie::{time::Duration, Cookie};
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

    let refresh_cookie = Cookie::build(("todoapp_rs_refresh_token", tokens.refresh().token()))
        .http_only(true)
        .path("/")
        .secure(EnvMode::is_prd(&ENV.env))
        .domain(&*ENV.domain)
        .max_age(Duration::seconds(ENV.refresh_token_expiration as i64));

    let session_token = Cookie::build(("todoapp_rs_session_token", tokens.session()))
        .http_only(false)
        .path("/")
        .secure(EnvMode::is_prd(&ENV.env))
        .domain(&*ENV.domain)
        .max_age(Duration::seconds(ENV.session_token_expiration as i64));

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
        HeaderValue::from_str(&session_token.to_string()).unwrap(),
    );

    Ok((
        headers,
        Json(json!({
            "status": "ok",
        })),
    ))
}
