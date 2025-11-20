use crate::keyboard::error::KeyError;
use crate::keyboard::event::SELF_EVENT_MARKER;
use crate::keyboard::key::{key_by_name, Key, KEY_MOUSE, KEY_WHEEL};
use crate::keyboard::sc::ScanCode;
use crate::keyboard::transition::KeyTransition;
use crate::keyboard::transition::KeyTransition::{Distance, Down, Up};
use crate::keyboard::vk::VirtualKey;
use crate::{deserialize_from_string, serialize_to_string, write_joined};
use serde::Deserializer;
use serde::Serializer;
use serde::{de, Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MOUSEINPUT, VIRTUAL_KEY,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct KeyAction {
    pub key: &'static Key,
    pub transition: KeyTransition,
}

impl KeyAction {
    fn new(key: &'static Key, transition: KeyTransition) -> Self {
        Self { key, transition }
    }

    pub(crate) fn from_str_to_vec(s: &str) -> Result<Vec<Self>, KeyError> {
        let ts = s.trim();
        let mut list = Vec::with_capacity(2);

        if let Some(k) = ts.strip_suffix("*^") {
            let key = key_by_name(k)?;
            list.push(KeyAction::new(key, Down));
            list.push(KeyAction::new(key, Up));
        } else if let Some(k) = ts.strip_suffix("↓↑") {
            let key = key_by_name(k)?;
            list.push(KeyAction::new(key, Down));
            list.push(KeyAction::new(key, Up));
        } else if let Some(k) = ts.strip_suffix('*') {
            list.push(KeyAction::new(key_by_name(k)?, Down));
        } else if let Some(k) = ts.strip_suffix('↓') {
            list.push(KeyAction::new(key_by_name(k)?, Down));
        } else if let Some(k) = ts.strip_suffix('^') {
            list.push(KeyAction::new(key_by_name(k)?, Up));
        } else if let Some(k) = ts.strip_suffix('↑') {
            list.push(KeyAction::new(key_by_name(k)?, Up));
        } else if let Some((k, sd)) = ts.split_once('(') {
            let (sdx, sdy) = sd
                .strip_suffix(')')
                .ok_or(KeyError::new(
                    "Invalid mouse action: missing ')' at the end of the string",
                ))?
                .split_once(':')
                .ok_or(KeyError::new(
                    "Invalid mouse action: missing ':' in the string",
                ))?;
            let dx = sdx
                .trim()
                .parse()
                .map_err(|e| KeyError::new("Invalid dx value."))?;
            let dy = sdy
                .trim()
                .parse()
                .map_err(|e| KeyError::new("Invalid dy value."))?;
            list.push(KeyAction::new(key_by_name(k)?, Distance(dx, dy)));
        } else {
            let key = key_by_name(ts)?;
            list.push(KeyAction::new(key, Down));
            list.push(KeyAction::new(key, Up));
        }

        Ok(list)
    }

    fn into_key_input(self) -> INPUT {
        let virtual_key = VirtualKey::from(self.key);
        let scan_code = ScanCode::from(self.key);

        let mut flags = KEYEVENTF_SCANCODE;
        if scan_code.is_extended {
            flags |= KEYEVENTF_EXTENDEDKEY
        }
        if self.transition == Up {
            flags |= KEYEVENTF_KEYUP;
        }

        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(virtual_key.value as u16),
                    wScan: scan_code.ext_value(),
                    dwFlags: flags,
                    dwExtraInfo: SELF_EVENT_MARKER.as_ptr() as usize,
                    ..Default::default()
                },
            },
        }
    }

    fn into_mouse_input(self) -> INPUT {
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    // dx:self.distance
                    // dy:self.distance
                    //todo
                    dwExtraInfo: SELF_EVENT_MARKER.as_ptr() as usize,
                    ..Default::default()
                },
            },
        }
    }
}

impl Into<INPUT> for KeyAction {
    fn into(self) -> INPUT {
        if self.key == &KEY_MOUSE || self.key == &KEY_WHEEL {
            Self::into_mouse_input(self)
        } else {
            Self::into_key_input(self)
        }
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
        Ok(Self::from_str_to_vec(s)?[0])
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
    pub(crate) input: Vec<INPUT>,
}

impl KeyActionSequence {
    pub fn new(actions: Vec<KeyAction>) -> Self {
        let input = actions.iter().map(|a| (*a).into()).collect();
        Self { actions, input }
    }

    pub(crate) fn from_str_to_vec(s: &str) -> Result<Vec<Self>, KeyError> {
        let mut down_actions = Vec::new();
        let mut up_actions = Vec::new();

        let mut is_expanded = false;
        for part in s.split(|c| ['→', '>'].contains(&c)) {
            let actions = KeyAction::from_str_to_vec(part)?;
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
        Ok(Self::from_str_to_vec(s)?[0].clone())
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
    use crate::keyboard::action::{KeyAction, KeyActionSequence};
    use crate::keyboard::event::SELF_EVENT_MARKER;
    use crate::keyboard::key::key_by_name;
    use crate::keyboard::sc::ScanCode;
    use crate::keyboard::transition::KeyTransition::{Down, Up};
    use crate::utils::test::SerdeWrapper;
    use crate::{key, sc_key};
    use std::str::FromStr;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_KEYBOARD, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
        VK_RETURN,
    };

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
    fn test_key_action_create_input() {
        let actual: INPUT = key_action!("ENTER*").into();
        unsafe {
            assert_eq!(INPUT_KEYBOARD, actual.r#type);
            assert_eq!(VK_RETURN, actual.Anonymous.ki.wVk);
            assert_eq!(sc_key!("SC_ENTER").ext_value(), actual.Anonymous.ki.wScan);
            assert_eq!(KEYEVENTF_SCANCODE, actual.Anonymous.ki.dwFlags);
            assert_eq!(
                SELF_EVENT_MARKER.as_ptr(),
                actual.Anonymous.ki.dwExtraInfo as *const u8
            );
        };

        let actual: INPUT = key_action!("NUM_ENTER^").into();
        unsafe {
            assert_eq!(INPUT_KEYBOARD, actual.r#type);
            assert_eq!(VK_RETURN, actual.Anonymous.ki.wVk);
            assert_eq!(
                sc_key!("SC_NUM_ENTER").ext_value(),
                actual.Anonymous.ki.wScan
            );
            assert_eq!(
                KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
                actual.Anonymous.ki.dwFlags
            );
            assert_eq!(
                SELF_EVENT_MARKER.as_ptr(),
                actual.Anonymous.ki.dwExtraInfo as *const u8
            );
        };
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
            KeyAction::from_str_to_vec("A↓").unwrap()
        );

        assert_eq!(
            vec![KeyAction {
                key: key!("B"),
                transition: Up,
            }],
            KeyAction::from_str_to_vec("B^").unwrap()
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
            KeyAction::from_str_to_vec("A*^").unwrap()
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
            KeyAction::from_str_to_vec("A↓↑").unwrap()
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
            KeyAction::from_str_to_vec("A").unwrap()
        );
    }

    #[test]
    fn test_key_action_serialize() {
        let source = SerdeWrapper::new(key_action!("A*"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);

        let source = SerdeWrapper::new(key_action!("B^"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);

        let source = SerdeWrapper::new(key_action!("MOUSE(-20:10)"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
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
            KeyActionSequence::from_str_to_vec("A↓").unwrap()
        );

        assert_eq!(
            vec![KeyActionSequence::new(vec![
                key_action!("A↓"),
                key_action!("B↑"),
                key_action!("C↓")
            ]),],
            KeyActionSequence::from_str_to_vec("A↓ → B↑ → C↓").unwrap()
        );
    }

    #[test]
    fn test_key_action_sequence_from_str_to_vec_expand() {
        assert_eq!(
            vec![key_action_seq!("A↓"), key_action_seq!("A↑")],
            KeyActionSequence::from_str_to_vec("A").unwrap()
        );

        assert_eq!(
            vec![key_action_seq!("A↓"), key_action_seq!("A↑")],
            KeyActionSequence::from_str_to_vec("A↓↑").unwrap()
        );

        assert_eq!(
            vec![key_action_seq!("A↓ → B↓"), key_action_seq!("A↑ → B↑")],
            KeyActionSequence::from_str_to_vec("A → B").unwrap()
        );

        assert_eq!(
            vec![
                key_action_seq!("A↓ → B↓ → C↓"),
                key_action_seq!("A↑ → B↑ → C↓")
            ],
            KeyActionSequence::from_str_to_vec("A → B → C↓").unwrap()
        );

        assert_eq!(
            vec![
                key_action_seq!("C↓ → A↓ → B↓"),
                key_action_seq!("C↓ → A↑ → B↑")
            ],
            KeyActionSequence::from_str_to_vec("C↓ → A → B").unwrap()
        );
    }

    #[test]
    fn test_key_action_sequence_serialize() {
        let source = SerdeWrapper::new(key_action_seq!("ENTER↓ → SHIFT↓ → MOUSE(-20: 10)"));
        let text = toml::to_string(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }
}
