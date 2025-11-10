use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

const FILE_PATH: &str = "settings.toml";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AppSettings {
    pub(crate) key_processing_enabled: bool,
    pub(crate) silent_key_processing: bool,
    pub(crate) auto_switch_profile_enabled: bool,
    pub(crate) main_window_position: Option<(i32, i32)>,
    pub(crate) main_window_size: Option<(u32, u32)>,
    pub(crate) main_window_selected_page: Option<usize>,
    pub(crate) profile: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            key_processing_enabled: true,
            auto_switch_profile_enabled: false,
            silent_key_processing: false,
            main_window_position: None,
            main_window_size: None,
            main_window_selected_page: None,
            profile: None,
        }
    }
}

impl AppSettings {
    pub(crate) fn load() -> Self {
        fs::read_to_string(&FILE_PATH)
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub(crate) fn save(&self) -> Result<()> {
        let text = toml::to_string(self) /* dont want `to_string_pretty` */
            .context(format!("Error serializing `{}`", FILE_PATH))?;
        fs::write(FILE_PATH, text).context(format!("Error writing `{}`", FILE_PATH))
    }
}
