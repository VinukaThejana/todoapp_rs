use dotenvy::dotenv;
use envy;
use once_cell::sync::Lazy;
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Debug, Validate, Deserialize)]
pub struct Env {
    #[validate(custom(function = "validate_database_url"))]
    pub database_url: String,

    #[validate(range(min = 8080, max = 8090))]
    pub port: u16,
}

impl Env {
    pub fn new() -> Self {
        dotenv().expect("Failed to load the .env file");

        let env: Self = envy::from_env()
            .expect("Failed to parse the envrionment variables, please check the .env file");
        env.validate().unwrap();

        env
    }
}

pub static ENV: Lazy<Env> = Lazy::new(Env::new);

fn validate_database_url(database_url: &str) -> Result<(), ValidationError> {
    if database_url.is_empty() {
        return Err(ValidationError::new("Database URL cannot be empty"));
    }
    if !database_url.starts_with("postgresql://") {
        return Err(ValidationError::new(
            "Database url must be a valid postgresql URL",
        ));
    }

    Ok(())
}
