use std::collections::HashMap;
use crate::profile::Profiles;
use keympostor::key_err;
use keympostor::keyboard::error::KeyError;
use serde::{Deserialize, Serialize};
use std::fs;

const SETTINGS_FILE: &str = "settings.toml";
pub const LAYOUTS_PATH: &str = "layouts";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct AppSettings {
    pub(crate) processing_enabled: bool, //todo: rename to `transform_enabled` ?
    pub(crate) logging_enabled: bool,
    pub(crate) layouts_enabled: bool,
    pub(crate) main_window: MainWindowSettings,
    pub(crate) log_view: LogViewSettings,
    pub(crate) layout: Option<String>,
    pub(crate) profiles: Option<Profiles>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            processing_enabled: true,
            logging_enabled: false,
            layouts_enabled: false,
            main_window: Default::default(),
            log_view: Default::default(),
            layout: None,
            profiles: Default::default(),
        }
    }
}

impl AppSettings {
    pub(crate) fn load(filename: &str) -> Self {
        fs::read_to_string(filename)
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub(crate) fn save(&self, path: &str) -> Result<(), KeyError> {
        fs::write(
            path,
            toml::to_string(self) /* dont want `to_string_pretty` */
                .or(key_err!("Error serializing `{path}`"))?,
        )
        .or(key_err!("Error writing `{path}`"))
    }

    pub(crate) fn load_default() -> Self {
        Self::load(SETTINGS_FILE)
    }

    pub(crate) fn save_default(&self) -> Result<(), KeyError> {
        self.save(SETTINGS_FILE)
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct MainWindowSettings {
    pub(crate) position: Option<(i32, i32)>,
    pub(crate) size: Option<(u32, u32)>,
    pub(crate) selected_page: Option<usize>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct LogViewSettings {
    pub(crate) columns: Option<HashMap<usize, isize>>,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::profile::Profile;
    use keympostor::map;

    #[test]
    fn test_save_load_settings() {
        let settings = AppSettings {
            processing_enabled: false,
            logging_enabled: false,
            layouts_enabled: true,
            layout: Some("test-layout".to_string()),
            main_window: MainWindowSettings {
                position: Some((0, 0)),
                size: Some((100, 200)),
                selected_page: Some(0),
            },
            profiles: Some(Profiles(map![
                "chrome".to_string() => Profile {
                    rule: Some("Chrome".to_string()),
                    layout: Some("game".to_string()),
                },
                "tc".to_string() => Profile {
                    rule: Some("TOTALCMD64.EXE".to_string()),
                    layout: Some("game".to_string()),
                },
            ])),
            log_view: Default::default(),
        };

        assert!(settings.save("test_settings.toml").is_ok());
        let loaded = AppSettings::load("test_settings.toml");
        assert_eq!(settings, loaded);
        assert_eq!(AppSettings::default(), AppSettings::load(""));
    }
}
