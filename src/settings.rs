use serde::{Deserialize, Serialize};
use std::fs;

const FILE_PATH: &str = "settings.toml";

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AppSettings {
    #[serde(default = "default_true")]
    pub(crate) key_processing_enabled: bool,
    #[serde(default = "default_true")]
    pub(crate) silent_key_processing: bool,
}

impl AppSettings {
    pub(crate) fn load() -> Result<Self, String> {
        let text = fs::read_to_string(&FILE_PATH)
            .map_err(|e| format!("Unable to read {} file.\n{}", FILE_PATH, e))?;
        toml::from_str(&text).map_err(|e| format!("Unable to parse {}.\n{}", FILE_PATH, e))
    }

    pub(crate) fn save(&self) -> Result<(), String> {
        let text = toml::to_string_pretty(self)
            .map_err(|e| format!("Unable to serialize {}.\n{}", FILE_PATH, e))?;
        fs::write(FILE_PATH, text)
            .map_err(|e| format!("Unable to write {} file.\n{}", FILE_PATH, e))
    }
}

fn default_true() -> bool {
    true
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
