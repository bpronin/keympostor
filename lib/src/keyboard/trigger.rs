use serde::{Deserializer, Serializer};
use crate::keyboard::action::KeyAction;
use crate::keyboard::modifiers::KeyModifiers;
use crate::keyboard::modifiers::KeyModifiers::Any;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{de, Deserialize, Serialize};
use crate::keyboard::error::KeyError;
use crate::{deserialize_from_string, serialize_to_string};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct KeyTrigger {
    pub action: KeyAction,
    pub modifiers: KeyModifiers,
}

impl KeyTrigger {
    pub(crate) fn from_str_list(s: &str) -> Result<Vec<Vec<Self>>, KeyError> {
        let mut list = Vec::new();
        for part in s.split(',') {
            list.push(Self::from_str_expand(part)?);
        }

        Ok(list)
    }

    fn from_str_expand(s: &str) -> Result<Vec<KeyTrigger>, KeyError> {
        let ts = s.trim();
        let mut list = Vec::with_capacity(2);

        if let Some(s) = ts.strip_prefix('[') {
            let mut parts = s.split(']');
            let modifiers = KeyModifiers::from_str(parts.next().expect("Missing modifiers part"))?;
            let actions = KeyAction::from_str_expand(parts.next().expect("Missing actions part"))?;
            for action in actions {
                list.push(Self { action, modifiers });
            }
        } else {
            let actions = KeyAction::from_str_expand(ts)?;
            for action in actions {
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
        let ts = s.trim();
        if let Some(s) = ts.strip_prefix('[') {
            let mut parts = s.split(']');
            Ok(Self {
                /* Modifiers go first! */
                modifiers: KeyModifiers::from_str(parts.next().expect("Missing modifiers part"))?,
                action: KeyAction::from_str(parts.next().expect("Missing action part."))?,
            })
        } else {
            Ok(Self {
                action: KeyAction::from_str(ts)?,
                modifiers: Any,
            })
        }
    }
}

impl Display for KeyTrigger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("{}{}", self.modifiers, self.action), f)
    }
}

impl Serialize for KeyTrigger {
    serialize_to_string!();
}

impl<'de> Deserialize<'de> for KeyTrigger {
    deserialize_from_string!();
}

#[cfg(test)]
mod tests {
    use crate::keyboard::modifiers::ModifierKeys;
    use crate::keyboard::modifiers::KeyModifiers::{All, Any};
    use crate::keyboard::modifiers::KM_LSHIFT;
    use crate::keyboard::modifiers::KM_NONE;
    use crate::keyboard::trigger::KeyAction;
    use crate::keyboard::trigger::KeyTrigger;
    use crate::{key_action, key_mod};
    use std::str::FromStr;

    #[macro_export]
    macro_rules! key_trigger {
        ($text:literal) => {
            $text.parse::<KeyTrigger>().unwrap()
        };
    }

    #[test]
    fn test_key_trigger_display() {
        let actual = KeyTrigger {
            action: key_action!("A↓"),
            modifiers: All(KM_LSHIFT),
        };
        assert_eq!("[LEFT_SHIFT]A↓", format!("{}", actual));

        let actual = KeyTrigger {
            action: key_action!("A↓"),
            modifiers: All(KM_NONE),
        };
        assert_eq!("[]A↓", format!("{}", actual));

        let actual = KeyTrigger {
            action: key_action!("A↓"),
            modifiers: Any,
        };
        assert_eq!("A↓", format!("{}", actual));

        let actual = KeyTrigger {
            action: key_action!("A↓"),
            modifiers: All(KM_LSHIFT),
        };
        assert_eq!("|      [LEFT_SHIFT]A↓|", format!("|{:>20}|", actual));
    }

    #[test]
    fn test_key_trigger_from_str_all_modifiers() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                modifiers: All(key_mod!("LEFT_SHIFT")),
            },
            KeyTrigger::from_str("[LEFT_SHIFT] A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_no_modifiers() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                modifiers: All(KM_NONE),
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
    fn test_key_trigger_from_str_list() {
        assert_eq!(
            vec![
                vec![key_trigger!("A*")],
                vec![key_trigger!("[LEFT_CTRL]B^")],
                vec![key_trigger!("C*")],
            ],
            KeyTrigger::from_str_list("A*, [LEFT_CTRL]B^, C*").unwrap()
        );

        assert_eq!(
            vec![vec![key_trigger!("A*")]],
            KeyTrigger::from_str_list("A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_expand() {
        assert_eq!(
            vec![vec![key_trigger!("A↓"), key_trigger!("A↑")]],
            KeyTrigger::from_str_list("A↓↑").unwrap()
        );

        assert_eq!(
            vec![vec![key_trigger!("A↓"), key_trigger!("A↑")]],
            KeyTrigger::from_str_list("A").unwrap()
        );

        assert_eq!(
            vec![vec![
                key_trigger!("[LEFT_CTRL]A↓"),
                key_trigger!("[LEFT_CTRL]A↑")
            ]],
            KeyTrigger::from_str_list("[LEFT_CTRL]A↓↑").unwrap()
        );

        assert_eq!(
            vec![vec![
                key_trigger!("[LEFT_CTRL]A↓"),
                key_trigger!("[LEFT_CTRL]A↑")
            ]],
            KeyTrigger::from_str_list("[LEFT_CTRL]A").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_list_expand() {
        assert_eq!(
            vec![
                vec![key_trigger!("A↓"), key_trigger!("A↑")],
                vec![key_trigger!("B↓"), key_trigger!("B↑")],
            ],
            KeyTrigger::from_str_list("A,B").unwrap()
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
            KeyTrigger::from_str_list("[LEFT_SHIFT] A, [LEFT_CTRL + LEFT_ALT] B").unwrap()
        );
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

        let source = key_trigger!("C^");
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<KeyTrigger>(&text).unwrap();

        assert_eq!(source, actual);

        let source = key_trigger!("D^");
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<KeyTrigger>(&text).unwrap();

        assert_eq!(source, actual);
    }

}
