use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PrimaryClaims {
    pub sub: String,
    pub jti: String,
    pub rjti: String,
    pub exp: usize,
    pub iat: usize,
    pub nbf: usize,
}

impl PrimaryClaims {
    pub fn new(sub: String, exp: usize, jti: Option<String>, rjti: Option<String>) -> (Self) {
        let jti = jti.unwrap_or(ulid::Ulid::new().to_string());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        let rjti = rjti.unwrap_or(jti.clone());

        Self {
            sub,
            jti,
            rjti,
            exp,
            iat: now,
            nbf: now,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ExtendedClaims {
    #[serde(flatten)]
    pub primary: PrimaryClaims,
    pub email: String,
    pub name: String,
    pub photo_url: String,
}

impl ExtendedClaims {
    pub fn new(sub: String, exp: usize, email: String, name: String, photo_url: String) -> Self {
        let claims = PrimaryClaims::new(sub, exp, None, None);
        Self {
            primary: claims,
            email,
            name,
            photo_url,
        }
    }
}

pub trait Claims {
    fn sub(&self) -> &str;
    fn jti(&self) -> &str;
    fn rjti(&self) -> &str;
    fn exp(&self) -> usize;
    fn iat(&self) -> usize;
    fn nbf(&self) -> usize;
}

impl Claims for PrimaryClaims {
    fn sub(&self) -> &str {
        &self.sub
    }

    fn jti(&self) -> &str {
        &self.jti
    }

    fn rjti(&self) -> &str {
        &self.rjti
    }

    fn exp(&self) -> usize {
        self.exp
    }

    fn iat(&self) -> usize {
        self.iat
    }

    fn nbf(&self) -> usize {
        self.nbf
    }
}

impl Claims for ExtendedClaims {
    fn sub(&self) -> &str {
        &self.primary.sub
    }

    fn jti(&self) -> &str {
        &self.primary.jti
    }

    fn rjti(&self) -> &str {
        &self.primary.rjti
    }

    fn exp(&self) -> usize {
        self.primary.exp
    }

    fn iat(&self) -> usize {
        self.primary.iat
    }

    fn nbf(&self) -> usize {
        self.primary.nbf
    }
}
