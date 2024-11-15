use base64::prelude::*;
use serde::{Deserialize, Deserializer};
use std::sync::Arc;

pub fn deserialize_base64<'de, D>(deserializer: D) -> Result<Arc<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    let bytes = BASE64_STANDARD
        .decode(s.as_bytes())
        .map_err(serde::de::Error::custom)?;

    Ok(Arc::new(bytes))
}

pub fn deserialize_arc_str<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    Ok(s.into())
}
