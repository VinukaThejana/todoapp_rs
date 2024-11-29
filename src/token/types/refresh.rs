use super::{params::TokenParams, response::TokenResponse};
use crate::{
    config::{state::AppState, ENV},
    database,
    token::{
        claims::{Claims, PrimaryClaims},
        error::TokenError,
        traits::Token,
        TokenType,
    },
};
use anyhow::anyhow;

pub struct Refresh {
    pub state: AppState,
    pub user_id: Option<String>,
}

impl Refresh {
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
            .expect("provide the user_id to create a new refresh token")
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

    fn exp(&self) -> usize {
        ENV.refresh_token_expiration
    }

    async fn create(&self, _: TokenParams) -> Result<TokenResponse, TokenError> {
        let claims = PrimaryClaims::new(self.user_id().to_owned(), self.exp(), None, None);
        let token = self.generate(&claims)?;
        let ajti = ulid::Ulid::new().to_string();

        let mut conn = self
            .state()
            .get_redis_conn()
            .await
            .map_err(TokenError::Other)?;

        redis::pipe()
            .cmd("SET")
            .arg(TokenType::Refresh.get_key(claims.jti()))
            .arg(&ajti)
            .arg("EX")
            .arg(self.exp())
            .ignore()
            .cmd("SET")
            .arg(TokenType::Access.get_key(&ajti))
            .arg(self.user_id())
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
        database::session::delete(rjti.to_string(), &self.state.db)
            .await
            .map_err(|err| TokenError::Other(anyhow!(err)))?;

        let mut conn = self
            .state()
            .get_redis_conn()
            .await
            .map_err(TokenError::Other)?;

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
