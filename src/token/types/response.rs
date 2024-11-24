use std::fmt::Display;

#[derive(Debug)]
pub enum TokenResponse {
    Access(String),
    Refresh {
        token: String,
        rjti: String,
        ajti: String,
    },
    Session(String),
    Reauth(String),
}

impl Display for TokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenResponse::Access(token) => write!(f, "{token}"),
            TokenResponse::Refresh {
                token,
                rjti: _,
                ajti: _,
            } => write!(f, "{token}"),
            TokenResponse::Session(token) => write!(f, "{token}"),
            TokenResponse::Reauth(token) => write!(f, "{token}"),
        }
    }
}
