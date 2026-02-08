use crate::profile::LayoutAutoswitchProfile;
use keympostor::key_trigger;
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
    pub(crate) last_transform_layout: Option<String>,
    pub(crate) toggle_layout_hot_key: Option<KeyTrigger>,
    pub(crate) layout_autoswitch: Option<LayoutAutoSwitchSettings>,
    pub(crate) main_window: MainWindowSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            keys_logging_enabled: false,
            toggle_layout_hot_key: Some(key_trigger!("[]FN_LAUNCH_APP2^")),
            last_transform_layout: Default::default(),
            layout_autoswitch: Default::default(),
            main_window: Default::default(),
        }
    }
}

impl AppSettings {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let text = fs::read_to_string(path)?;
        let this = toml::from_str(&text)?;
        Ok(this)
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let text = toml::to_string(self)?;
        fs::write(path, text)?;
        Ok(())
    }

    pub(crate) fn load_default() -> Result<Self, Box<dyn Error>> {
        Self::load(SETTINGS_FILE)
    }

    pub(crate) fn save_default(&self) {
        self.save(SETTINGS_FILE).expect("Failed to save settings");
        debug!("Settings saved");
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct LayoutAutoSwitchSettings {
    pub(crate) enabled: bool,
    pub(crate) profiles: Option<HashMap<String, LayoutAutoswitchProfile>>,
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
    use crate::profile::LayoutAutoswitchProfile;
    use crate::{map, str};

    #[test]
    fn test_save_load_settings() {
        let settings = AppSettings {
            keys_logging_enabled: false,
            toggle_layout_hot_key: None,
            last_transform_layout: Some(str!("test-layout")),
            main_window: MainWindowSettings {
                position: Some((0, 0)),
                size: Some((100, 200)),
                selected_page: Some(0),
                log_view: Default::default(),
            },
            layout_autoswitch: Some(LayoutAutoSwitchSettings {
                enabled: true,
                profiles: Some(map![
                    str!("chrome") => LayoutAutoswitchProfile {
                        activation_rule: Some(str!("Chrome")),
                        layout: str!("desktop"),
                    },
                    str!("tc") => LayoutAutoswitchProfile {
                        activation_rule: Some(str!("TOTALCMD64.EXE")),
                        layout: str!("game"),
                    },
                ])
            }),
        };

        const PATH: &'static str = "etc/test_data/test_settings.toml";

        assert!(settings.save(PATH).is_ok());

        let loaded = AppSettings::load(PATH).unwrap();
        assert_eq!(settings, loaded);
    }
}
