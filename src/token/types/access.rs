use crate::{
    config::{state::AppState, ENV},
    token::{
        claims::PrimaryClaims,
        error::TokenError,
        traits::{Token, TokenParams, TokenResponse},
        TokenType,
    },
};
use redis::Commands;

pub struct Access {
    pub state: AppState,
    pub user_id: String,
    pub exp: usize,
}

impl Access {
    pub fn new(state: AppState, user_id: String) -> Self {
        Self {
            state,
            user_id,
            exp: ENV.access_token_expiration,
        }
    }
}

impl Token<PrimaryClaims> for Access {
    fn state(&self) -> AppState {
        self.state.clone()
    }

    fn public_key(&self) -> String {
        ENV.access_token_public_key.clone()
    }

    fn private_key(&self) -> String {
        ENV.access_token_private_key.clone()
    }

    fn create(&self, params: TokenParams) -> Result<TokenResponse, TokenError> {
        let rjti = params.rjti.unwrap_or_else(|| {
            panic!("please provide the jti of the refresh token to create the access token")
        });

        let (claims, jti) = PrimaryClaims::new(
            self.user_id.clone(),
            self.exp,
            params.ajti.clone(),
            Some(rjti.clone()),
        );
        let token = self.generate(claims)?;

        if params.ajti.is_none() {
            let mut conn = self
                .state
                .rd
                .get_connection()
                .map_err(|err| TokenError::Other(err.to_string()))?;

            let _: () = conn
                .set(TokenType::Access.get_key(jti.as_str()), rjti.as_str())
                .map_err(|err| TokenError::Other(err.to_string()))?;
        }

        Ok(TokenResponse::Access(token))
    }
}
