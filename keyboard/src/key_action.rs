use crate::key::{KeyCode, ScanCode, VirtualKey};
use crate::key_event::{KeyTransition, SELF_KEY_EVENT_MARKER};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VIRTUAL_KEY,
};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyAction {
    pub key: KeyCode,
    pub transition: KeyTransition,
}

impl KeyAction {
    fn create_input(&self) -> INPUT {
        match self.key {
            KeyCode::VK(vk) => self.create_virtual_key_input(vk),
            KeyCode::SC(sc) => self.create_scancode_input(sc),
        }
    }

    fn create_virtual_key_input(&self, virtual_key: VirtualKey) -> INPUT {
        let mut flags = KEYBD_EVENT_FLAGS::default();
        if self.transition.is_up() {
            flags |= KEYEVENTF_KEYUP
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(virtual_key.value as u16),
                    dwFlags: flags,
                    dwExtraInfo: SELF_KEY_EVENT_MARKER.as_ptr() as usize,
                    ..Default::default()
                },
            },
        }
    }

    fn create_scancode_input(&self, scan_code: ScanCode) -> INPUT {
        let mut flags = KEYEVENTF_SCANCODE;
        if scan_code.is_extended {
            flags |= KEYEVENTF_EXTENDEDKEY
        }
        if self.transition.is_up() {
            flags |= KEYEVENTF_KEYUP;
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wScan: scan_code.ext_value(),
                    dwFlags: flags,
                    dwExtraInfo: SELF_KEY_EVENT_MARKER.as_ptr() as usize,
                    ..Default::default()
                },
            },
        }
    }
}

impl Display for KeyAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.key, self.transition)
    }
}

impl KeyActionSequence {
    fn create_input(&self) -> Vec<INPUT> {
        self.actions.iter().map(|a| a.create_input()).collect()
    }
}

impl FromStr for KeyAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let suf = trimmed
            .chars()
            .last()
            .expect(&format!("Error parsing key action. String is empty. `{s}`"));
        let key = trimmed
            .strip_suffix(suf)
            .expect(&format!("Invalid key action suffix: `{suf}`."));
        Ok(Self {
            key: key.parse()?,
            transition: suf.to_string().parse()?,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyActionSequence {
    actions: Vec<KeyAction>,
}

#[macro_export]
macro_rules! write_joined {
    ($dst:expr, $src:expr, $separator:expr) => {{
        let mut first = true;
        for item in $src {
            if !first {
                write!($dst, $separator)?;
            }
            write!($dst, "{}", item)?;
            first = false;
        }
        Ok(())
    }};
}

impl Display for KeyActionSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_joined!(f, &self.actions, " → ")
    }
}

impl FromStr for KeyActionSequence {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let actions = s
            .split(|c| "→>".contains(c))
            .map(str::parse)
            .collect::<Result<_, _>>()?;

        Ok(Self { actions })
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyActionPattern {
    sequence: Vec<KeyActionSequence>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyTransformRule {
    pub source: KeyActionSequence,
    pub target: KeyActionSequence,
}

impl KeyTransformRule {
    // pub fn trigger(&self) -> &KeyAction {
    //     &self.source.actions[0]
    // }

    // pub fn modifiers(&self) -> Option<&[KeyAction]> {
    //     self.source.actions.get(1..self.source.actions.len() - 1)
    // }
}

impl FromStr for KeyTransformRule {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(":");
        Ok(Self {
            source: KeyActionSequence::from_str(split.next().unwrap())?,
            target: KeyActionSequence::from_str(split.next().unwrap())?,
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
    // pub title: &'static str,
    pub title: String,
    pub rules: Vec<KeyTransformRule>,
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
        let rules = split.map(str::parse).collect::<Result<_, _>>()?;
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
    use crate::key_action::{
        KeyAction, KeyActionPattern, KeyActionSequence, KeyTransformProfile, KeyTransformRule,
    };
    use crate::key_event::KeyTransition::{Down, Up};
    use std::fs;
    use std::str::FromStr;
    use windows::Win32::UI::Input::KeyboardAndMouse::{KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP};
    use KeyCode::VK;

    #[test]
    fn test_key_action_display() {
        let actual = KeyAction {
            key: VK(VirtualKey::from_name("VK_RETURN").unwrap()),
            transition: Down,
        };
        assert_eq!("VK_RETURN↓", format!("{}", actual));

        let actual = KeyAction {
            key: SC(ScanCode::from_name("SC_ENTER").unwrap()),
            transition: Up,
        };
        assert_eq!("SC_ENTER↑", format!("{}", actual));
    }

    #[test]
    fn test_key_action_serialize() {
        let source = KeyAction {
            key: VK(VirtualKey::from_name("VK_RETURN").unwrap()),
            transition: Down,
        };
        let json = serde_json::to_string_pretty(&source).unwrap();

        // dbg!(&json);

        let actual = serde_json::from_str::<KeyAction>(&json).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_action_sequence_display() {
        let actual = KeyActionSequence {
            actions: vec![
                KeyAction {
                    key: VK(VirtualKey::from_name("VK_RETURN").unwrap()),
                    transition: Down,
                },
                KeyAction {
                    key: VK(VirtualKey::from_name("VK_SHIFT").unwrap()),
                    transition: Up,
                },
            ],
        };

        assert_eq!("VK_RETURN↓ → VK_SHIFT↑", format!("{}", actual));
    }

    #[test]
    fn test_key_action_sequence_serialize() {
        let source = KeyActionSequence {
            actions: vec![
                KeyAction {
                    key: VK(VirtualKey::from_name("VK_RETURN").unwrap()),
                    transition: Down,
                },
                KeyAction {
                    key: VK(VirtualKey::from_name("VK_SHIFT").unwrap()),
                    transition: Down,
                },
            ],
        };

        let json = serde_json::to_string_pretty(&source).unwrap();

        // dbg!(&json);

        let actual = serde_json::from_str::<KeyActionSequence>(&json).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_action_sequence_create_input() {
        let source = KeyActionSequence {
            actions: vec![
                KeyAction {
                    key: VK(VirtualKey::from_name("VK_RETURN").unwrap()),
                    transition: Down,
                },
                KeyAction {
                    key: SC(ScanCode::from_name("SC_NUM_ENTER").unwrap()),
                    transition: Up,
                },
            ],
        };
        let input = source.create_input();

        assert_eq!(2, input.len());

        let VK(vk) = source.actions[0].key else {
            panic!("Not an VK")
        };
        assert_eq!(vk.value, unsafe { input[0].Anonymous.ki.wVk.0 } as u8);
        assert!(!unsafe {
            input[0]
                .Anonymous
                .ki
                .dwFlags
                .contains(KEYEVENTF_EXTENDEDKEY)
        });
        assert!(!unsafe { input[0].Anonymous.ki.dwFlags.contains(KEYEVENTF_KEYUP) });

        let SC(sc) = source.actions[1].key else {
            panic!("Not an SC")
        };
        assert_eq!(sc.value, unsafe { input[1].Anonymous.ki.wScan } as u8);
        assert!(unsafe {
            input[1]
                .Anonymous
                .ki
                .dwFlags
                .contains(KEYEVENTF_EXTENDEDKEY)
        });
        assert!(unsafe { input[1].Anonymous.ki.dwFlags.contains(KEYEVENTF_KEYUP) });
    }

    #[test]
    fn test_key_action_pattern_serialize() {
        let source = KeyActionPattern {
            sequence: vec![
                KeyActionSequence {
                    actions: vec![
                        KeyAction {
                            key: VK(VirtualKey::from_name("VK_RETURN").unwrap()),
                            transition: Down,
                        },
                        KeyAction {
                            key: VK(VirtualKey::from_name("VK_SHIFT").unwrap()),
                            transition: Down,
                        },
                    ],
                },
                KeyActionSequence {
                    actions: vec![KeyAction {
                        key: SC(ScanCode::from_name("SC_ENTER").unwrap()),
                        transition: Down,
                    }],
                },
            ],
        };

        let json = serde_json::to_string_pretty(&source).unwrap();

        // dbg!(&json);

        let actual = serde_json::from_str::<KeyActionPattern>(&json).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_transform_rule_display() {
        let source = KeyTransformRule {
            source: KeyActionSequence {
                actions: vec![
                    KeyAction {
                        key: VK(VirtualKey::from_name("VK_RETURN").unwrap()),
                        transition: Down,
                    },
                    KeyAction {
                        key: VK(VirtualKey::from_name("VK_SHIFT").unwrap()),
                        transition: Down,
                    },
                ],
            },
            target: KeyActionSequence {
                actions: vec![KeyAction {
                    key: SC(ScanCode::from_name("SC_ENTER").unwrap()),
                    transition: Down,
                }],
            },
        };

        assert_eq!("VK_RETURN↓ → VK_SHIFT↓ : SC_ENTER↓", format!("{}", source));
    }

    #[test]
    fn test_key_transform_rule_parse() {
        let actual = KeyTransformRule::from_str("VK_RETURN↓ → VK_SHIFT↓ : SC_ENTER↓").unwrap();

        let expected = KeyTransformRule {
            source: KeyActionSequence {
                actions: vec![
                    KeyAction {
                        key: VK(VirtualKey::from_name("VK_RETURN").unwrap()),
                        transition: Down,
                    },
                    KeyAction {
                        key: VK(VirtualKey::from_name("VK_SHIFT").unwrap()),
                        transition: Down,
                    },
                ],
            },
            target: KeyActionSequence {
                actions: vec![KeyAction {
                    key: SC(ScanCode::from_name("SC_ENTER").unwrap()),
                    transition: Down,
                }],
            },
        };

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_transform_rule_serialize() {
        let source = KeyTransformRule {
            source: KeyActionSequence {
                actions: vec![
                    KeyAction {
                        key: VK(VirtualKey::from_name("VK_RETURN").unwrap()),
                        transition: Down,
                    },
                    KeyAction {
                        key: VK(VirtualKey::from_name("VK_SHIFT").unwrap()),
                        transition: Down,
                    },
                ],
            },
            target: KeyActionSequence {
                actions: vec![KeyAction {
                    key: SC(ScanCode::from_name("SC_ENTER").unwrap()),
                    transition: Down,
                }],
            },
        };

        let json = serde_json::to_string_pretty(&source).unwrap();

        // dbg!(&json);

        let actual = serde_json::from_str::<KeyTransformRule>(&json).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_transform_rules_serialize() {
        let json = fs::read_to_string("../test/profiles/test.json").unwrap();
        let actual: KeyTransformProfile = serde_json::from_str(&json).unwrap();

        // println!("{}", actual);
        // dbg!(&actual);

        let expected: KeyTransformProfile = "
            Test profile;
            SC_CAPS_LOCK↓ : SC_LEFT_WINDOWS↓ → SC_SPACE↓ → SC_SPACE↑ → SC_LEFT_WINDOWS↑;
            VK_SHIFT↓ → VK_CAPITAL↓ : VK_CAPITAL↓ → VK_CAPITAL↑;
            "
        .parse()
        .unwrap();

        // println!("{}", expected);

        assert_eq!(expected, actual);
    }
}
