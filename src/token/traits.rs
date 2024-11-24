use super::{
    claims,
    error::TokenError,
    types::{params::TokenParams, response::TokenResponse},
    TokenType,
};
use crate::config::state::AppState;
use anyhow::anyhow;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::future::Future;

pub trait Token<T>
where
    T: for<'a> Serialize,
    T: for<'a> Deserialize<'a>,
    T: claims::Claims,
    T: Send + Sync,
    Self: Send + Sync,
{
    fn state(&self) -> AppState;
    fn public_key(&self) -> &[u8];
    fn private_key(&self) -> &[u8];
    fn exp(&self) -> usize;

    fn encode_rsa_key_pem(&self, key: &[u8]) -> EncodingKey {
        EncodingKey::from_rsa_pem(key).unwrap_or_else(|err| {
            log::error!("Error encoding RSA key: {}", err);
            std::process::exit(1);
        })
    }
    fn decode_rsa_key_pem(&self, key: &[u8]) -> DecodingKey {
        DecodingKey::from_rsa_pem(key).unwrap_or_else(|err| {
            log::error!("Error decoding RSA key: {}", err);
            std::process::exit(1);
        })
    }

    fn generate(&self, claims: &T) -> Result<String, TokenError> {
        jsonwebtoken::encode(
            &Header::new(Algorithm::RS256),
            claims,
            &self.encode_rsa_key_pem(self.private_key()),
        )
        .map_err(|err| TokenError::Creation(err.into()))
    }

    fn create(
        &self,
        params: TokenParams,
    ) -> impl Future<Output = Result<TokenResponse, TokenError>> + Send;

    fn decode(&self, token: String) -> Result<T, TokenError> {
        let claims = jsonwebtoken::decode::<T>(
            &token,
            &self.decode_rsa_key_pem(self.public_key()),
            &Validation::new(Algorithm::RS256),
        )
        .map_err(|err| TokenError::Validation(err.into()))?
        .claims;

        Ok(claims)
    }

    fn verify(
        &self,
        token: String,
        token_type: TokenType,
    ) -> impl Future<Output = Result<T, TokenError>> + Send
    where
        T: Send,
    {
        async move {
            let claims = self.decode(token)?;

            let mut conn = self
                .state()
                .get_redis_conn()
                .await
                .map_err(TokenError::Other)?;

            let value: Option<String> = redis::cmd("GET")
                .arg(token_type.get_key(claims.jti()))
                .query_async(&mut conn)
                .await
                .map_err(|err| TokenError::Other(err.into()))?;
            let value =
                value.ok_or_else(|| TokenError::Validation(anyhow!("token not found in redis")))?;

            match token_type {
                TokenType::Access => {
                    if value != claims.sub() {
                        return Err(TokenError::Validation(anyhow!("access token is not valid")));
                    }
                }
                TokenType::Refresh => {
                    if value.is_empty() {
                        return Err(TokenError::Validation(anyhow!(
                            "refresh token is not valid"
                        )));
                    }
                }
                TokenType::Session => {
                    unimplemented!("please implement the session token verification logic")
                }
                TokenType::ReAuth => {
                    unimplemented!("please implement the re-auth token verification logic")
                }
            }

            Ok(claims)
        }
    }
}
