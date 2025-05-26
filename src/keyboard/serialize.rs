use crate::keyboard::key::Key;
use crate::keyboard::key_modifiers::KeyModifiersState;
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

impl Serialize for KeyModifiersState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for KeyModifiersState {
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
                .map(|rule| (rule.trigger.to_string(), rule.actions.to_string()))
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
                    trigger: k.parse().map_err(de::Error::custom)?,
                    actions: v.parse().map_err(de::Error::custom)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { items })
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key::Key;
    use crate::keyboard::key_action::KeyAction;
    use crate::keyboard::key_action::KeyActionSequence;
    use crate::keyboard::key_action::KeyTransition::{Down, Up};
    use crate::keyboard::key_modifiers::KeyModifiers::{All, Any};
    use crate::keyboard::key_modifiers::KeyModifiersState;
    use crate::keyboard::key_trigger::KeyTrigger;
    use crate::keyboard::transform_rules::{KeyTransformProfile, KeyTransformRule};
    use crate::{key, key_action, key_action_seq, key_mod, key_profile, key_rule, key_trigger};
    use serde::{Deserialize, Serialize};

    /* TOML requires root node to be annotated as #[derive(Serialize, Deserialize)] */
    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Wrapper<T> {
        value: T,
    }

    #[test]
    fn test_key_serialize() {
        let source = Wrapper {
            value: key!("ENTER"),
        };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);

        let source = Wrapper {
            value: key!("NUM_ENTER"),
        };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_transition_serialize() {
        let source = Wrapper { value: Down };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);

        let source = Wrapper { value: Up };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_modifiers_serialize() {
        let source = Wrapper {
            value: key_mod!("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN"),
        };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }

    #[test]
    fn test_keyboard_state_serialize() {
        let source = Wrapper {
            value: All(key_mod!("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN")),
        };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);

        let source = Wrapper { value: Any };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_action_serialize() {
        let source = key_action!("A*");
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<KeyAction>(&text).unwrap();

        assert_eq!(source, actual);

        let source = key_action!("B^");
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<KeyAction>(&text).unwrap();

        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_trigger_serialize() {
        let source = key_trigger!("[LEFT_SHIFT] A*");
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<KeyTrigger>(&text).unwrap();

        assert_eq!(source, actual);

        let source = key_trigger!("[] B*");
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<KeyTrigger>(&text).unwrap();

        assert_eq!(source, actual);

        let source = key_trigger!("[*] C^");
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<KeyTrigger>(&text).unwrap();

        assert_eq!(source, actual);

        let source = key_trigger!("D^");
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<KeyTrigger>(&text).unwrap();

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
    fn test_key_transform_rule_serialize() {
        let source = key_rule!("[LEFT_SHIFT] ENTER↓ : ENTER↓");
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }

    // #[test]
    // fn test_key_transform_rules_deserialize() {
    //     let text = r#"
    //     "A*, B*" = "C*"
    //     "#;
    //     let actual: KeyTransformRules = toml::from_str(&text).unwrap();
    //     let expected = KeyTransformRules {
    //         items: vec![
    //             key_rule!("A* : C*"),
    //             key_rule!("B* : C*")
    //         ],
    //     };
    //     assert_eq!(expected, actual);
    // }

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
