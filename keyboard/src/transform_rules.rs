use crate::key_action::KeyActionSequence;
use crate::key_trigger::KeyTrigger;
use crate::write_joined;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::str::{FromStr, Lines};

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
            source: parts.next().ok_or("Missing source part.")?.parse()?,
            target: parts.next().ok_or("Missing target part.")?.parse()?,
        })
    }
}

impl Display for KeyTransformRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {}", self.source, self.target)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct KeyTransformRules {
    pub(crate) items: Vec<KeyTransformRule>,
}

impl KeyTransformRules {
    fn from_lines(lines: Lines) -> Result<Self, String> {
        Ok(Self {
            items: lines.map(|l| l.parse()).collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl Display for KeyTransformRules {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_joined!(f, &self.items, "\n")
    }
}

impl FromStr for KeyTransformRules {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_lines(s.lines())
    }
}

impl Serialize for KeyTransformRules {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let map = BTreeMap::from_iter(
            self.items
                .iter()
                .map(|rule| (rule.source.to_string(), rule.target.to_string()))
                .collect::<Vec<_>>(),
        );
        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for KeyTransformRules {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let items = BTreeMap::<String, String>::deserialize(deserializer)?
            .iter()
            .map(|(k, v)| {
                Ok(KeyTransformRule {
                    source: k.parse().map_err(de::Error::custom)?,
                    target: v.parse().map_err(de::Error::custom)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { items })
    }
}
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyTransformProfile {
    pub title: String,
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

    fn save(&self, path: &str) -> Result<(), String> {
        fs::write(
            path,
            toml::to_string(self).map_err(|e| format!("Unable to serialize {}. {}", path, e))?,
        )
        .map_err(|e| format!("Unable to write {} file. {}", path, e))
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
            title: lines.next().expect("Error parsing title.").trim().into(),
            rules: KeyTransformRules::from_lines(lines)?,
        })
    }
}

/* --- TESTS --- */

#[cfg(test)]
mod tests {
    use crate::key::Key;
    use crate::key_action::KeyTransition::Down;
    use crate::key_action::{KeyAction, KeyActionSequence};
    use crate::key_modifiers::{KM_LSHIFT, KM_RSHIFT};
    use crate::key_trigger::KeyTrigger;
    use crate::transform_rules::{KeyTransformProfile, KeyTransformRule, KeyTransformRules};
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
            r#"
            Test profile
            A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
            [CTRL + SHIFT] ENTER↓ : ENTER↓ → ENTER↑
            "#
        );

        let expected = KeyTransformProfile {
            title: "Test profile".to_string(),
            rules: KeyTransformRules {
                items: vec![
                    key_rule!("A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"),
                    key_rule!("[CTRL + SHIFT] ENTER↓: ENTER↓ → ENTER↑"),
                ],
            },
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

    #[test]
    fn test_key_transform_rules_parse_expand_transition() {
        let actual = key_profile!(
            r#"
            Test profile
            A↓ : A↓↑ → B↓↑
            "#
        );
        let expected = key_profile!(
            r#"
            Test profile
            A↓ : A↓ → A↑ → B↓ → B↑ 
            "#
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_transform_profile_serialize() {
        let actual = KeyTransformProfile::load("../test/profiles/test.toml").unwrap();

        println!("{}", actual);
        let expected = key_profile!(
            r#"
            Test profile
            CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
            [LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑
            "#
        );

        assert_eq!(expected, actual);

        actual.save("../test/profiles/test-copy.toml").unwrap()
    }
}
