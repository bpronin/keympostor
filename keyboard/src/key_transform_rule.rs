use crate::key_action::KeyActionSequence;
use crate::key_trigger::KeyTrigger;
use crate::write_joined;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyTransformRule {
    pub source: KeyTrigger,
    pub target: KeyActionSequence,
}

impl KeyTransformRule {}

impl FromStr for KeyTransformRule {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.trim().split(":");
        Ok(Self {
            source: parts.next().unwrap().parse()?,
            target: parts.next().unwrap().parse()?,
        })
    }
}

impl Display for KeyTransformRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {}", self.source, self.target)
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyTransformProfile {
    pub title: String,
    pub rules: Vec<KeyTransformRule>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeyTransformProfileToml {
    profile: String,
    rules: BTreeMap<String, String>,
}

impl KeyTransformProfile {
    pub fn load(path: &str) -> Result<Self, String> {
        let toml = toml::from_str::<KeyTransformProfileToml>(
            &fs::read_to_string(&path)
                .map_err(|e| format!("Unable to read {} file.\n{}", path, e))?,
        )
        .map_err(|e| format!("Unable to parse {}.\n{}", path, e))?;

        Ok(KeyTransformProfile {
            title: toml.profile,
            rules: toml
                .rules
                .iter()
                .map(|entry| KeyTransformRule {
                    source: entry.0.parse().unwrap(),
                    target: entry.1.parse().unwrap(),
                })
                .collect(),
        })
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let toml = KeyTransformProfileToml {
            profile: self.title.clone(),
            rules: BTreeMap::from_iter(
                self.rules
                    .iter()
                    .map(|rule| (rule.source.to_string(), rule.target.to_string()))
                    .collect::<Vec<_>>(),
            ),
        };

        fs::write(
            path,
            toml::to_string(&toml).map_err(|e| format!("Unable to serialize {}.\n{}", path, e))?,
        )
        .map_err(|e| format!("Unable to write {} file.\n{}", path, e))
    }

    // fn load_raw(path: &str) -> Result<Self, String> {
    //     toml::from_str(
    //         &fs::read_to_string(&path)
    //             .map_err(|e| format!("Unable to read {} file.\n{}", path, e))?,
    //     )
    //     .map_err(|e| format!("Unable to parse {}.\n{}", path, e))
    // }
    //
    // fn save_raw(&self, path: &str) -> Result<(), String> {
    //     fs::write(
    //         path,
    //         toml::to_string(self).map_err(|e| format!("Unable to serialize {}.\n{}", path, e))?,
    //     )
    //     .map_err(|e| format!("Unable to write {} file.\n{}", path, e))
    // }
}

impl Display for KeyTransformProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{};", self.title)?;
        write_joined!(f, &self.rules, ";\r\n")?;
        write!(f, ";")
    }
}

impl FromStr for KeyTransformProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.trim().trim_end_matches(';').split(';');
        Ok(Self {
            title: split.next().unwrap().trim().into(),
            rules: split.map(str::parse).collect::<Result<_, _>>()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::key::Key;
    use crate::key_action::KeyTransition::Down;
    use crate::key_action::{KeyAction, KeyActionSequence};
    use crate::key_modifiers::{KM_LSHIFT, KM_RSHIFT};
    use crate::key_transform_rule::{KeyTransformProfile, KeyTransformRule};
    use crate::key_trigger::KeyTrigger;
    use crate::{key, key_act, key_trig};

    #[macro_export]
    macro_rules! key_rule {
        ($text:literal) => {
            $text.parse::<KeyTransformRule>().unwrap()
        };
    }

    #[macro_export]
    macro_rules! key_profile {
        ($text:literal) => {
            $text.parse::<KeyTransformProfile>().unwrap()
        };
    }

    #[test]
    fn test_key_transform_rule_source() {
        let rule = key_rule!("[CTRL + SHIFT] ENTER↓ : ENTER↓");
        assert_eq!(key_trig!("[CTRL + SHIFT] ENTER↓"), rule.source);
    }

    #[test]
    fn test_key_transform_rule_display() {
        let source = KeyTransformRule {
            source: KeyTrigger {
                action: key_act!("ENTER↓"),
                modifiers: KM_LSHIFT,
            },
            target: KeyActionSequence {
                actions: vec![KeyAction {
                    key: key!("ENTER"),
                    transition: Down,
                }],
            },
        };

        assert_eq!("[LEFT_SHIFT]ENTER↓ : ENTER↓", format!("{}", source));
    }

    #[test]
    fn test_key_transform_rule_parse() {
        let expected = KeyTransformRule {
            source: KeyTrigger {
                action: key_act!("ENTER↓"),
                modifiers: KM_LSHIFT | KM_RSHIFT,
            },
            target: KeyActionSequence {
                actions: vec![KeyAction {
                    key: key!("ENTER"),
                    transition: Down,
                }],
            },
        };

        assert_eq!(expected, "[SHIFT] ENTER ↓ : ENTER ↓".parse().unwrap());
    }

    #[test]
    fn test_key_transform_rule_serialize() {
        let source = KeyTransformRule {
            source: KeyTrigger {
                action: key_act!("ENTER↓"),
                modifiers: KM_LSHIFT,
            },
            target: KeyActionSequence {
                actions: vec![KeyAction {
                    key: key!("ENTER"),
                    transition: Down,
                }],
            },
        };

        let text = toml::to_string_pretty(&source).unwrap();

        let actual = toml::from_str::<KeyTransformRule>(&text).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_transform_profile_parse() {
        let actual = key_profile!(
            "
            Test profile;
            A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑;
            [CTRL + SHIFT] ENTER↓ : ENTER↓ → ENTER↑;
            "
        );

        let expected = KeyTransformProfile {
            title: "Test profile".to_string(),
            rules: vec![
                key_rule!("A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"),
                key_rule!("[CTRL + SHIFT] ENTER↓: ENTER↓ → ENTER↑"),
            ],
        };

        assert_eq!(expected, actual);
    }

    /*    todo:;
        #[test]
        fn test_key_transform_rules_parse_split_transition() {
            let actual: KeyTransformProfile = "
            Test profile;
            A : B;
            "
            .parse()
            .unwrap();

            println!("{}", actual);

            let expected: KeyTransformProfile = "
            Test profile;
            A↓ : B↓;
            A↑ : B↑;
            "
            .parse()
            .unwrap();

            assert_eq!(expected, actual);
        }
    */

    /*    todo:
        #[test]
        fn test_key_transform_rules_parse_expand_transition() {
            let actual: KeyTransformProfile = "
            Test profile;
            A↓↑ : B↓↑;
            "
            .parse()
            .unwrap();

            println!("{}", actual);

            let expected: KeyTransformProfile = "
            Test profile;
            A↓ → A↓: B↓ → B↑;
            "
            .parse()
            .unwrap();

            assert_eq!(expected, actual);
        }
    */

    #[test]
    fn test_key_transform_profile_serialize() {
        let actual = KeyTransformProfile::load("../test/profiles/test.toml").unwrap();

        let expected = key_profile!(
            "
            Test profile;
            CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑;
            [LEFT_SHIFT] CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑;
            "
        );

        assert_eq!(expected, actual);

        actual.save("../test/profiles/test-copy.toml").unwrap()
    }
}
