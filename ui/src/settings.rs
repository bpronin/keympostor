use anyhow::{Context, Result};
use keympostor::profile::Profiles;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;

const FILE_PATH: &str = "settings.toml";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct AppSettings {
    pub(crate) processing_enabled: bool,
    pub(crate) logging_enabled: bool,
    pub(crate) window_profile_enabled: bool,
    pub(crate) main_window: MainWindow,
    pub(crate) profile: Option<String>,
    pub(crate) profiles: Option<Profiles>,
    pub(crate) window_profile: Option<Vec<WindowProfile>>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            processing_enabled: true,
            logging_enabled: false,
            window_profile_enabled: false,
            main_window: Default::default(),
            profiles: None,
            profile: None,
            window_profile: Default::default(),
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
        Self::load(FILE_PATH)
    }

    pub(crate) fn save_default(&self) -> Result<()> {
        self.save(FILE_PATH)
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct MainWindow {
    pub(crate) position: Option<(i32, i32)>,
    pub(crate) size: Option<(u32, u32)>,
    pub(crate) selected_page: Option<usize>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct WindowProfile {
    pub(crate) rule: String,
    pub(crate) profile: Option<String>,
}

impl WindowProfile {
    pub(crate) fn regex(&self) -> Regex {
        Regex::new(self.rule.as_str()).unwrap()
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;
    use super::*;
    use keympostor::profile::Profile;

    #[test]
    fn test_save_load_settings() {
        let settings = AppSettings {
            processing_enabled: false,
            logging_enabled: false,
            window_profile_enabled: true,
            profile: Some("test-profile".to_string()),
            main_window: MainWindow {
                position: Some((0, 0)),
                size: Some((100, 200)),
                selected_page: Some(0),
            },
            window_profile: Some(vec![
                WindowProfile {
                    rule: "Chrome".to_string(),
                    profile: Some("game".to_string()),
                },
                WindowProfile {
                    rule: "TOTALCMD64.EXE".to_string(),
                    profile: Some("game".to_string()),
                },
            ]),
            profiles: Some(Profiles (
                vec![
                    Profile::from_str(
                        r#"
                        one
                        First profile
                        A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
                        [LEFT_CTRL + LEFT_SHIFT] ENTER↓ : ENTER↓ → ENTER↑
                        "#
                    ).unwrap(),
                    Profile::from_str(
                        r#"
                        game
                        Game profile
                        [LEFT_CTRL + LEFT_SHIFT] ENTER↓ : ENTER↓ → ENTER↑
                        "#
                    ).unwrap(),
                ],
            )),
        };

        assert!(settings.save("test_settings.toml").is_ok());
        let loaded = AppSettings::load("test_settings.toml");
        assert_eq!(settings, loaded);
        assert_eq!(AppSettings::default(), AppSettings::load(""));
    }
}
