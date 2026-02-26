use crate::profile::Profile;
use keympostor::{key_trigger, load_from_toml_file, save_to_toml_file};
use keympostor::trigger::KeyTrigger;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::str::FromStr;

const SETTINGS_FILE: &str = "settings.toml";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct AppSettings {
    pub(crate) keys_logging_enabled: bool,
    pub(crate) layout_autoswitch_enabled: bool,
    pub(crate) toggle_layout_hot_key: Option<KeyTrigger>,
    pub(crate) main_window: MainWindowSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            keys_logging_enabled: false,
            layout_autoswitch_enabled: false,
            toggle_layout_hot_key: Some(key_trigger!("[]FN_LAUNCH_APP2^")),
            main_window: Default::default(),
        }
    }
}

impl AppSettings {
    pub(crate) fn load() -> Result<Self, Box<dyn Error>> {
        Self::load_from(SETTINGS_FILE)
    }

    pub(crate) fn save(&self) {
        self.save_to(SETTINGS_FILE)
            .expect("Failed to save settings");
        debug!("Settings saved");
    }

    load_from_toml_file!();

    save_to_toml_file!();

}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct MainWindowSettings {
    pub(crate) position: Option<(i32, i32)>,
    pub(crate) size: Option<(u32, u32)>,
    pub(crate) selected_page: Option<usize>,
    pub(crate) log_view: LogViewSettings,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct LogViewSettings {
    pub(crate) columns: Option<HashMap<usize, isize>>,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::str;

    #[test]
    fn test_save_load_settings() {
        let settings = AppSettings {
            keys_logging_enabled: false,
            layout_autoswitch_enabled: false,
            toggle_layout_hot_key: None,
            main_window: MainWindowSettings {
                position: Some((0, 0)),
                size: Some((100, 200)),
                selected_page: Some(0),
                log_view: Default::default(),
            },
        };

        let path = "etc/test_data/test_settings.toml";

        assert!(settings.save_to(path).is_ok());

        let loaded = AppSettings::load_from(path).unwrap();
        assert_eq!(settings, loaded);
    }
}
