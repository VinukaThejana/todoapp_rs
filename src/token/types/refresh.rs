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
use ulid::Ulid;

pub struct Refresh {
    pub state: AppState,
    pub user_id: String,
    pub exp: usize,
}

impl Refresh {
    pub fn new(state: AppState, user_id: String) -> Self {
        Self {
            state,
            user_id,
            exp: ENV.refresh_token_expiration,
        }
    }
}

impl Token<PrimaryClaims> for Refresh {
    fn state(&self) -> AppState {
        self.state.clone()
    }

    fn public_key(&self) -> String {
        ENV.refresh_token_public_key.clone()
    }

    fn private_key(&self) -> String {
        ENV.refresh_token_private_key.clone()
    }

    fn create(&self, _: TokenParams) -> Result<TokenResponse, TokenError> {
        let (claims, rjti) = PrimaryClaims::new(self.user_id.clone(), self.exp, None, None);
        let ajti = Ulid::new().to_string();

        let token = self.generate(claims)?;

        let mut conn = self
            .state
            .rd
            .get_connection()
            .map_err(|err| TokenError::Other(err.to_string()))?;

        redis::pipe()
            .set(
                TokenType::Refresh.get_key(rjti.as_str()),
                self.user_id.clone(),
            )
            .set(TokenType::Access.get_key(ajti.as_str()), rjti.as_str())
            .query(&mut conn)
            .map_err(|err| TokenError::Other(err.to_string()))?;

        Ok(TokenResponse::Refresh { token, rjti, ajti })
    }
}

impl Refresh {
    fn delete(&self, jti: String) -> Result<(), TokenError> {
        let mut conn = self
            .state
            .rd
            .get_connection()
            .map_err(|err| TokenError::Other(err.to_string()))?;

        let value: Option<String> = conn
            .get(TokenType::Refresh.get_key(jti.as_str()))
            .map_err(|err| TokenError::Other(err.to_string()))?;
        let ajti =
            value.ok_or_else(|| TokenError::Validation("token not found on redis".to_string()))?;

        redis::pipe()
            .del(TokenType::Refresh.get_key(jti.as_str()))
            .del(TokenType::Access.get_key(ajti.as_str()))
            .query(&mut conn)
            .map_err(|err| TokenError::Other(err.to_string()))?;

        Ok(())
    }
}
