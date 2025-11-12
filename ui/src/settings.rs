use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use crate::profile::Profile;

const SETTINGS_FILE: &str = "settings.toml";
pub const LAYOUTS_PATH: &str = "layouts";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct AppSettings {
    pub(crate) processing_enabled: bool, //todo: rename to `transform_enabled` ?
    pub(crate) logging_enabled: bool,
    pub(crate) layouts_enabled: bool,
    pub(crate) main_window: MainWindow,
    pub(crate) layout: Option<String>,
    pub(crate) profiles: Option<Vec<Profile>>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            processing_enabled: true,
            logging_enabled: false,
            layouts_enabled: false,
            main_window: Default::default(),
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

    pub(crate) fn save(&self, filename: &str) -> Result<()> {
        let text = toml::to_string(self) /* dont want `to_string_pretty` */
            .context(format!("Error serializing `{}`", filename))?;
        fs::write(filename, text).context(format!("Error writing `{}`", filename))
    }

    pub(crate) fn load_default() -> Self {
        Self::load(SETTINGS_FILE)
    }

    pub(crate) fn save_default(&self) -> Result<()> {
        self.save(SETTINGS_FILE)
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct MainWindow {
    pub(crate) position: Option<(i32, i32)>,
    pub(crate) size: Option<(u32, u32)>,
    pub(crate) selected_page: Option<usize>,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_save_load_settings() {
        let settings = AppSettings {
            processing_enabled: false,
            logging_enabled: false,
            layouts_enabled: true,
            layout: Some("test-layout".to_string()),
            main_window: MainWindow {
                position: Some((0, 0)),
                size: Some((100, 200)),
                selected_page: Some(0),
            },
            profiles: Some(vec![
                Profile {
                    name: "chrome".to_string(),
                    activation_rule: "Chrome".to_string(),
                    layout: Some("game".to_string()),
                },
                Profile {
                    name: "tc".to_string(),
                    activation_rule: "TOTALCMD64.EXE".to_string(),
                    layout: Some("game".to_string()),
                },
            ]),
            // layouts: Some(Layouts(
            //     vec![
            //         Layout::from_str(
            //             r#"
            //             one
            //             First layout
            //             A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
            //             [LEFT_CTRL + LEFT_SHIFT] ENTER↓ : ENTER↓ → ENTER↑
            //             "#
            //         ).unwrap(),
            //         Layout::from_str(
            //             r#"
            //             game
            //             Game layout
            //             [LEFT_CTRL + LEFT_SHIFT] ENTER↓ : ENTER↓ → ENTER↑
            //             "#
            //         ).unwrap(),
            //     ],
            // )),
        };

        assert!(settings.save("test_settings.toml").is_ok());
        let loaded = AppSettings::load("test_settings.toml");
        assert_eq!(settings, loaded);
        assert_eq!(AppSettings::default(), AppSettings::load(""));
    }
}
