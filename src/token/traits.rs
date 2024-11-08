use base64::prelude::*;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use redis::Commands;
use serde::{Deserialize, Serialize};

use crate::config::state::AppState;
use crate::token::claims;
use crate::token::error::TokenError;
use crate::token::types::{params::TokenParams, response::TokenResponse};
use crate::token::TokenType;

pub trait Token<T>
where
    T: for<'a> Serialize + for<'a> Deserialize<'a> + claims::HasClaims,
{
    fn state(&self) -> AppState;
    fn public_key(&self) -> String;
    fn private_key(&self) -> String;

    fn secret(&self, secret: String) -> Result<Vec<u8>, TokenError> {
        let secret = BASE64_STANDARD
            .decode(secret)
            .map_err(|err| TokenError::Parsing(err.to_string()))?;
        Ok(secret)
    }

    fn encode_rsa_key_pem(&self, key: String) -> Result<EncodingKey, TokenError> {
        EncodingKey::from_rsa_pem(&self.secret(key)?)
            .map_err(|err| TokenError::Parsing(err.to_string()))
    }
    fn decode_rsa_key_pem(&self, key: String) -> Result<DecodingKey, TokenError> {
        DecodingKey::from_rsa_pem(&self.secret(key)?)
            .map_err(|err| TokenError::Parsing(err.to_string()))
    }

    fn generate(&self, claims: T) -> Result<String, TokenError> {
        jsonwebtoken::encode(
            &Header::new(Algorithm::RS256),
            &claims,
            &self.encode_rsa_key_pem(self.private_key())?,
        )
        .map_err(|err| TokenError::Creation(err.to_string()))
    }
    fn create(&self, params: TokenParams) -> Result<TokenResponse, TokenError>;
    fn decode(&self, token: String) -> Result<T, TokenError> {
        let claims = jsonwebtoken::decode::<T>(
            &token,
            &self.decode_rsa_key_pem(self.public_key())?,
            &Validation::new(Algorithm::RS256),
        )
        .map_err(|err| TokenError::Validation(err.to_string()))?
        .claims;
        Ok(claims)
    }

    fn verify(&self, token: String, token_type: TokenType) -> Result<T, TokenError> {
        let claims = self.decode(token)?;
        let mut redis = self
            .state()
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
            _ => unimplemented!("use the decode function to decode the session token"),
        }
        Ok(claims)
    }
}
