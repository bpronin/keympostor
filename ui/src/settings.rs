use crate::kb_light::KeyboardZoneColors;
use crate::profile::Profiles;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

const SETTINGS_FILE: &str = "settings.toml";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct AppSettings {
    pub(crate) logging_enabled: bool,
    pub(crate) profiles_enabled: bool,
    pub(crate) main_window: MainWindowSettings,
    pub(crate) transform_layout: Option<String>,
    pub(crate) profiles: Option<Profiles>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            logging_enabled: false,
            profiles_enabled: false,
            main_window: Default::default(),
            transform_layout: None,
            profiles: Default::default(),
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

    pub(crate) fn load_default() -> Self {
        Self::load(SETTINGS_FILE)
            .and_then(|settings| {
                debug!("Settings loaded");
                Ok(settings)
            })
            .unwrap_or_else(|e| {
                warn!("Failed to load settings: `{}`. Using defaults.", e);
                Self::default()
            })
    }

    pub(crate) fn save_default(&self) {
        self.save(SETTINGS_FILE).expect("Failed to save settings");
        debug!("Settings saved");
    }
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

#[derive(Debug, Default, PartialEq, Deserialize)]
pub(crate) struct KeyboardLightingSettings {
    pub(crate) layouts: HashMap<String, KeyboardLightingLangSettings>,
}

impl KeyboardLightingSettings {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let text = fs::read_to_string(path)?;
        let this = toml::from_str(&text)?;
        Ok(this)
    }

    pub(crate) fn load_default() -> Self {
        Self::load("kb_lighting.toml").unwrap_or_else(|e| {
            warn!("Failed to load kb_lighting.toml: {}", e);
            Self::default()
        })
    }
}

#[derive(Debug, Default, PartialEq, Deserialize)]
pub(crate) struct KeyboardLightingLangSettings(pub(crate) HashMap<String, KeyboardZoneColors>);

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::profile::Profile;
    use keympostor::map;

    #[test]
    fn test_save_load_settings() {
        let settings = AppSettings {
            logging_enabled: false,
            profiles_enabled: true,
            transform_layout: Some("test-layout".to_string()),
            main_window: MainWindowSettings {
                position: Some((0, 0)),
                size: Some((100, 200)),
                selected_page: Some(0),
                log_view: Default::default(),
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
        };

        assert!(settings.save("test_settings.toml").is_ok());

        let loaded = AppSettings::load("test_settings.toml").unwrap();
        assert_eq!(settings, loaded);
    }
}
