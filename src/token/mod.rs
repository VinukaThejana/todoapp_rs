use std::fmt::Display;

pub mod claims;
pub mod error;
pub mod traits;
pub mod types;

pub enum TokenType {
    Access,
    Refresh,
    Session,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token: &str = match *self {
            TokenType::Access => "access_token",
            TokenType::Refresh => "refresh_token",
            TokenType::Session => "session_token",
        };

        write!(f, "{token}")
    }
}

impl TokenType {
    pub fn get_key(&self, jti: &str) -> String {
        format!("{}:{}", self, jti)
    }
}