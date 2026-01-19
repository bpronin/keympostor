use crate::kb_light::KeyboardZoneColors;
use crate::profile::Profiles;
use keympostor::error::KeyError;
use keympostor::key_err;
use log::{warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

const SETTINGS_FILE: &str = "settings.toml";
pub const LAYOUTS_PATH: &str = "layouts";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct AppSettings {
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
    pub(crate) fn load(path: &str) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_else(|| {
                warn!("Failed to load {path}. Using default values");
                Self::default()
            })
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

#[derive(Debug, Default, PartialEq, Deserialize)]
pub(crate) struct KeyboardLightingSettings {
    pub(crate) layouts: HashMap<String, KeyboardLightingLangSettings>,
}

impl KeyboardLightingSettings {
    pub(crate) fn load() -> Self {
        fs::read_to_string("kb_lighting.toml")
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_else(|| {
                warn!("Failed to load kb_lighting.toml. Using default values");
                Self::default()
            })
    }
}

#[derive(Debug, Default, PartialEq, Deserialize)]
pub(crate) struct KeyboardLightingLangSettings(pub HashMap<String, KeyboardZoneColors>);

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::profile::Profile;
    use keympostor::map;

    #[test]
    fn test_save_load_settings() {
        let settings = AppSettings {
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
            // keyboard_lighting_colors: Some(KeyboardLightingSettings(map![
            //     "default".to_string() => KeyboardLightingLangSettings(map![
            //         "en_en".to_string() => KeyboardZoneColors{right: 1, center: 2,left: 3, game: 4,},
            //         "ru_ru".to_string() => KeyboardZoneColors{right: 1, center: 2,left: 3, game: 4,},
            //     ]),
            //     "game".to_string() => KeyboardLightingLangSettings(map![
            //         "en_en".to_string() => KeyboardZoneColors{right: 10, center: 20,left: 30, game: 40,},
            //         "ru_ru".to_string() => KeyboardZoneColors{right: 10, center: 20,left: 30, game: 40,},
            //     ]),
            // ])),
        };

        assert!(settings.save("test_settings.toml").is_ok());
        let loaded = AppSettings::load("test_settings.toml");
        assert_eq!(settings, loaded);
        assert_eq!(AppSettings::default(), AppSettings::load(""));
    }
}
