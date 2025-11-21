use crate::keyboard::error::KeyError;
use crate::keyboard::key::{key_by_name, Key};
use crate::keyboard::transition::KeyTransition;
use crate::keyboard::transition::KeyTransition::{Down, Up};
use crate::{deserialize_from_string, serialize_to_string, write_joined};
use serde::Deserializer;
use serde::Serializer;
use serde::{de, Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct KeyAction {
    pub key: &'static Key,
    pub transition: KeyTransition,
}

impl KeyAction {
    fn new(key: &'static Key, transition: KeyTransition) -> Self {
        Self { key, transition }
    }

    pub(crate) fn from_str_expand(s: &str) -> Result<Vec<Self>, KeyError> {
        let (sk, st) = match s.find(|c| ['^', '*', '↓', '↑'].contains(&c)) {
            Some(p) => {
                (s.get(..p).ok_or(KeyError::new("Missing key part"))?, s.get(p..))
            }
            None => (s, None),
        };

        let key = key_by_name(sk)?;

        let mut list = Vec::new();
        match st {
            Some(t) => {
                for char in t.trim().chars() {
                    match char {
                        '*' | '↓' => list.push(KeyAction::new(key, Down)),
                        '^' | '↑' => list.push(KeyAction::new(key, Up)),
                        _ => return Err(KeyError::new(&format!("Invalid transition character: `{}`", char))),
                    }
                }
            }
            None => {
                list.push(KeyAction::new(key, Down));
                list.push(KeyAction::new(key, Up));
            }
        }

        Ok(list)
    }
}

impl Display for KeyAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("{}{}", self.key, self.transition), f)
    }
}

impl FromStr for KeyAction {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str_expand(s)?[0])
    }
}

impl Serialize for KeyAction {
    serialize_to_string!();
}

impl<'de> Deserialize<'de> for KeyAction {
    deserialize_from_string!();
}

#[derive(Clone)]
pub struct KeyActionSequence {
    pub(crate) actions: Vec<KeyAction>,
}

impl KeyActionSequence {
    pub fn new(actions: Vec<KeyAction>) -> Self {
        Self { actions }
    }
}

impl PartialEq<Self> for KeyActionSequence {
    fn eq(&self, other: &Self) -> bool {
        self.actions == other.actions
    }
}

impl Eq for KeyActionSequence {}

impl Debug for KeyActionSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.actions)
    }
}

impl Display for KeyActionSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_joined!(f, &self.actions, " → ")
    }
}

impl FromStr for KeyActionSequence {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut actions = Vec::new();
        for part in s.split(|c| ['→', '>'].contains(&c)) {
            actions.append(&mut KeyAction::from_str_expand(part)?);
        }

        Ok(Self { actions })
    }
}

impl Serialize for KeyActionSequence {
    serialize_to_string!();
}

impl<'de> Deserialize<'de> for KeyActionSequence {
    deserialize_from_string!();
}

#[cfg(test)]
mod tests {
    use crate::key;
    use crate::keyboard::action::{KeyAction, KeyActionSequence};
    use crate::keyboard::key::{key_by_name, KEY_ENTER, KEY_SHIFT};
    use crate::keyboard::transition::KeyTransition::{Down, Up};
    use crate::utils::test::SerdeWrapper;
    use std::str::FromStr;

    #[macro_export]
    macro_rules! key_action {
        ($text:literal) => {
            $text.parse::<KeyAction>().unwrap()
        };
    }

    #[macro_export]
    macro_rules! key_action_seq {
        ($text:literal) => {
            $text.parse::<KeyActionSequence>().unwrap()
        };
    }

    // Key action

    #[test]
    fn test_action_display() {
        let actual = KeyAction {
            key: key!("ENTER"),
            transition: Down,
        };
        assert_eq!("ENTER↓", format!("{}", actual));

        let actual = KeyAction {
            key: key!("NUM_ENTER"),
            transition: Up,
        };
        assert_eq!("NUM_ENTER↑", format!("{}", actual));

        let actual = KeyAction {
            key: key!("ENTER"),
            transition: Down,
        };
        assert_eq!("[    ENTER↓]", format!("[{:>10}]", actual));
    }

    #[test]
    fn test_action_from_str() {
        assert_eq!(
            KeyAction {
                key: key!("ENTER"),
                transition: Down,
            },
            KeyAction::from_str("ENTER↓").unwrap()
        );

        assert_eq!(
            KeyAction {
                key: key!("F3"),
                transition: Down,
            },
            KeyAction::from_str("F3*").unwrap()
        );
    }

    #[test]
    fn test_action_from_str_expand() {
        assert_eq!(
            vec![KeyAction {
                key: key!("A"),
                transition: Down,
            }],
            KeyAction::from_str_expand("A↓").unwrap()
        );

        assert_eq!(
            vec![KeyAction {
                key: key!("B"),
                transition: Up,
            }],
            KeyAction::from_str_expand("B^").unwrap()
        );

        assert_eq!(
            vec![
                KeyAction {
                    key: key!("A"),
                    transition: Down,
                },
                KeyAction {
                    key: key!("A"),
                    transition: Up,
                }
            ],
            KeyAction::from_str_expand("A*^").unwrap()
        );

        assert_eq!(
            vec![
                KeyAction {
                    key: key!("A"),
                    transition: Up,
                },
                KeyAction {
                    key: key!("A"),
                    transition: Down,
                },
            ],
            KeyAction::from_str_expand("A^*").unwrap()
        );

        assert_eq!(
            vec![
                KeyAction {
                    key: key!("A"),
                    transition: Down,
                },
                KeyAction {
                    key: key!("A"),
                    transition: Up,
                }
            ],
            KeyAction::from_str_expand("A↓↑").unwrap()
        );

        assert_eq!(
            vec![
                KeyAction {
                    key: key!("A"),
                    transition: Down,
                },
                KeyAction {
                    key: key!("A"),
                    transition: Up,
                }
            ],
            KeyAction::from_str_expand("A").unwrap()
        );

        assert_eq!(
            vec![
                KeyAction {
                    key: key!("A"),
                    transition: Down,
                },
                KeyAction {
                    key: key!("A"),
                    transition: Down,
                },
                KeyAction {
                    key: key!("A"),
                    transition: Up,
                }
            ],
            KeyAction::from_str_expand("A↓↓↑").unwrap()
        );
    }

    #[test]
    fn test_action_serialize() {
        let source = SerdeWrapper::new(key_action!("A*"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
        assert_eq!(actual.value.key, key!("A"));
        assert_eq!(actual.value.transition, Down);

        let source = SerdeWrapper::new(key_action!("B^"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
        assert_eq!(actual.value.key, key!("B"));
        assert_eq!(actual.value.transition, Up);
    }

    // Key action sequence

    #[test]
    fn test_sequence_display() {
        let actual = key_action_seq!("ENTER↓ → SHIFT↑");

        assert_eq!("ENTER↓ → SHIFT↑", format!("{}", actual));
    }

    #[test]
    fn test_sequence_from_str() {
        assert_eq!(
            KeyActionSequence {
                actions: vec![KeyAction {
                    key: &KEY_ENTER,
                    transition: Down,
                }]
            },
            KeyActionSequence::from_str("ENTER↓").unwrap()
        );

        assert_eq!(
            KeyActionSequence {
                actions: vec![
                    KeyAction {
                        key: &KEY_ENTER,
                        transition: Down,
                    },
                    KeyAction {
                        key: &KEY_SHIFT,
                        transition: Up,
                    }
                ]
            },
            KeyActionSequence::from_str("ENTER↓ → SHIFT↑").unwrap()
        );

        assert_eq!(
            KeyActionSequence {
                actions: vec![
                    KeyAction {
                        key: &KEY_ENTER,
                        transition: Down,
                    },
                    KeyAction {
                        key: &KEY_ENTER,
                        transition: Up,
                    }
                ]
            },
            KeyActionSequence::from_str("ENTER↓↑").unwrap()
        );

        assert_eq!(
            KeyActionSequence {
                actions: vec![
                    KeyAction {
                        key: &KEY_ENTER,
                        transition: Down,
                    },
                    KeyAction {
                        key: &KEY_ENTER,
                        transition: Up,
                    },
                    KeyAction {
                        key: &KEY_SHIFT,
                        transition: Down,
                    },
                    KeyAction {
                        key: &KEY_SHIFT,
                        transition: Up,
                    }
                ]
            },
            KeyActionSequence::from_str("ENTER → SHIFT").unwrap()
        );
    }

    #[test]
    fn test_key_action_sequence_serialize() {
        let source = SerdeWrapper::new(key_action_seq!("ENTER↓ → SHIFT↓"));
        let text = toml::to_string(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }
}
