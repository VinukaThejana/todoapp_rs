use crate::{
    config::{state::AppState, ENV},
    token::{
        claims::ExtendedClaims,
        error::TokenError,
        traits::Token,
        types::{params::TokenParams, response::TokenResponse},
        TokenType,
    },
};

pub struct Session {
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub photo_url: String,
    pub exp: usize,
}

impl Session {
    pub fn new(user_id: String, email: String, name: String, photo_url: String) -> Self {
        Self {
            user_id,
            email,
            name,
            photo_url,
            exp: ENV.session_token_expiration,
        }
    }
}

impl Token<ExtendedClaims> for Session {
    fn state(&self) -> AppState {
        unimplemented!("state is not needed for the session token")
    }

    fn public_key(&self) -> String {
        ENV.session_token_public_key.clone()
    }

    fn private_key(&self) -> String {
        ENV.session_token_private_key.clone()
    }

    fn create(&self, _: TokenParams) -> Result<TokenResponse, TokenError> {
        let claims = ExtendedClaims::new(
            self.user_id.clone(),
            self.exp,
            self.email.clone(),
            self.name.clone(),
            self.photo_url.clone(),
        );
        let token = self.generate(claims)?;

        Ok(TokenResponse::Session(token))
    }

    fn verify(&self, _: String, _: TokenType) -> Result<ExtendedClaims, TokenError> {
        unimplemented!(
            "verify method is not needed for session token, use the decode function instead"
        )
    }
}
