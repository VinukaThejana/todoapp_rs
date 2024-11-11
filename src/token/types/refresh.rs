use anyhow::anyhow;

use super::{params::TokenParams, response::TokenResponse};
use crate::{
    config::{state::AppState, ENV},
    token::{
        claims::{Claims, PrimaryClaims},
        error::TokenError,
        traits::Token,
        TokenType,
    },
};

pub struct Refresh {
    pub state: AppState,
    pub user_id: String,
    pub exp: usize,
}

impl Refresh {
    pub fn new(state: AppState, user_id: String, exp: usize) -> Self {
        Self {
            state,
            user_id,
            exp,
        }
    }
}

impl Token<PrimaryClaims> for Refresh {
    fn state(&self) -> AppState {
        self.state.clone()
    }

    fn public_key(&self) -> &[u8] {
        &ENV.refresh_token_public_key
    }

    fn private_key(&self) -> &[u8] {
        &ENV.refresh_token_private_key
    }

    async fn create(&self, _: TokenParams) -> Result<TokenResponse, TokenError> {
        let claims = PrimaryClaims::new(self.user_id.clone(), self.exp, None, None);
        let token = self.generate(&claims)?;
        let ajti = ulid::Ulid::new().to_string();

        let mut conn = self
            .state
            .rd
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| TokenError::Other(err.into()))?;

        redis::pipe()
            .cmd("SET")
            .arg(TokenType::Refresh.get_key(claims.jti()))
            .arg(&self.user_id)
            .arg("EX")
            .arg(self.exp)
            .ignore()
            .cmd("SET")
            .arg(TokenType::Access.get_key(&ajti))
            .arg(claims.jti())
            .arg("EX")
            .arg(ENV.access_token_expiration)
            .ignore()
            .query_async(&mut conn)
            .await
            .map_err(|err| TokenError::Other(err.into()))?;

        Ok(TokenResponse::Refresh {
            token,
            rjti: claims.jti,
            ajti,
        })
    }
}

impl Refresh {
    pub async fn delete(&self, rjti: &str) -> Result<(), TokenError> {
        let mut conn = self
            .state
            .rd
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| TokenError::Other(err.into()))?;

        let value: Option<String> = redis::cmd("GET")
            .arg(TokenType::Refresh.get_key(rjti))
            .query_async(&mut conn)
            .await
            .map_err(|err| TokenError::Other(err.into()))?;
        let value =
            value.ok_or_else(|| TokenError::Validation(anyhow!("token not found in redis")))?;

        redis::pipe()
            .cmd("DEL")
            .arg(TokenType::Refresh.get_key(rjti))
            .ignore()
            .cmd("DEL")
            .arg(TokenType::Access.get_key(&value))
            .ignore()
            .query_async(&mut conn)
            .await
            .map_err(|err| TokenError::Other(err.into()))?;

        Ok(())
    }
}
