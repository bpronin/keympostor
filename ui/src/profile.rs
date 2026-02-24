use keympostor::{load_from_toml_file, save_to_toml_file};
use log::debug;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::str::FromStr;

const PROFILES_FILE: &str = "profiles.toml";

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub(crate) struct Profile {
    pub(crate) activation_rule: Option<String>,
    pub(crate) transform_layout: String,
    pub(crate) keyboard_locale: Option<String>,
}

impl Profile {
    pub(crate) fn rule_regex(&self) -> Option<Regex> {
        self.activation_rule
            .as_deref()
            .and_then(|r| Regex::from_str(r).ok())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Profiles(pub HashMap<String, Profile>);

impl Profiles {
    pub(crate) fn load() -> Result<Self, Box<dyn Error>> {
        Self::load_from(PROFILES_FILE)
    }

    pub(crate) fn save(&self) {
        self.save_to(PROFILES_FILE)
            .expect("Failed to save settings");
        debug!("Profiles saved");
    }

    load_from_toml_file!();

    save_to_toml_file!();
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{map, str};

    #[test]
    fn test_regex_parsing() {
        let profile = Profile {
            activation_rule: Some(str!("")),
            ..Default::default()
        };

        assert!(profile.rule_regex().unwrap().is_match("test"));
    }

    #[test]
    fn test_save_load() {
        let profiles = Profiles(map![
            str!("chrome") => Profile {
                activation_rule: Some(str!("Chrome")),
                transform_layout: str!("desktop"),
                keyboard_locale: Some(str!("ru_ru"))
            },
            str!("tc") => Profile {
                activation_rule: Some(str!("TOTALCMD64.EXE")),
                transform_layout: str!("game"),
                keyboard_locale: Some(str!("en_en"))
            },
        ]);

        let path = "etc/test_data/test_profiles.toml";

        assert!(profiles.save_to(path).is_ok());

        let loaded = Profiles::load_from(path).unwrap();
        assert_eq!(profiles, loaded);
    }
}
