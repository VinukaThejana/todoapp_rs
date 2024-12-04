use std::borrow::Cow;
use validator::ValidationError;

pub fn database_url(database_url: &str) -> Result<(), ValidationError> {
    if database_url.is_empty() {
        return Err(ValidationError::new("database_url")
            .with_message(Cow::Owned(String::from("Database URL must be provided"))));
    }
    if !database_url.starts_with("postgresql://") {
        return Err(
            ValidationError::new("database_url").with_message(Cow::Owned(String::from(
                "Please provide a valid database URL",
            ))),
        );
    }

    Ok(())
}

pub fn log_level(level: &str) -> Result<(), ValidationError> {
    let levels = Vec::from(["trace", "debug", "info", "warn", "error"]);

    if level.is_empty() {
        return Err(ValidationError::new("rust_log")
            .with_message(Cow::Owned(String::from("Log level must be provided"))));
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
            .with_message(Cow::Owned(String::from("Redis URL must be provided"))));
    }

    if url.starts_with("redis://") {
        return Err(
            ValidationError::new("redis_url").with_message(Cow::Owned(String::from(
                "Please provide a secure redis URL",
            ))),
        );
    }

    if !url.starts_with("rediss://") {
        return Err(ValidationError::new("redis_url")
            .with_message(Cow::Owned(String::from("Please provide a valid redis URL"))));
    }

    Ok(())
}
