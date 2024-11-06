use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) struct Claims {
    pub sub: String,
    pub jti: String,
    pub exp: u64,
    pub iat: u64,
    pub nbf: u64,
}

impl Claims {
    pub fn new(sub: String, exp: u64) -> Self {
        let jti = ulid::Ulid::new().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            sub,
            jti,
            exp,
            iat: now,
            nbf: now,
        }
    }
}

pub(crate) struct Session {
    pub claims: Claims,
    pub email: String,
    pub name: String,
    pub photo_url: String,
}

impl Session {
    pub fn new(sub: String, exp: u64, email: String, name: String, photo_url: String) -> Self {
        Self {
            claims: Claims::new(sub, exp),
            email,
            name,
            photo_url,
        }
    }
}
