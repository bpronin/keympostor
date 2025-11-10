use std::collections::HashMap;
use crate::keyboard::error::KeyError;
use crate::keyboard::rules::KeyTransformRules;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use log::{warn};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub title: String,
    pub rules: KeyTransformRules,
}

impl Profile {
    pub fn load(path: &str) -> Result<Self> {
        toml::from_str(&fs::read_to_string(&path).context(format!("Unable to read {} file", path))?)
            .context(format!("Unable to parse {}", path))
    }
}

impl Display for Profile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.title, self.rules)
    }
}

impl FromStr for Profile {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.trim().lines();

        Ok(Self {
            name: lines
                .next()
                .ok_or(KeyError::new("Error parsing name."))?
                .trim()
                .into(),
            title: lines
                .next()
                .ok_or(KeyError::new("Error parsing title."))?
                .trim()
                .into(),
            rules: KeyTransformRules::from_str_lines(lines)?,
        })
    }
}

pub struct Profiles {
    pub items: HashMap<String, Profile>,
}

impl Profiles {
    pub fn load(path: &str) -> Result<Self> {
        let mut items = HashMap::new();
        for entry in fs::read_dir(Path::new(path))? {
            let path = entry?.path();
            if path.is_file() {
                let filename = path.to_str().unwrap();
                if let Ok(profile) = Profile::load(filename) {
                    items.insert(profile.name.clone(), profile);
                }else {
                    warn!("Ignored corrupted profile: {}", filename);
                }
            }
        }
        Ok(Self { items })
    }
}

impl Default for Profiles {
    fn default() -> Self {
        Self::load("profiles").unwrap()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::key_rule;
    use crate::keyboard::rules::KeyTransformRule;
    use crate::keyboard::rules::KeyTransformRules;
    use crate::profile::{Profile, Profiles};
    use anyhow::{Context, Error};
    use std::fs;

    #[macro_export]
    macro_rules! key_profile {
        ($text:expr) => {
            $text.parse::<Profile>().unwrap()
        };
    }

    impl Profile {
        pub(crate) fn save(&self, path: &str) -> Result<(), Error> {
            fs::write(
                path,
                toml::to_string_pretty(self).context(format!("Unable to serialize {}", path))?,
            )
            .context(format!("Unable to write {} file", path))
        }
    }

    #[test]
    fn test_profile_from_str() {
        assert_eq!(
            Profile {
                name: "test".to_string(),
                title: "Test profile".to_string(),
                rules: KeyTransformRules {
                    items: vec![
                        key_rule!("A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"),
                        key_rule!("[LEFT_CTRL + LEFT_SHIFT] ENTER↓: ENTER↓ → ENTER↑"),
                    ],
                }
            },
            key_profile!(
                r#"
                test
                Test profile
                A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
                [LEFT_CTRL + LEFT_SHIFT] ENTER↓ : ENTER↓ → ENTER↑
                "#
            )
        );
    }

    #[test]
    fn test_key_transform_profile_serialize() {
        let profile = key_profile!(
            r#"
            test
            Test profile
            [LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑
            []CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
            "#
        );
        let expected = r#"
            name = "test"
            title = "Test profile"
            [rules]
            "[LEFT_SHIFT]CAPS_LOCK↓" = "CAPS_LOCK↓ → CAPS_LOCK↑"
            "[]CAPS_LOCK↓" = "LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"
            "#;

        let actual = toml::to_string_pretty(&profile).unwrap();

        assert_eq!(
            expected.split_whitespace().collect::<String>(),
            actual.split_whitespace().collect::<String>()
        );
    }

    #[test]
    fn test_key_transform_profile_deserialize() {
        let actual = toml::from_str(
            &r#"
            name = "test"
            title = "Test profile"
            [rules]
            "[LEFT_SHIFT]CAPS_LOCK↓" = "CAPS_LOCK↓ → CAPS_LOCK↑"
            "[]CAPS_LOCK↓" = "LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"
            "#,
        )
        .unwrap();

        /* NOTE: rules deserialized as sorted map so check the "expected" order */
        let expected = key_profile!(
            r#"
            test
            Test profile
            [LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑
            []CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
            "#
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_transform_profile_load() {
        let actual = Profile::load("etc/test_data/profiles/test.toml").unwrap();

        /* NOTE: rules deserialized as sorted map so check the "expected" order */
        let expected = key_profile!(
            "
            test
            Test profile
            [LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑
            []CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
            "
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_transform_profile_load_fails() {
        assert!(Profile::load("test/profiles/bad.toml").is_err());
    }

    #[test]
    fn test_key_transform_profile_save() {
        let actual = Profile::load("etc/test_data/profiles/test.toml").unwrap();
        actual
            .save("etc/test_data/profiles/test-copy.toml")
            .unwrap();
        let expected = Profile::load("etc/test_data/profiles/test-copy.toml").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_profiles_load() {
        let result = Profiles::load("etc/test_data/profiles/");
        assert!(result.is_ok());
    }
}
