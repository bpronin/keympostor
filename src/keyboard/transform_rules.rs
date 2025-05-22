use crate::keyboard::key_action::KeyActionSequence;
use crate::keyboard::key_trigger::KeyTrigger;
use crate::write_joined;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct KeyTransformRule {
    pub(crate) source: KeyTrigger,
    pub(crate) target: KeyActionSequence,
}

impl Display for KeyTransformRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {}", self.source, self.target)
    }
}

#[derive(Debug, Eq, PartialEq, Default)]
pub(crate) struct KeyTransformRules {
    pub(crate) items: Vec<KeyTransformRule>,
}

impl Display for KeyTransformRules {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_joined!(f, &self.items, "\n")
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
pub(crate) struct KeyTransformProfile {
    pub(crate) title: String,
    pub(crate) rules: KeyTransformRules,
}

impl KeyTransformProfile {
    pub(crate) fn load(path: &str) -> Result<Self, String> {
        toml::from_str(
            &fs::read_to_string(&path)
                .map_err(|e| format!("Unable to read {} file. {}", path, e))?,
        )
        .map_err(|e| format!("Unable to parse {}. {}", path, e))
    }

    // fn save(&self, path: &str) -> Result<(), String> {
    //     fs::write(
    //         path,
    //         toml::to_string(self).map_err(|e| format!("Unable to serialize {}. {}", path, e))?,
    //     )
    //     .map_err(|e| format!("Unable to write {} file. {}", path, e))
    // }
}

impl Default for KeyTransformProfile {
    fn default() -> Self {
        Self {
            title: "No profile".to_string(),
            rules: Default::default(),
        }
    }
}

impl Display for KeyTransformProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.title, self.rules)
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key_action::KeyActionSequence;
    use crate::keyboard::key_trigger::KeyTrigger;
    use crate::keyboard::transform_rules::{KeyTransformProfile, KeyTransformRule};
    use crate::{key_action_seq, key_trigger};

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
        let rule = key_rule!("[LEFT_CTRL + LEFT_SHIFT] ENTER↓ : ENTER↓");
        assert_eq!(key_trigger!("[LEFT_CTRL + LEFT_SHIFT] ENTER↓"), rule.source);
    }

    #[test]
    fn test_key_transform_rule_display() {
        let source = KeyTransformRule {
            source: key_trigger!("[LEFT_SHIFT] ENTER ↓"),
            target: key_action_seq!("ENTER↓"),
        };

        assert_eq!("[LEFT_SHIFT]ENTER↓ : ENTER↓", format!("{}", source));
    }

    #[test]
    fn test_key_transform_rule_serialize() {
        let source = KeyTransformRule {
            source: key_trigger!("[LEFT_SHIFT] ENTER↓"),
            target: key_action_seq!("ENTER↓"),
        };

        let text = toml::to_string_pretty(&source).unwrap();

        let actual = toml::from_str::<KeyTransformRule>(&text).unwrap();
        assert_eq!(source, actual);
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
}
