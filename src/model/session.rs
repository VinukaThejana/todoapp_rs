use crate::{
    config::{state::AppState, ENV},
    token::{
        claims::ExtendedClaims,
        error::TokenError,
        traits::Token,
        types::{params::TokenParams, response::TokenResponse},
    },
};

pub struct Session {
    pub state: AppState,
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub photo_url: String,
    pub exp: usize,
}

impl Session {
    pub fn new(
        state: AppState,
        user_id: String,
        email: String,
        name: String,
        photo_url: String,
        exp: usize,
    ) -> Self {
        Self {
            state,
            user_id,
            email,
            name,
            photo_url,
            exp,
        }
    }
}

impl Token<ExtendedClaims> for Session {
    fn state(&self) -> AppState {
        self.state.clone()
    }

    fn public_key(&self) -> &[u8] {
        &ENV.session_token_public_key
    }

    fn private_key(&self) -> &[u8] {
        &ENV.session_token_private_key
    }

    async fn create(&self, _: TokenParams) -> Result<TokenResponse, TokenError> {
        let claims = ExtendedClaims::new(
            self.user_id.clone(),
            self.exp,
            self.email.clone(),
            self.name.clone(),
            self.photo_url.clone(),
        );
        let token = self.generate(&claims)?;

        Ok(TokenResponse::Session(token))
    }

    async fn verify(
        &self,
        token: String,
        _: crate::token::TokenType,
    ) -> Result<ExtendedClaims, TokenError>
    where
        ExtendedClaims: Send,
    {
        let claims = self.decode(token)?;
        Ok(claims)
    }
}
