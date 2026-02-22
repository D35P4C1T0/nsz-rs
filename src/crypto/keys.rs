use std::collections::HashMap;

use crate::error::NszError;

const REQUIRED_KEYS: &[&str] = &[
    "aes_kek_generation_source",
    "aes_key_generation_source",
    "titlekek_source",
    "key_area_key_application_source",
    "key_area_key_ocean_source",
    "key_area_key_system_source",
];

#[derive(Debug, Clone)]
pub struct LoadedKeys {
    pub values: HashMap<String, String>,
}

pub fn load_from_str(content: &str) -> Result<LoadedKeys, NszError> {
    let mut values = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            values.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    for key in REQUIRED_KEYS {
        if !values.contains_key(*key) {
            return Err(NszError::MissingRequiredKey {
                key: (*key).to_string(),
            });
        }
    }

    Ok(LoadedKeys { values })
}
