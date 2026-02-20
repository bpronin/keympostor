use crate::action::KeyAction;
use crate::error::KeyError;
use crate::modifiers::KeyModifiers;
use crate::modifiers::KeyModifiers::{All, Any};
use crate::{deserialize_from_string, key_err, key_error, serialize_to_string};
use serde::{de, Deserialize, Serialize};
use serde::{Deserializer, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyTrigger {
    pub action: KeyAction,
    pub modifiers: KeyModifiers,
}

impl KeyTrigger {
    pub(crate) fn from_str_expand_list(s: &str) -> Result<Vec<Vec<Self>>, KeyError> {
        Ok(s.split(',')
            .map(|part| Self::from_str_expand(part.trim()))
            .collect::<Result<Vec<_>, _>>()?)
    }

    fn from_str_expand(s: &str) -> Result<Vec<KeyTrigger>, KeyError> {
        let mut list = Vec::with_capacity(2);

        if s.starts_with('[') {
            let mut parts = s.split(']');

            let modifiers =
                KeyModifiers::from_str(parts.next().ok_or(key_error!("Missing modifiers part"))?)?;

            let actions = KeyAction::from_str_expand(
                parts.next().ok_or(key_error!("Missing actions part"))?,
            )?;

            for action in actions {
                list.push(Self { action, modifiers });
            }
        } else {
            for action in KeyAction::from_str_expand(s)? {
                list.push(Self {
                    action,
                    modifiers: Any,
                });
            }
        }

        Ok(list)
    }
}

impl FromStr for KeyTrigger {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = Self::from_str_expand(s)?;
        if vec.len() > 1 {
            return key_err!("String must be exactly single trigger");
        }
        vec.first()
            .cloned()
            .ok_or_else(|| key_error!("Invalid trigger string"))
    }
}

impl Display for KeyTrigger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.modifiers {
            Any => write!(f, "{}", self.action),
            All(m) => write!(f, "[{}] {}", m, self.action),
        }
    }
}

impl Serialize for KeyTrigger {
    serialize_to_string!();
}

impl<'de> Deserialize<'de> for KeyTrigger {
    deserialize_from_string!();
}

#[macro_export]
macro_rules! key_trigger {
    ($text:literal) => {
        KeyTrigger::from_str($text).unwrap()
    };
}

#[cfg(test)]
mod tests {
    use crate::key::Key;
    use crate::key_action;
    use crate::modifiers::KeyModifiers::{All, Any};
    use crate::state::tests::kb_state_from_keys;
    use crate::state::KeyboardState;
    use crate::trigger::KeyAction;
    use crate::trigger::KeyTrigger;
    use crate::utils::test::SerdeWrapper;
    use std::str::FromStr;

    #[test]
    fn test_key_trigger_display() {
        let actual = KeyTrigger {
            action: key_action!("A↓"),
            modifiers: All(kb_state_from_keys(&[Key::LeftShift])),
        };
        assert_eq!("[LEFT_SHIFT] A↓", format!("{}", actual));

        let actual = KeyTrigger {
            action: key_action!("A↓"),
            modifiers: All(KeyboardState::default()),
        };
        assert_eq!("[] A↓", format!("{}", actual));

        let actual = KeyTrigger {
            action: key_action!("A↓"),
            modifiers: Any,
        };
        assert_eq!("A↓", format!("{}", actual));

        let actual = KeyTrigger {
            action: key_action!("A↓"),
            modifiers: All(kb_state_from_keys(&[Key::LeftShift])),
        };
        assert_eq!("|     [LEFT_SHIFT] A↓|", format!("|{:>20}|", actual));
    }

    #[test]
    fn test_key_trigger_from_str_all_modifiers() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                modifiers: All(kb_state_from_keys(&[Key::LeftShift])),
            },
            KeyTrigger::from_str("[LEFT_SHIFT] A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_no_modifiers() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                modifiers: All(KeyboardState::default()),
            },
            KeyTrigger::from_str("[] A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_any_modifiers() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                modifiers: Any,
            },
            KeyTrigger::from_str("A*").unwrap()
        );

        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                modifiers: Any,
            },
            KeyTrigger::from_str("A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_to_vec() {
        assert_eq!(
            vec![
                vec![key_trigger!("A*")],
                vec![key_trigger!("[LEFT_CTRL]B^")],
                vec![key_trigger!("C*")],
            ],
            KeyTrigger::from_str_expand_list("A*, [LEFT_CTRL]B^, C*").unwrap()
        );

        assert_eq!(
            vec![vec![key_trigger!("A*")]],
            KeyTrigger::from_str_expand_list("A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_expand() {
        assert_eq!(
            vec![vec![key_trigger!("A↓"), key_trigger!("A↑")]],
            KeyTrigger::from_str_expand_list("A↓↑").unwrap()
        );

        assert_eq!(
            vec![vec![key_trigger!("A↓"), key_trigger!("A↑")]],
            KeyTrigger::from_str_expand_list("A").unwrap()
        );

        assert_eq!(
            vec![vec![
                key_trigger!("[LEFT_CTRL]A↓"),
                key_trigger!("[LEFT_CTRL]A↑")
            ]],
            KeyTrigger::from_str_expand_list("[LEFT_CTRL]A↓↑").unwrap()
        );

        assert_eq!(
            vec![vec![
                key_trigger!("[LEFT_CTRL]A↓"),
                key_trigger!("[LEFT_CTRL]A↑")
            ]],
            KeyTrigger::from_str_expand_list("[LEFT_CTRL]A").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_to_vec_expand() {
        assert_eq!(
            vec![
                vec![key_trigger!("A↓"), key_trigger!("A↑")],
                vec![key_trigger!("B↓"), key_trigger!("B↑")],
            ],
            KeyTrigger::from_str_expand_list("A,B").unwrap()
        );

        assert_eq!(
            vec![
                vec![
                    key_trigger!("[LEFT_SHIFT]A↓"),
                    key_trigger!("[LEFT_SHIFT]A↑")
                ],
                vec![
                    key_trigger!("[LEFT_CTRL + LEFT_ALT]B↓"),
                    key_trigger!("[LEFT_CTRL + LEFT_ALT]B↑")
                ],
            ],
            KeyTrigger::from_str_expand_list("[LEFT_SHIFT] A, [LEFT_CTRL + LEFT_ALT] B").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_serialize() {
        let source = SerdeWrapper::new(key_trigger!("[LEFT_SHIFT] A*"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);

        let source = SerdeWrapper::new(key_trigger!("[] B*"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);

        let source = SerdeWrapper::new(key_trigger!("C^"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);

        let source = SerdeWrapper::new(key_trigger!("D^"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }
}
