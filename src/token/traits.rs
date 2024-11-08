use super::{claims, error::TokenError, TokenType};
use crate::config::state::AppState;
use base64::prelude::*;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use redis::Commands;
use serde::{Deserialize, Serialize};

pub enum TokenResponse {
    Access(String),
    Refresh {
        token: String,
        rjti: String,
        ajti: String,
    },
    Session(String),
}

pub trait Token<T>
where
    T: for<'a> Serialize + for<'a> Deserialize<'a> + claims::HasClaims,
{
    fn secret(&self, secret: String) -> Result<String, TokenError> {
        let secret = BASE64_STANDARD
            .decode(secret)
            .map_err(|err| TokenError::Parsing(err.to_string()))?;
        let secret =
            String::from_utf8(secret).map_err(|err| TokenError::Parsing(err.to_string()))?;

        Ok(secret)
    }

    fn generate(&self, claims: T, secret: String) -> Result<String, TokenError> {
        let secret = self.secret(secret)?;

        jsonwebtoken::encode(
            &Header::new(Algorithm::RS256),
            &claims,
            &EncodingKey::from_rsa_pem(secret.as_bytes())
                .map_err(|err| TokenError::Creation(err.to_string()))?,
        )
        .map_err(|err| TokenError::Creation(err.to_string()))
    }

    fn create(&self, state: AppState) -> Result<TokenResponse, TokenError>;

    fn decode(&self, token: String, secret: String) -> Result<T, TokenError> {
        let secret = self.secret(secret)?;

        let claims = jsonwebtoken::decode::<T>(
            &token,
            &DecodingKey::from_rsa_pem(secret.as_bytes())
                .map_err(|err| TokenError::Parsing(err.to_string()))?,
            &Validation::new(Algorithm::RS256),
        )
        .map_err(|err| TokenError::Validation(err.to_string()))?
        .claims;

        Ok(claims)
    }

    fn verify(
        &self,
        state: AppState,
        token: String,
        secret: String,
        token_type: TokenType,
    ) -> Result<T, TokenError> {
        let claims = self.decode(token, secret)?;

        let mut redis = state
            .rd
            .get_connection()
            .map_err(|err| TokenError::Other(err.to_string()))?;

        let value: Option<String> = redis
            .get(token_type.get_key(claims.get_jti()))
            .map_err(|err| TokenError::Other(err.to_string()))?;
        let value =
            value.ok_or_else(|| TokenError::Validation("token not found in redis".to_string()))?;

        match token_type {
            TokenType::Access => {
                if value != claims.get_rjti() {
                    return Err(TokenError::Validation(
                        "access token is not valid".to_string(),
                    ));
                }
            }
            TokenType::Refresh => {
                if value != claims.get_sub() {
                    return Err(TokenError::Validation(
                        "refresh token is not valid".to_string(),
                    ));
                }
            }
            _ => panic!("please provide a custom implementation for the session token"),
        }

        Ok(claims)
    }
}
