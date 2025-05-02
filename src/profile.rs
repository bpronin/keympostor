use crate::key_action::{KeyAction, KeyActionSequence};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{env, fs};

const FILE_PATH: &str = "profiles/default.json";

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Profile {
    pub(crate) name: String,
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) transform_rules: Vec<TransformRule>,
}

impl Profile {
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
pub(crate) struct TransformRule {
    pub(crate) source: KeyAction,
    pub(crate) target: KeyActionSequence,
}

#[cfg(test)]
mod tests {
    use crate::profile::Profile;

    #[test]
    fn test_load_config() {
        let config = Profile::load().unwrap();
        dbg!(&config);
    }
}
