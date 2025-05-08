use crate::key_action::{KeyAction, KeyActionSequence};
use crate::write_joined;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyTransformRule {
    pub source: KeyAction,
    pub target: KeyActionSequence,
}

impl KeyTransformRule {}

impl FromStr for KeyTransformRule {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(":");
        Ok(Self {
            source: split.next().unwrap().parse()?,
            target: split.next().unwrap().parse()?,
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

impl KeyTransformProfile {
    pub fn load(path: &str) -> Result<Self, String> {
        let json = fs::read_to_string(&path)
            .map_err(|e| format!("Unable to read {} file.\n{}", path, e))?;

        Ok(serde_json::from_str(&json).map_err(|e| format!("Unable to parse {}.\n{}", path, e))?)
    }
}

impl Display for KeyTransformProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{};", self.title)?;
        write_joined!(f, &self.rules, ";\n")?;
        write!(f, ";")
    }
}

impl FromStr for KeyTransformProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.trim().trim_end_matches(';').split(';');
        let title = split.next().unwrap().trim();
        let mut rules = vec![];
        for rs in split {
            rules.push(rs.parse()?);
        }
        Ok(Self {
            title: title.into(),
            rules,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::key::KeyCode::SC;
    use crate::key::{KeyCode, ScanCode, VirtualKey};
    use crate::key_action::KeyTransition::Down;
    use crate::key_action::{KeyAction, KeyActionSequence};
    use crate::key_transform_rule::{KeyTransformProfile, KeyTransformRule};
    use crate::{key, key_act};
    use KeyCode::VK;

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
        let rule = key_rule!("VK_RETURN + VK_CONTROL + VK_SHIFT↓ : SC_ENTER↓");
        assert_eq!(key_act!("VK_RETURN + VK_CONTROL + VK_SHIFT↓"), rule.source);
    }

    #[test]
    fn test_key_transform_rule_display() {
        let source = KeyTransformRule {
            source: KeyAction {
                keys: vec![key!("VK_SHIFT"), key!("VK_RETURN")],
                transition: Down,
            },
            target: KeyActionSequence {
                actions: vec![KeyAction {
                    keys: vec![key!("SC_ENTER")],
                    transition: Down,
                }],
            },
        };

        assert_eq!("VK_SHIFT + VK_RETURN↓ : SC_ENTER↓", format!("{}", source));
    }

    #[test]
    fn test_key_transform_rule_parse() {
        let actual = "VK_SHIFT + VK_RETURN ↓ : SC_ENTER↓".parse().unwrap();

        let expected = KeyTransformRule {
            source: KeyAction {
                keys: vec![key!("VK_SHIFT"), key!("VK_RETURN")],
                transition: Down,
            },
            target: KeyActionSequence {
                actions: vec![KeyAction {
                    keys: vec![key!("SC_ENTER")],
                    transition: Down,
                }],
            },
        };

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_transform_rule_serialize() {
        let source = KeyTransformRule {
            source: KeyAction {
                keys: vec![key!("VK_SHIFT"), key!("VK_RETURN")],
                transition: Down,
            },
            target: KeyActionSequence {
                actions: vec![KeyAction {
                    keys: vec![key!("SC_ENTER")],
                    transition: Down,
                }],
            },
        };

        let json = serde_json::to_string_pretty(&source).unwrap();

        let actual = serde_json::from_str::<KeyTransformRule>(&json).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_transform_rules_parse() {
        let actual = key_profile!(
            "
            Test profile;
            SC_A↓ : SC_LEFT_WINDOWS↓ → SC_SPACE↓ → SC_SPACE↑ → SC_LEFT_WINDOWS↑;
            VK_SHIFT + VK_CAPITAL↓ : VK_CAPITAL↓ → VK_CAPITAL↑;
            "
        );

        let expected = key_profile!(
            "
            Test profile;
            SC_A↓ : SC_LEFT_WINDOWS↓ → SC_SPACE↓ → SC_SPACE↑ → SC_LEFT_WINDOWS↑;
            VK_SHIFT + VK_CAPITAL↓ : VK_CAPITAL↓ → VK_CAPITAL↑;
            "
        );

        println!("{}", actual);
        println!("{}", expected);

        assert_eq!(expected, actual);
    }

    /*    todo:;
        #[test]
        fn test_key_transform_rules_parse_split_transition() {
            let actual: KeyTransformProfile = "
            Test profile;
            VK_A : VK_B;
            "
            .parse()
            .unwrap();

            println!("{}", actual);

            let expected: KeyTransformProfile = "
            Test profile;
            VK_A↓ : VK_B↓;
            VK_A↑ : VK_B↑;
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
            VK_A↓↑ : VK_B↓↑;
            "
            .parse()
            .unwrap();

            println!("{}", actual);

            let expected: KeyTransformProfile = "
            Test profile;
            VK_A↓ → VK_A↓: VK_B↓ → VK_B↑;
            "
            .parse()
            .unwrap();

            assert_eq!(expected, actual);
        }
    */

    #[test]
    fn test_key_transform_rules_serialize() {
        let actual = KeyTransformProfile::load("../test/profiles/test.json").unwrap();

        let expected = key_profile!(
            "
            Test profile;
            SC_CAPS_LOCK↓ : SC_LEFT_WINDOWS↓ → SC_SPACE↓ → SC_SPACE↑ → SC_LEFT_WINDOWS↑;
            VK_CAPITAL + VK_SHIFT ↓ : VK_CAPITAL↓ → VK_CAPITAL↑;
            "
        );

        assert_eq!(expected, actual);
    }
}
