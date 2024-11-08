use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub fn new(
        sub: String,
        exp: usize,
        jti: Option<String>,
        rjti: Option<String>,
    ) -> (Self, String) {
        let jti = jti.unwrap_or(ulid::Ulid::new().to_string());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        let rjti = rjti.unwrap_or(jti.clone());

        (
            Self {
                sub,
                jti: jti.clone(),
                rjti,
                exp,
                iat: now,
                nbf: now,
            },
            jti,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ExtendedClaims {
    pub sub: String,
    pub jti: String,
    pub exp: usize,
    pub iat: usize,
    pub nbf: usize,
    pub email: String,
    pub name: String,
    pub photo_url: String,
}

impl ExtendedClaims {
    pub fn new(sub: String, exp: usize, email: String, name: String, photo_url: String) -> Self {
        let (claims, _) = PrimaryClaims::new(sub, exp, None, None);

        Self {
            email,
            name,
            photo_url,
            sub: claims.sub,
            jti: claims.jti,
            exp: claims.exp,
            iat: claims.iat,
            nbf: claims.nbf,
        }
    }
}

pub trait HasClaims {
    fn get_sub(&self) -> &str;
    fn get_jti(&self) -> &str;
    fn get_rjti(&self) -> &str;
    fn get_exp(&self) -> usize;
    fn get_iat(&self) -> usize;
    fn get_nbf(&self) -> usize;
}

impl HasClaims for PrimaryClaims {
    fn get_sub(&self) -> &str {
        &self.sub
    }

    fn get_jti(&self) -> &str {
        &self.jti
    }

    fn get_rjti(&self) -> &str {
        &self.rjti
    }

    fn get_exp(&self) -> usize {
        self.exp
    }

    fn get_iat(&self) -> usize {
        self.iat
    }

    fn get_nbf(&self) -> usize {
        self.nbf
    }
}

impl HasClaims for ExtendedClaims {
    fn get_sub(&self) -> &str {
        &self.sub
    }

    fn get_jti(&self) -> &str {
        &self.jti
    }

    fn get_rjti(&self) -> &str {
        &self.jti
    }

    fn get_exp(&self) -> usize {
        self.exp
    }

    fn get_iat(&self) -> usize {
        self.iat
    }

    fn get_nbf(&self) -> usize {
        self.nbf
    }
}
