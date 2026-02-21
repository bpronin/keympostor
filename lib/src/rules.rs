use crate::action::KeyActionSequence;
use crate::error::KeyError;
use crate::trigger::KeyTrigger;
use crate::{key_err, key_error, write_joined};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::Write;
use std::fmt::{Display, Formatter};
use std::slice::Iter;
use std::str::{FromStr, Lines};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyTransformRule {
    pub trigger: KeyTrigger,
    pub actions: KeyActionSequence,
}

impl KeyTransformRule {
    fn from_str_pair(triggers_str: &str, actions_str: &str) -> Result<Vec<Self>, KeyError> {
        let triggers_list = KeyTrigger::from_str_expand_list(triggers_str)?;
        let sequences = KeyActionSequence::from_str_expand(actions_str)?;
        let mut rules = Vec::new();

        for triggers in triggers_list {
            let len_t = triggers.len();
            let len_s = sequences.len();
            for i in 0..len_t.max(len_s) {
                let rule = KeyTransformRule {
                    trigger: if i < len_t {
                        &triggers[i]
                    } else {
                        &triggers[len_t - 1]
                    }
                    .clone(),
                    actions: if i < len_s {
                        &sequences[i]
                    } else {
                        &sequences[len_s - 1]
                    }
                    .clone(),
                };

                rules.push(rule);
            }
        }

        Ok(rules)
    }

    fn from_str_expand(s: &str) -> Result<Vec<Self>, KeyError> {
        let mut parts = s.trim().split(":");
        Self::from_str_pair(
            parts
                .next()
                .ok_or(key_error!("Missing trigger part in `{s}`"))?,
            parts
                .next()
                .ok_or(key_error!("Missing rule part in `{s}`."))?,
        )
    }
}

impl Display for KeyTransformRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        write!(s, "{} : {}", self.trigger, self.actions)?;
        f.pad(&s)
    }
}

impl FromStr for KeyTransformRule {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = Self::from_str_expand(s)?;
        if vec.len() > 1 {
            return key_err!("String must be exactly single rule");
        }
        Ok(vec[0].clone())
    }
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct KeyTransformRules(Vec<KeyTransformRule>);

impl KeyTransformRules {
    pub fn from_lines(lines: Lines) -> Result<Self, KeyError> {
        let mut items = Vec::new();
        for line in lines {
            items.extend(KeyTransformRule::from_str_expand(line.trim())?);
        }

        Ok(Self(items))
    }

    pub fn iter(&self) -> Iter<'_, KeyTransformRule> {
        self.0.iter()
    }
}

impl Display for KeyTransformRules {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_joined!(f, &self.0, "\n")
    }
}

impl From<Vec<KeyTransformRule>> for KeyTransformRules {
    fn from(vec: Vec<KeyTransformRule>) -> Self {
        Self(vec)
    }
}

impl FromStr for KeyTransformRules {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_lines(s.trim().lines())
    }
}

impl Serialize for KeyTransformRules {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for rule in &self.0 {
            map.serialize_entry(&rule.trigger, &rule.actions)
                .unwrap_or_else(|e| {
                    panic!("Failed to serialize rule: {} {}", rule, e);
                });
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for KeyTransformRules {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(KeyTransformRuleVisitor)
    }
}

struct KeyTransformRuleVisitor;

impl<'de> Visitor<'de> for KeyTransformRuleVisitor {
    type Value = KeyTransformRules;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("map of string -> string")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut items = Vec::new();

        while let Some((k, v)) = map.next_entry::<String, String>()? {
            let rules = KeyTransformRule::from_str_pair(&k, &v).map_err(de::Error::custom)?;
            for rule in rules {
                items.push(rule);
            }
        }

        Ok(KeyTransformRules(items))
    }
}

#[macro_export]
macro_rules! key_rule {
    ($text:literal) => {
        KeyTransformRule::from_str($text).unwrap()
    };
}

#[macro_export]
macro_rules! key_rules {
    ($text:literal) => {
        KeyTransformRules::from_str($text).unwrap()
    };
}

#[cfg(test)]
pub mod tests {
    use crate::action::KeyActionSequence;
    use crate::rules::KeyTransformRule;
    use crate::rules::KeyTransformRules;
    use crate::trigger::KeyTrigger;
    use crate::{key_action_seq, key_trigger};
    use std::str::FromStr;

    // Transform rule

    #[test]
    fn test_key_transform_rule_display() {
        let actual = KeyTransformRule {
            trigger: key_trigger!("[LEFT_SHIFT] ENTER ↓"),
            actions: key_action_seq!("ENTER↓"),
        };

        assert_eq!(
            "|       [LEFT_SHIFT] ENTER↓ : ENTER↓|",
            format!("|{:>35}|", actual)
        );
    }

    #[test]
    fn test_key_transform_rule_trigger() {
        assert_eq!(
            key_trigger!("[LEFT_CTRL + LEFT_SHIFT] ENTER↓"),
            key_rule!("[LEFT_CTRL + LEFT_SHIFT] ENTER↓ : ENTER↓").trigger
        );
    }

    #[test]
    fn test_key_transform_rule_from_str() {
        assert_eq!(
            KeyTransformRule {
                trigger: key_trigger!("[LEFT_SHIFT] ENTER↓"),
                actions: key_action_seq!("A↓"),
            },
            KeyTransformRule::from_str("[LEFT_SHIFT] ENTER↓ : A↓").unwrap()
        );
    }

    #[test]
    fn test_key_transform_rule_from_str_to_vec() {
        assert_eq!(
            vec![
                key_rule!("NUM_DOT↓ : LEFT_ALT↓"),
                key_rule!("NUM_DELETE↓ : LEFT_ALT↓"),
            ],
            KeyTransformRule::from_str_expand("NUM_DOT↓, NUM_DELETE↓ : LEFT_ALT↓").unwrap()
        );

        assert_eq!(
            vec![
                key_rule!("A* : ENTER*"),
                key_rule!("[LEFT_CTRL]B* : ENTER*"),
                key_rule!("C^ : ENTER*"),
            ],
            KeyTransformRule::from_str_expand("A*, [LEFT_CTRL]B*, C^ : ENTER*").unwrap()
        );
    }

    #[test]
    fn test_key_transform_rule_serialize() {
        let source = key_rule!("[LEFT_SHIFT] ENTER↓ : ENTER↓");
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }

    // Transform rules

    #[test]
    fn test_key_transform_rules_from_str_expand_no_trigger_transition() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : B↓
                A↑ : B↓
                "#
            ),
            key_rules!(
                r#"
                A : B↓
                "#
            )
        );
    }

    #[test]
    fn test_key_transform_rules_from_str_expand_no_action_transition() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : B↓
                A↓ : B↑
                "#
            ),
            key_rules!(
                r#"
                A↓ : B
                "#
            )
        );
    }

    #[test]
    fn test_key_transform_rules_from_str_expand_no_any_transition() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : B↓
                A↑ : B↑
                "#
            ),
            key_rules!(
                "
                A : B
                "
            )
        );
    }

    #[test]
    fn test_key_transform_rules_from_str_expand_list_no_any_transition() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : C↓
                A↑ : C↑
                B↓ : C↓
                B↑ : C↑
                "#
            ),
            key_rules!(
                r#"
                    A,B : C
                    "#
            )
        );
    }

    #[test]
    fn test_key_transform_rules_from_str_expand_multiple_actions() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : B↓ → C↑
                A↑ : B↑ → C↑
                "#
            ),
            key_rules!(
                "
                A : B → C↑
                "
            )
        );
    }

    #[test]
    fn test_key_transform_rules_from_str_expand() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : B↓
                A↑ : B↓
                C↓ : D↓
                C↓ : D↑
                E↓ : F↓
                E↑ : F↑
                "#
            ),
            key_rules!(
                r#"
                A : B↓
                C↓ : D
                E : F
                "#
            )
        );
    }

    #[test]
    fn test_key_transform_rules_from_str_to_vec() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : C↓
                B↓ : C↓
                "#
            ),
            key_rules!("A↓, B↓ : C↓")
        );
    }

    #[test]
    fn test_key_transform_rules_deserialize() {
        assert_eq!(
            KeyTransformRules::from(vec![key_rule!("A* : C*"), key_rule!("B* : C*")],),
            toml::from_str(
                r#"
                "A*, B*" = "C*"
                "#,
            )
            .unwrap()
        );
    }
}
