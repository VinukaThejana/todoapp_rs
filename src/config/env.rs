use crate::utils::{
    utils::{deserialize_arc_str, deserialize_base64},
    verify,
};
use dotenvy::dotenv;
use envy;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvMode {
    Prd,
    Dev,
}

impl From<&Arc<str>> for EnvMode {
    fn from(mode: &Arc<str>) -> Self {
        match mode.as_ref() {
            "prd" => Self::Prd,
            "dev" => Self::Dev,
            _ => unreachable!("Invalid environment mode"),
        }
    }
}

impl EnvMode {
    pub fn is_prd(mode: &str) -> bool {
        mode == "prd"
    }
    pub fn is_dev(mode: &str) -> bool {
        mode == "dev"
    }

    pub fn is_valid(mode: &str) -> bool {
        Self::is_prd(mode) || Self::is_dev(mode)
    }
}

#[derive(Debug, Validate, Deserialize)]
pub struct Env {
    #[validate(custom(function = "verify::database_url"))]
    #[serde(deserialize_with = "deserialize_arc_str")]
    pub database_url: Arc<str>,

    #[validate(custom(function = "verify::log_level"))]
    #[serde(deserialize_with = "deserialize_arc_str")]
    pub rust_log: Arc<str>,

    #[validate(custom(function = "verify::redis_url"))]
    #[serde(deserialize_with = "deserialize_arc_str")]
    pub redis_url: Arc<str>,

    #[validate(length(min = 1, message = "domain is required and cannot be empty"))]
    #[serde(deserialize_with = "deserialize_arc_str")]
    pub domain: Arc<str>,

    #[validate(custom(function = "verify::env_mode"))]
    #[serde(deserialize_with = "deserialize_arc_str")]
    pub env: Arc<str>,

    #[validate(length(
        min = 1,
        message = "refresh token private key is required and cannot be empty"
    ))]
    #[serde(deserialize_with = "deserialize_base64")]
    pub refresh_token_private_key: Arc<Vec<u8>>,

    #[validate(length(
        min = 1,
        message = "refresh token public key is required and cannot be empty"
    ))]
    #[serde(deserialize_with = "deserialize_base64")]
    pub refresh_token_public_key: Arc<Vec<u8>>,

    #[validate(length(
        min = 1,
        message = "access token private key is required and cannot be empty"
    ))]
    #[serde(deserialize_with = "deserialize_base64")]
    pub access_token_private_key: Arc<Vec<u8>>,

    #[validate(length(
        min = 1,
        message = "access token public key is required and cannot be empty"
    ))]
    #[serde(deserialize_with = "deserialize_base64")]
    pub access_token_public_key: Arc<Vec<u8>>,

    #[validate(length(
        min = 1,
        message = "session token private key is required and cannot be empty"
    ))]
    #[serde(deserialize_with = "deserialize_base64")]
    pub session_token_private_key: Arc<Vec<u8>>,

    #[validate(length(
        min = 1,
        message = "session token public key is required and cannot be empty"
    ))]
    #[serde(deserialize_with = "deserialize_base64")]
    pub session_token_public_key: Arc<Vec<u8>>,

    #[validate(length(
        min = 1,
        message = "reauth token private key is required and cannot be empty"
    ))]
    #[serde(deserialize_with = "deserialize_base64")]
    pub reauth_token_private_key: Arc<Vec<u8>>,

    #[validate(length(
        min = 1,
        message = "reauth token public key is required and cannot be empty"
    ))]
    #[serde(deserialize_with = "deserialize_base64")]
    pub reauth_token_public_key: Arc<Vec<u8>>,

    #[validate(range(
        min = 172_800,
        message = "refresh token expiration must be greater than 172,800 seconds (2 Days)"
    ))]
    pub refresh_token_expiration: usize,

    #[validate(range(
        max = 86_400,
        message = "access token expiration must be smaller than 86,400 seconds (1 Day)"
    ))]
    pub access_token_expiration: usize,

    #[validate(range(
        min = 172_800,
        message = "session token expiration must be greater than 172,800 seconds (2 Days)"
    ))]
    pub session_token_expiration: usize,

    #[validate(range(
        min = 1,
        message = "reauth token expiration must be greater than 1 second"
    ))]
    pub reauth_token_expiration: usize,

    #[validate(range(
        min = 8080,
        max = 8090,
        message = "please provide a valid port number between 8080 and 8090 (inclusive)"
    ))]
    pub port: u16,
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    pub fn new() -> Self {
        dotenv().expect("Failed to load the .env file");

        let env: Self = envy::from_env().unwrap_or_else(|_| {
            eprintln!("Failed to load the environment variables, exiting ... ");
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
                                .unwrap_or_else(|| String::from("invalid input"))
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

            eprintln!("\nValidation errors");
            eprintln!("{}", message);
            eprintln!("\nUpdate the .env file to resolve the above errors and try again");
            std::process::exit(1);
        });

        env_logger::init();
        env
    }
}

pub static ENV: Lazy<Env> = Lazy::new(Env::new);
