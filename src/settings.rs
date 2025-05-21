use serde::{Deserialize, Serialize};
use std::fs;

const FILE_PATH: &str = "settings.toml";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AppSettings {
    pub(crate) key_processing_enabled: bool,
    pub(crate) silent_key_processing: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            key_processing_enabled: true,
            silent_key_processing: false,
        }
    }
}

impl AppSettings {
    pub(crate) fn load() -> Result<Self, String> {
        let text = fs::read_to_string(&FILE_PATH)
            .map_err(|e| format!("Error reading `{}`. {}", FILE_PATH, e))?;
        toml::from_str(&text).map_err(|e| format!("Error parsing `{}`. {}", FILE_PATH, e))
    }

    pub(crate) fn save(&self) -> Result<(), String> {
        let text = toml::to_string_pretty(self)
            .map_err(|e| format!("Error serializing `{}`. {}", FILE_PATH, e))?;
        fs::write(FILE_PATH, text).map_err(|e| format!("Error writing `{}`. {}", FILE_PATH, e))
    }
}

#[cfg(test)]
mod tests {
    use crate::settings::AppSettings;

    #[test]
    fn test_load_save() {
        let settings = AppSettings::load();
        assert!(settings.is_ok());
        assert!(settings.unwrap().save().is_ok());
    }
}
