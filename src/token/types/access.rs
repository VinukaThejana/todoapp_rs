use anyhow::anyhow;

use crate::{
    config::{state::AppState, ENV},
    token::{
        claims::{Claims, PrimaryClaims},
        error::TokenError,
        traits::Token,
        TokenType,
    },
};

use super::{params::TokenParams, response::TokenResponse};

pub struct Access {
    pub state: AppState,
    pub user_id: String,
    pub exp: usize,
}

impl Access {
    pub fn new(state: AppState, user_id: String, exp: usize) -> Self {
        Self {
            state,
            user_id,
            exp,
        }
    }
}

impl Token<PrimaryClaims> for Access {
    fn state(&self) -> AppState {
        self.state.clone()
    }

    fn public_key(&self) -> &[u8] {
        &ENV.access_token_public_key
    }

    fn private_key(&self) -> &[u8] {
        &ENV.refresh_token_public_key
    }

    async fn create(&self, params: TokenParams) -> Result<TokenResponse, TokenError> {
        let ajti = params.ajti.clone().unwrap_or(ulid::Ulid::new().to_string());
        let rjti = params.rjti.ok_or_else(|| {
            TokenError::Other(anyhow!("rjti is required, please provide the rjti"))
        })?;

        let claims = PrimaryClaims::new(
            self.user_id.clone(),
            self.exp,
            Some(ajti.clone()),
            Some(rjti.clone()),
        );
        let token = self.generate(&claims)?;

        if params.ajti.is_none() {
            let mut conn = self
                .state()
                .get_redis_conn()
                .await
                .map_err(TokenError::Other)?;

            redis::cmd("SET")
                .arg(TokenType::Access.get_key(&ajti))
                .arg(&rjti)
                .arg("EX")
                .arg(self.exp)
                .query_async(&mut conn)
                .await
                .map_err(|err| TokenError::Other(err.into()))?;
        }

        Ok(TokenResponse::Access(token))
    }
}

impl Access {
    pub async fn refresh(&self, rjti: String) -> Result<String, TokenError> {
        let claims = PrimaryClaims::new(self.user_id.clone(), self.exp, None, Some(rjti.clone()));
        let token = self.generate(&claims)?;

        let mut conn = self
            .state()
            .get_redis_conn()
            .await
            .map_err(TokenError::Other)?;

        let value: Option<String> = redis::cmd("GET")
            .arg(TokenType::Access.get_key(&rjti))
            .query_async(&mut conn)
            .await
            .map_err(|err| TokenError::Other(err.into()))?;

        redis::pipe()
            .cmd("DEL")
            .arg(if let Some(ref v) = value {
                TokenType::Access.get_key(v)
            } else {
                "no_key".to_string()
            })
            .cmd("SET")
            .arg(TokenType::Access.get_key(claims.jti()))
            .arg(&rjti)
            .arg("EX")
            .arg(self.exp)
            .query_async(&mut conn)
            .await
            .map_err(|err| TokenError::Other(err.into()))?;

        Ok(token)
    }
}
