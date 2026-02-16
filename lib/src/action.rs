use crate::error::KeyError;
use crate::key::Key;
use crate::key_error;
use crate::transition::KeyTransition;
use crate::transition::KeyTransition::{Down, Up};
use crate::{deserialize_from_string, key_err, serialize_to_string, write_joined};
use serde::Deserializer;
use serde::Serializer;
use serde::{de, Deserialize, Serialize};
use slice::Iter;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::slice;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct KeyAction {
    pub key: Key,
    pub transition: KeyTransition,
}

impl KeyAction {
    pub(crate) const fn new(key: Key, transition: KeyTransition) -> Self {
        Self { key, transition }
    }

    pub(crate) fn from_str_expand(s: &str) -> Result<Vec<Self>, KeyError> {
        let (sk, st) = match s.find(|c| ['^', '*', '↓', '↑'].contains(&c)) {
            Some(p) => (
                s.get(..p).ok_or(key_error!("Missing key part"))?,
                s.get(p..),
            ),
            None => (s, None),
        };

        let key = Key::from_str(sk.trim()).ok_or(key_error!("Invalid key part: `{sk}`"))?;

        let mut result = Vec::new();
        match st {
            Some(t) => {
                for char in t.trim().chars() {
                    match char {
                        '*' | '↓' => result.push(KeyAction::new(key, Down)),
                        '^' | '↑' => result.push(KeyAction::new(key, Up)),
                        _ => return key_err!("Invalid transition character: `{char}`"),
                    }
                }
            }
            None => {
                result.push(KeyAction::new(key, Down));
                result.push(KeyAction::new(key, Up));
            }
        }

        Ok(result)
    }
}

impl Display for KeyAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("{}{}", self.key.as_str(), self.transition), f)
    }
}

impl FromStr for KeyAction {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = Self::from_str_expand(s)?;
        if vec.len() > 1 {
            return key_err!("String must be exactly single action");
        }
        Ok(vec[0])
    }
}

impl Serialize for KeyAction {
    serialize_to_string!();
}

impl<'de> Deserialize<'de> for KeyAction {
    deserialize_from_string!();
}

#[derive(Clone, Eq)]
pub struct KeyActionSequence(Vec<KeyAction>);

impl KeyActionSequence {
    pub fn new(actions: Vec<KeyAction>) -> Self {
        Self(actions)
    }

    pub fn iter(&self) -> Iter<'_, KeyAction> {
        self.0.iter()
    }

    pub(crate) fn from_str_expand(s: &str) -> Result<Vec<Self>, KeyError> {
        let mut down_actions = Vec::new();
        let mut up_actions = Vec::new();

        let mut is_expanded = false;
        for part in s.split(|c| ['→', '>'].contains(&c)) {
            let actions = KeyAction::from_str_expand(part)?;
            down_actions.push(actions[0]);
            if actions.len() == 1 {
                up_actions.push(actions[0]);
            } else {
                up_actions.push(actions[1]);
                is_expanded = true;
            }
        }

        let mut list = Vec::new();
        list.push(KeyActionSequence::new(down_actions));
        if is_expanded {
            list.push(KeyActionSequence::new(up_actions))
        }

        Ok(list)
    }
}

impl PartialEq<Self> for KeyActionSequence {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Debug for KeyActionSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Display for KeyActionSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_joined!(f, &self.0, " → ")
    }
}

impl FromStr for KeyActionSequence {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = Self::from_str_expand(s)?;
        if vec.len() > 1 {
            return key_err!("String must be exactly single sequence");
        }
        Ok(vec[0].clone())
    }
}

impl Serialize for KeyActionSequence {
    serialize_to_string!();
}

impl<'de> Deserialize<'de> for KeyActionSequence {
    deserialize_from_string!();
}

#[macro_export]
macro_rules! key_action {
    ($text:literal) => {
        KeyAction::from_str($text).unwrap()
    };
}

#[macro_export]
macro_rules! key_action_seq {
    ($text:literal) => {
        KeyActionSequence::from_str($text).unwrap()
    };
}

#[cfg(test)]
mod tests {
    use crate::action::KeyAction;
    use crate::action::KeyActionSequence;
    use crate::key;
    use crate::key::Key;
    use crate::transition::KeyTransition::{Down, Up};
    use crate::utils::test::SerdeWrapper;
    use std::str::FromStr;

    // Key action

    #[test]
    fn test_key_action_display() {
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
    fn test_key_action_from_str() {
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
    fn test_key_action_from_str_expand() {
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
    fn test_key_action_serialize() {
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
    fn test_key_action_sequence_display() {
        let actual = key_action_seq!("ENTER↓ → SHIFT↑");

        assert_eq!("ENTER↓ → SHIFT↑", format!("{}", actual));
    }

    #[test]
    fn test_key_action_sequence_from_str_to_vec() {
        assert_eq!(
            vec![KeyActionSequence::new(vec![key_action!("A↓")]),],
            KeyActionSequence::from_str_expand("A↓").unwrap()
        );

        assert_eq!(
            vec![KeyActionSequence::new(vec![
                key_action!("A↓"),
                key_action!("B↑"),
                key_action!("C↓")
            ]),],
            KeyActionSequence::from_str_expand("A↓ → B↑ → C↓").unwrap()
        );
    }

    #[test]
    fn test_key_action_sequence_from_str_to_vec_expand() {
        assert_eq!(
            vec![key_action_seq!("A↓"), key_action_seq!("A↑")],
            KeyActionSequence::from_str_expand("A").unwrap()
        );

        assert_eq!(
            vec![key_action_seq!("A↓"), key_action_seq!("A↑")],
            KeyActionSequence::from_str_expand("A↓↑").unwrap()
        );

        assert_eq!(
            vec![key_action_seq!("A↓ → B↓"), key_action_seq!("A↑ → B↑")],
            KeyActionSequence::from_str_expand("A → B").unwrap()
        );

        assert_eq!(
            vec![
                key_action_seq!("A↓ → B↓ → C↓"),
                key_action_seq!("A↑ → B↑ → C↓")
            ],
            KeyActionSequence::from_str_expand("A → B → C↓").unwrap()
        );

        assert_eq!(
            vec![
                key_action_seq!("C↓ → A↓ → B↓"),
                key_action_seq!("C↓ → A↑ → B↑")
            ],
            KeyActionSequence::from_str_expand("C↓ → A → B").unwrap()
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
