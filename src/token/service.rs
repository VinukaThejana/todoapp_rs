use super::{
    claims::Claims,
    traits::Token,
    types::{
        access::Access, params::TokenParams, refresh::Refresh, response::TokenResponse,
        session::Session,
    },
};
use crate::entity::user::Model;
use crate::{config::state::AppState, error::AppError};
use serde::{Deserialize, Serialize};

pub async fn create_token<T, U>(token: T, params: TokenParams) -> Result<TokenResponse, AppError>
where
    T: Token<U>,
    U: for<'a> Serialize,
    U: for<'a> Deserialize<'a>,
    U: Claims,
    U: Send + Sync,
{
    token
        .create(params)
        .await
        .map_err(AppError::from_token_error)
}

pub struct RefreshTokenFactory(String, String);
impl RefreshTokenFactory {
    pub fn token(&self) -> &str {
        &self.0
    }

    pub fn rjti(&self) -> &str {
        &self.1
    }
}

pub struct TokenFactory(RefreshTokenFactory, String, String);
impl TokenFactory {
    pub fn refresh(&self) -> &RefreshTokenFactory {
        &self.0
    }

    pub fn access(&self) -> &str {
        &self.1
    }

    pub fn session(&self) -> &str {
        &self.2
    }
}

pub async fn factory(state: AppState, user: &Model) -> Result<TokenFactory, AppError> {
    let (refresh_token, rjti, ajti) = create_token(
        Refresh::new(state.clone(), user.id.clone()),
        TokenParams::default(),
    )
    .await
    .map(|token| {
        let TokenResponse::Refresh { token, rjti, ajti } = token else {
            unreachable!("Refresh token is expected");
        };
        (token, rjti, ajti)
    })?;

    let access_token = create_token(
        Access::new(state.clone(), user.id.clone()),
        TokenParams::default()
            .with_ajti(ajti)
            .with_rjti(rjti.clone()),
    )
    .await
    .map(|token| {
        let TokenResponse::Access(token) = token else {
            unreachable!("Access token is expected");
        };
        token
    })?;

    let session_token = create_token(
        Session::new(
            state.clone(),
            user.id.clone(),
            user.email.clone(),
            user.name.clone(),
            format!(
                "https://api.dicebear.com/9.x/notionists/svg?seed={}",
                &user.name
            ),
        ),
        TokenParams::default(),
    )
    .await
    .map(|token| {
        let TokenResponse::Session(token) = token else {
            unreachable!("Session token is expected");
        };

        token
    })?;

    Ok(TokenFactory(
        RefreshTokenFactory(refresh_token, rjti),
        access_token,
        session_token,
    ))
}
