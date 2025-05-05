use crate::key::{KeyCode, ScanCode, VirtualKey};
use crate::key_event::{KeyTransition, SELF_KEY_EVENT_MARKER};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
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

    fn create_virtual_key_input(&self, virtual_key: &VirtualKey) -> INPUT {
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

    fn create_scancode_input(&self, scan_code: &ScanCode) -> INPUT {
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

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyActionSequence {
    actions: Vec<KeyAction>,
}

impl KeyActionSequence {
    fn create_input(&self) -> Vec<INPUT> {
        self.actions.iter().map(|a| a.create_input()).collect()
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
    pub fn trigger(&self) -> &KeyAction {
        &self.source.actions[0]
    }

    pub fn modifiers(&self) -> Option<&[KeyAction]> {
        self.source.actions.get(1..self.source.actions.len() - 1)
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::UI::Input::KeyboardAndMouse::{KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP};
    use crate::key::KeyCode::SC;
    use crate::key::{KeyCode, ScanCode, VirtualKey};
    use crate::key_action::{KeyAction, KeyActionPattern, KeyActionSequence, KeyTransformRule};
    use crate::key_event::KeyTransition::{Down, Up};
    use KeyCode::VK;

    #[test]
    fn test_key_action_serialize() {
        let source = KeyAction {
            key: VK(VirtualKey::by_name("VK_RETURN").unwrap()),
            transition: Down,
        };
        let json = serde_json::to_string_pretty(&source).unwrap();

        println!("{}", json);

        let actual = serde_json::from_str::<KeyAction>(&json).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_action_sequence_serialize() {
        let source = KeyActionSequence {
            actions: vec![
                KeyAction {
                    key: VK(VirtualKey::by_name("VK_RETURN").unwrap()),
                    transition: Down,
                },
                KeyAction {
                    key: VK(VirtualKey::by_name("VK_SHIFT").unwrap()),
                    transition: Down,
                },
            ],
        };

        let json = serde_json::to_string_pretty(&source).unwrap();

        println!("{}", json);

        let actual = serde_json::from_str::<KeyActionSequence>(&json).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_action_sequence_create_input() {
        let source = KeyActionSequence {
            actions: vec![
                KeyAction {
                    key: VK(VirtualKey::by_name("VK_RETURN").unwrap()),
                    transition: Down,
                },
                KeyAction {
                    key: SC(ScanCode::by_name("SC_NUM_ENTER").unwrap()),
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
        assert!(!unsafe { input[0].Anonymous.ki.dwFlags.contains(KEYEVENTF_EXTENDEDKEY) } );
        assert!(!unsafe { input[0].Anonymous.ki.dwFlags.contains(KEYEVENTF_KEYUP) } );
        
        let SC(sc) = source.actions[1].key else {
            panic!("Not an SC")
        };
        assert_eq!(sc.value, unsafe { input[1].Anonymous.ki.wScan } as u8);
        assert!(unsafe { input[1].Anonymous.ki.dwFlags.contains(KEYEVENTF_EXTENDEDKEY) } );
        assert!(unsafe { input[1].Anonymous.ki.dwFlags.contains(KEYEVENTF_KEYUP) } );
    }

    #[test]
    fn test_key_action_pattern_serialize() {
        let source = KeyActionPattern {
            sequence: vec![
                KeyActionSequence {
                    actions: vec![
                        KeyAction {
                            key: VK(VirtualKey::by_name("VK_RETURN").unwrap()),
                            transition: Down,
                        },
                        KeyAction {
                            key: VK(VirtualKey::by_name("VK_SHIFT").unwrap()),
                            transition: Down,
                        },
                    ],
                },
                KeyActionSequence {
                    actions: vec![KeyAction {
                        key: SC(ScanCode::by_name("SC_ENTER").unwrap()),
                        transition: Down,
                    }],
                },
            ],
        };

        let json = serde_json::to_string_pretty(&source).unwrap();

        println!("{}", json);

        let actual = serde_json::from_str::<KeyActionPattern>(&json).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_transform_rule_serialize() {
        let source = KeyTransformRule {
            source: KeyActionSequence {
                actions: vec![
                    KeyAction {
                        key: VK(VirtualKey::by_name("VK_RETURN").unwrap()),
                        transition: Down,
                    },
                    KeyAction {
                        key: VK(VirtualKey::by_name("VK_SHIFT").unwrap()),
                        transition: Down,
                    },
                ],
            },
            target: KeyActionSequence {
                actions: vec![KeyAction {
                    key: SC(ScanCode::by_name("SC_ENTER").unwrap()),
                    transition: Down,
                }],
            },
        };

        let json = serde_json::to_string_pretty(&source).unwrap();

        println!("{}", json);

        let actual = serde_json::from_str::<KeyTransformRule>(&json).unwrap();
        assert_eq!(source, actual);
    }
}
