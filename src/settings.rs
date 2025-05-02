use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;

const FILE_PATH: &str = "settings.json";

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AppSettings {
    #[serde(default = "default_true")]
    pub(crate) key_processing_enabled: bool,
    #[serde(default = "default_true")]
    pub(crate) silent_key_processing: bool,
}

impl AppSettings {
    pub(crate) fn load() -> Result<Self, String> {
        let json = fs::read_to_string(&FILE_PATH)
            .map_err(|e| format!("Unable to read {} file.\n{}", FILE_PATH, e))?;
        let config = serde_json::from_str(&json)
            .map_err(|e| format!("Unable to parse {}.\n{}", FILE_PATH, e))?;

        Ok(config)
    }

    pub(crate) fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Unable to serialize {}.\n{}", FILE_PATH, e))?;
        fs::write(FILE_PATH, json)
            .map_err(|e| format!("Unable to write {} file.\n{}", FILE_PATH, e))?;

        Ok(())
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use crate::settings::AppSettings;

    #[test]
    fn test_load() {
        let settings = AppSettings::load().unwrap();
        dbg!(&settings);
    }
}
