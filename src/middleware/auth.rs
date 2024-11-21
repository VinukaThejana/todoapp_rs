use crate::{
    config::state::AppState,
    error::AppError,
    token::{traits::Token, types::access::Access, TokenType},
};
use anyhow::anyhow;
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::IntoResponse,
};

pub async fn auth_m(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let access_token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| {
            auth_header
                .to_str()
                .ok()
                .and_then(|s| s.strip_prefix("Bearer "))
        })
        .ok_or_else(|| AppError::Unauthorized(anyhow!("Missing Authorization header")))?
        .to_owned();

    Access::default(state)
        .verify(access_token, TokenType::Access)
        .await
        .map_err(AppError::from_token_error)?;

    Ok(next.run(req).await)
}
