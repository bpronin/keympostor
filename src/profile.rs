use crate::keyboard::transform_rules::KeyTransformRules;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyTransformProfile {
    pub title: String,
    pub auto_activation: Option<ActivationRules>,
    pub rules: KeyTransformRules,
}

impl KeyTransformProfile {
    pub fn load(path: &str) -> Result<Self, String> {
        toml::from_str(
            &fs::read_to_string(&path)
                .map_err(|e| format!("Unable to read {} file. {}", path, e))?,
        )
        .map_err(|e| format!("Unable to parse {}. {}", path, e))
    }
}

impl Default for KeyTransformProfile {
    fn default() -> Self {
        Self {
            title: "No profile".to_string(),
            auto_activation: Default::default(),
            rules: Default::default(),
        }
    }
}

impl Display for KeyTransformProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.title, self.rules)
    }
}

impl FromStr for KeyTransformProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.trim().lines();

        Ok(Self {
            title: lines.next().ok_or("Error parsing title.")?.trim().into(),
            rules: KeyTransformRules::from_str_lines(lines)?,
            auto_activation: None,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActivationRules {
    pub window_title: String,
}

#[cfg(test)]
pub mod tests {
    use crate::key_rule;
    use crate::keyboard::transform_rules::KeyTransformRule;
    use crate::keyboard::transform_rules::KeyTransformRules;
    use crate::profile::KeyTransformProfile;
    use std::fs;

    #[macro_export]
    macro_rules! key_profile {
        ($text:expr) => {
            $text.parse::<KeyTransformProfile>().unwrap()
        };
    }

    impl KeyTransformProfile {
        pub(crate) fn save(&self, path: &str) -> Result<(), String> {
            fs::write(
                path,
                toml::to_string_pretty(self)
                    .map_err(|e| format!("Unable to serialize {}. {}", path, e))?,
            )
            .map_err(|e| format!("Unable to write {} file. {}", path, e))
        }
    }

    #[test]
    fn test_profile_display() {
        todo!()
    }

    #[test]
    fn test_profile_from_str() {
        assert_eq!(
            KeyTransformProfile {
                title: "Test profile".to_string(),
                rules: KeyTransformRules {
                    items: vec![
                        key_rule!("A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"),
                        key_rule!("[LEFT_CTRL + LEFT_SHIFT] ENTER↓: ENTER↓ → ENTER↑"),
                    ],
                },
                ..Default::default()
            },
            key_profile!(
                r#"
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
            Test profile
            [LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑
            []CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
            "#
        );
        let expected = r#"
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
            Test profile
            [LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑
            []CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
            "#
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_transform_profile_load() {
        let actual = KeyTransformProfile::load("test/profiles/test.toml").unwrap();

        /* NOTE: rules deserialized as sorted map so check the "expected" order */
        let expected = key_profile!(
            "
            Test profile
            [LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑
            []CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
            "
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_transform_profile_load_fails() {
        assert!(KeyTransformProfile::load("test/profiles/bad.toml").is_err());
    }

    #[test]
    fn test_key_transform_profile_save() {
        let actual = KeyTransformProfile::load("test/profiles/test.toml").unwrap();
        actual.save("test/profiles/test-copy.toml").unwrap();
        let expected = KeyTransformProfile::load("test/profiles/test-copy.toml").unwrap();

        assert_eq!(expected, actual);
    }
}
