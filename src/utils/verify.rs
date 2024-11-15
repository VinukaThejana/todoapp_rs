use std::borrow::Cow;
use validator::ValidationError;

pub fn database_url(database_url: &str) -> Result<(), ValidationError> {
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

pub fn log_level(level: &str) -> Result<(), ValidationError> {
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

pub fn redis_url(url: &str) -> Result<(), ValidationError> {
    if url.is_empty() {
        return Err(ValidationError::new("redis_url")
            .with_message(Cow::Owned("Redis URL must be provided".to_string())));
    }

    if url.starts_with("redis://") {
        return Err(ValidationError::new("redis_url")
            .with_message(Cow::Owned("Redis URL should be TLS encrypted".to_string())));
    }

    if !url.starts_with("rediss://") {
        return Err(ValidationError::new("redis_url")
            .with_message(Cow::Owned("Please provide a valid redis URL".to_string())));
    }

    Ok(())
}
