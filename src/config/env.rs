use std::borrow::Cow;

use dotenvy::dotenv;
use envy;
use once_cell::sync::Lazy;
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Debug, Validate, Deserialize)]
pub struct Env {
    #[validate(custom(function = "validate_database_url"))]
    pub database_url: String,

    #[validate(custom(function = "validate_log_level"))]
    pub rust_log: String,

    #[validate(range(
        min = 8080,
        max = 8090,
        message = "please provide a valid port number between 8080 and 8090 (inclusive)"
    ))]
    pub port: u16,
}

impl Env {
    pub fn new() -> Self {
        dotenv().expect("Failed to load the .env file");

        let env: Self = envy::from_env().unwrap_or_else(|_| {
            println!("Failed to load the environment variables, exiting ... ");
            std::process::exit(1);
        });
        env.validate().unwrap_or_else(|e| {
            let message = e
                .field_errors()
                .iter()
                .fold(String::new(), |acc, (field, errs)| {
                    let field_errs = errs
                        .iter()
                        .map(|err| {
                            err.message
                                .as_ref()
                                .map(|msg| msg.to_string())
                                .unwrap_or_else(|| "invalid input".to_string())
                        })
                        .collect::<Vec<String>>()
                        .join(", ");

                    if field_errs.is_empty() {
                        return acc;
                    }

                    if acc.is_empty() {
                        format!("\n{}: {}", field, field_errs)
                    } else {
                        format!("{}\n{}: {}", acc, field, field_errs)
                    }
                });

            println!("\nValidation errors");
            println!("{}", message);
            println!("\nUpdate the .env file to resolve the above errors and try again");
            std::process::exit(1);
        });
        env_logger::init();

        env
    }
}

pub static ENV: Lazy<Env> = Lazy::new(Env::new);

fn validate_database_url(database_url: &str) -> Result<(), ValidationError> {
    if database_url.is_empty() {
        return Err(ValidationError::new("database_url")
            .with_message(Cow::Owned("Database url must be provided".to_string())));
    }
    if !database_url.starts_with("postgresql://") {
        return Err(
            ValidationError::new("database_url").with_message(Cow::Owned(
                "Please provide a valid postgresql database url".to_string(),
            )),
        );
    }

    Ok(())
}

fn validate_log_level(level: &str) -> Result<(), ValidationError> {
    let levels = Vec::from(["trace", "debug", "info", "warn", "error"]);

    if level.is_empty() {
        return Err(ValidationError::new("rust_log")
            .with_message(Cow::Owned("Log level must be provided".to_string())));
    }

    if !levels.contains(&level.to_lowercase().as_str()) {
        let message = levels.iter().fold(String::new(), |acc, level| {
            let level = level.to_uppercase();
            if acc.is_empty() {
                format!("Please provide a valid log level: {}", level)
            } else {
                format!("{}, {}", acc, level)
            }
        });

        return Err(ValidationError::new("rust_log").with_message(Cow::Owned(message)));
    }

    Ok(())
}
