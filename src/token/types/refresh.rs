use crate::{
    config::{state::AppState, ENV},
    token::{
        claims::PrimaryClaims,
        error::TokenError,
        traits::{Token, TokenResponse},
        TokenType,
    },
};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use redis::Commands;
use ulid::Ulid;

pub struct Refresh {
    pub public_key: String,
    pub private_key: String,
    pub user_id: String,
    pub exp: usize,
}

impl Refresh {
    pub fn new(user_id: String) -> Self {
        Self {
            public_key: ENV.refresh_token_public_key.clone(),
            private_key: ENV.refresh_token_private_key.clone(),
            user_id,
            exp: ENV.refresh_token_expiration,
        }
    }
}

impl Token<PrimaryClaims> for Refresh {
    fn create(&self, state: AppState) -> Result<TokenResponse, TokenError> {
        let claims = PrimaryClaims::new(self.user_id.clone(), self.exp, None, None);
        let token = jsonwebtoken::encode(
            &Header::new(Algorithm::RS256),
            &claims,
            &EncodingKey::from_rsa_pem(self.secret(self.private_key.clone())?.as_bytes())
                .map_err(|err| TokenError::Creation(err.to_string()))?,
        )
        .map_err(|err| TokenError::Creation(err.to_string()))?;
        let ajti = Ulid::new().to_string();

        let mut conn = state
            .rd
            .get_connection()
            .map_err(|err| TokenError::Other(err.to_string()))?;

        redis::pipe()
            .set(
                TokenType::Refresh.get_key(claims.jti.as_str()),
                self.user_id.clone(),
            )
            .set(TokenType::Access.get_key(ajti.as_str()), claims.jti.clone())
            .query(&mut conn)
            .map_err(|err| TokenError::Other(err.to_string()))?;

        Ok(TokenResponse::Refresh {
            token,
            rjti: claims.rjti,
            ajti,
        })
    }
}

impl Refresh {
    fn delete(&self, jti: String, state: AppState) -> Result<(), TokenError> {
        let mut conn = state
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
