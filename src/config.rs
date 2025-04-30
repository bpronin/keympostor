use crate::key_action::{KeyAction, KeyActionSequence};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{env, fs};
use crate::key_modifier::{KeyModifiers, KM_NONE};

const FILE_PATH: &str = "config.json";

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AppConfig {
    #[serde(default = "ConfigDefaults::app_state")]
    pub(crate) app_state: AppState,
    #[serde(default)]
    pub(crate) transform_rules: Vec<TransformRule>,
}

impl AppConfig {
    pub(crate) fn load() -> Result<Self, String> {
        let path = Self::file_path();
        
        let json = fs::read_to_string(&path)
            .map_err(|e| format!("Unable to read {} file.\n{}", path, e))?;
        let config =
            serde_json::from_str(&json).map_err(|e| format!("Unable to parse {}.\n{}", path, e))?;

        Ok(config)
    }

    pub(crate) fn save(&self) -> Result<(), String> {
        let path = Self::file_path();
        
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Unable to serialize {}.\n{}", path, e))?;
        fs::write(&path, json).map_err(|e| format!("Unable to write {} file.\n{}", path, e))?;

        Ok(())
    }

    pub(crate) fn file_path() -> String {
        let mut args = env::args();
        args.next(); /* executable name */
        args.next().unwrap_or(FILE_PATH.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AppState {
    #[serde(default = "ConfigDefaults::bool_true")]
    pub(crate) key_processing_enabled: bool,
    #[serde(default = "ConfigDefaults::bool_true")]
    pub(crate) silent_key_processing: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TransformRule {
    pub(crate) source: KeyAction,
    pub(crate) target: KeyActionSequence,
}

pub(crate) struct ConfigDefaults {}

impl ConfigDefaults {
    pub(crate) fn bool_true() -> bool {
        true
    }

    pub(crate) fn app_state() -> AppState {
        AppState {
            key_processing_enabled: true,
            silent_key_processing: true,
        }
    }

    pub(crate) fn km_none() -> KeyModifiers {
        KM_NONE
    }

}

#[cfg(test)]
mod tests {
    use crate::config::AppConfig;

    #[test]
    fn test_load_config() {
        let config = AppConfig::load().unwrap();
        dbg!(&config);
    }
}
