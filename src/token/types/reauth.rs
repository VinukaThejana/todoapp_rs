use super::{params::TokenParams, response::TokenResponse};
use crate::{
    config::{state::AppState, ENV},
    token::{claims::PrimaryClaims, error::TokenError, traits::Token},
};

pub struct Reauth {
    pub state: AppState,
    pub user_id: Option<String>,
}

impl Reauth {
    pub fn default(state: AppState) -> Self {
        Self {
            state,
            user_id: None,
        }
    }

    pub fn new(state: AppState, user_id: String) -> Self {
        Self {
            state,
            user_id: Some(user_id),
        }
    }

    fn user_id(&self) -> &str {
        self.user_id
            .as_deref()
            .expect("user_id is required to create a new reauth token")
    }

    fn claims(&self, _: Option<String>, _: Option<String>) -> PrimaryClaims {
        PrimaryClaims::new(self.user_id().to_owned(), self.exp(), None, None)
    }
}

impl Token<PrimaryClaims> for Reauth {
    fn state(&self) -> AppState {
        self.state.clone()
    }

    fn public_key(&self) -> &[u8] {
        &ENV.access_token_public_key
    }

    fn private_key(&self) -> &[u8] {
        &ENV.access_token_private_key
    }

    fn exp(&self) -> usize {
        ENV.access_token_expiration
    }

    async fn create(&self, _: TokenParams) -> Result<TokenResponse, TokenError> {
        Ok(TokenResponse::Reauth(
            self.generate(&self.claims(None, None))?,
        ))
    }

    async fn verify(
        &self,
        token: String,
        _: crate::token::TokenType,
    ) -> Result<PrimaryClaims, TokenError> {
        self.decode(token)
    }
}
