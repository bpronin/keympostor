use crate::keyboard::key::Key;
use crate::keyboard::key_modifiers::{KeyModifiers, KeyModifiersMatrix};
use crate::keyboard::transform_rules::{KeyTransformRule, KeyTransformRules};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.name())
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

// impl Serialize for KeyModifiersMatrix {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_str(&self.to_string())
//     }
// }

// impl<'de> Deserialize<'de> for KeyModifiersMatrix {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         String::deserialize(deserializer)?
//             .parse()
//             .map_err(de::Error::custom)
//     }
// }

impl Serialize for KeyModifiers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for KeyModifiers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
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

#[cfg(test)]
mod tests {
    use crate::keyboard::key::Key;
    use crate::keyboard::key_action::KeyActionSequence;
    use crate::keyboard::key_action::KeyTransition::{Down, Up};
    use crate::keyboard::key_action::{KeyAction, KeyTransition};
    use crate::keyboard::key_modifiers::KeyModifiers;
    use crate::keyboard::key_trigger::KeyTrigger;
    use crate::keyboard::transform_rules::{KeyTransformProfile, KeyTransformRule};
    use crate::{key, key_action_seq, key_profile, key_trigger};
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_key_serialize() {
        /* TOML requires wrapper */
        #[derive(Debug, Serialize, Deserialize)]
        struct Wrapper {
            key: Key,
        }

        let source = Wrapper { key: key!("ENTER") };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<Wrapper>(&text).unwrap();
        assert_eq!(source.key, actual.key);

        let source = Wrapper {
            key: key!("NUM_ENTER"),
        };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<Wrapper>(&text).unwrap();
        assert_eq!(source.key, actual.key);
    }

    #[test]
    fn test_key_transition_serialize() {
        /* TOML requires wrapper */
        #[derive(Debug, Serialize, Deserialize)]
        struct Wrapper {
            value: KeyTransition,
        }

        let source = Wrapper { value: Down };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<Wrapper>(&text).unwrap();
        assert_eq!(source.value, actual.value);

        let source = Wrapper { value: Up };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<Wrapper>(&text).unwrap();
        assert_eq!(source.value, actual.value);
    }

    #[test]
    fn test_key_action_serialize() {
        let source = KeyAction {
            key: key!("ENTER"),
            transition: Down,
        };
        let text = toml::to_string_pretty(&source).unwrap();

        let actual = toml::from_str::<KeyAction>(&text).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_action_sequence_serialize() {
        let source = key_action_seq!("ENTER↓ → SHIFT↓");
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_modifiers_serialize() {
        /* TOML requires wrapper */
        #[derive(Debug, Serialize, Deserialize)]
        struct Wrapper {
            value: KeyModifiers,
        }

        let source = Wrapper {
            value: "LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN".parse().unwrap(),
        };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<Wrapper>(&text).unwrap();
        assert_eq!(source.value, actual.value);
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
