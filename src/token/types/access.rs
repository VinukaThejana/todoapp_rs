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
use anyhow::anyhow;

pub struct Access {
    pub state: AppState,
    pub user_id: Option<String>,
}

impl Access {
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
            .expect("user_id is required to create a new access token")
    }

    fn claims(&self, ajti: Option<String>, rjti: Option<String>) -> PrimaryClaims {
        PrimaryClaims::new(self.user_id().to_owned(), self.exp(), ajti, rjti)
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
        &ENV.access_token_private_key
    }

    fn exp(&self) -> usize {
        ENV.access_token_expiration
    }

    async fn create(&self, params: TokenParams) -> Result<TokenResponse, TokenError> {
        let ajti = params.ajti.clone().unwrap_or(ulid::Ulid::new().to_string());
        let rjti = params.rjti.ok_or_else(|| {
            TokenError::Other(anyhow!("rjti is required, please provide the rjti"))
        })?;

        let token = self.generate(&self.claims(Some(ajti.clone()), Some(rjti.clone())))?;

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
                .arg(self.exp())
                .query_async(&mut conn)
                .await
                .map_err(|err| TokenError::Other(err.into()))?;
        }

        Ok(TokenResponse::Access(token))
    }
}

impl Access {
    pub async fn refresh(&self, rjti: String) -> Result<String, TokenError> {
        let claims = self.claims(None, Some(rjti.clone()));
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
            .arg(self.exp())
            .query_async(&mut conn)
            .await
            .map_err(|err| TokenError::Other(err.into()))?;

        Ok(token)
    }
}
