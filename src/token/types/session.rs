use crate::{
    config::{state::AppState, ENV},
    model::user::UserDetails,
    token::{
        claims::ExtendedClaims,
        error::TokenError,
        traits::Token,
        types::{params::TokenParams, response::TokenResponse},
    },
};

pub struct Session {
    pub state: AppState,
    pub user_id: Option<String>,
    pub user: Option<UserDetails>,
}

impl Session {
    pub fn default(state: AppState) -> Self {
        Self {
            state,
            user_id: None,
            user: None,
        }
    }

    pub fn new(
        state: AppState,
        user_id: String,
        email: String,
        name: String,
        photo_url: String,
    ) -> Self {
        Self {
            state,
            user_id: Some(user_id),
            user: Some(UserDetails {
                email,
                name,
                photo_url,
            }),
        }
    }

    fn user(&self) -> (&str, &UserDetails) {
        self.user_id
            .as_deref()
            .zip(self.user.as_ref())
            .expect("please provide the user_details and the user_id to create a new session token")
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

    fn exp(&self) -> usize {
        ENV.session_token_expiration
    }

    async fn create(&self, _: TokenParams) -> Result<TokenResponse, TokenError> {
        let (user_id, user) = self.user();

        Ok(TokenResponse::Session(self.generate(
            &ExtendedClaims::new(
                user_id.to_owned(),
                self.exp(),
                user.email.clone(),
                user.name.clone(),
                user.photo_url.clone(),
            ),
        )?))
    }

    async fn verify(
        &self,
        token: String,
        _: crate::token::TokenType,
    ) -> Result<ExtendedClaims, TokenError> {
        self.decode(token)
    }
}
