use crate::key_action::KeyTransition::{Down, Up};
use crate::key_hook::SELF_MARKER;
use crate::key_id::{KeyIdentifier, ScanCode, VirtualKey};
use crate::key_modifier::KeyModifiers;
use crate::util::slices_equal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, LLKHF_UP};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum KeyTransition {
    #[serde(alias = "UP", alias = "up")]
    Up,
    #[serde(alias = "DOWN", alias = "down")]
    Down,
}

impl KeyTransition {
    pub(crate) fn from_kb(kb: &KBDLLHOOKSTRUCT) -> KeyTransition {
        if kb.flags.contains(LLKHF_UP) {
            Up
        } else {
            Down
        }
    }

    pub(crate) fn is_up(self) -> bool {
        match self {
            Up => true,
            Down => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub(crate) struct KeyAction {
    pub(crate) key: KeyIdentifier,
    pub(crate) transition: KeyTransition,
    pub(crate) modifiers: Option<KeyModifiers>,
}

impl KeyAction {
    fn create_input(&self) -> INPUT {
        if let Some(scancode) = self.key.scancode {
            Self::create_scancode_input(scancode, self.transition)
        } else {
            let virtual_key = self.key.virtual_key.unwrap();
            Self::create_virtual_key_input(virtual_key, self.transition)
        }
    }

    fn create_virtual_key_input(virtual_key: &VirtualKey, transition: KeyTransition) -> INPUT {
        let mut flags = KEYBD_EVENT_FLAGS::default();
        if transition.is_up() {
            flags |= KEYEVENTF_KEYUP
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(virtual_key.value as u16),
                    dwFlags: flags,
                    dwExtraInfo: SELF_MARKER.as_ptr() as usize,
                    ..Default::default()
                },
            },
        }
    }

    fn create_scancode_input(scancode: &ScanCode, transition: KeyTransition) -> INPUT {
        let mut flags = KEYEVENTF_SCANCODE;
        if scancode.is_extended {
            flags |= KEYEVENTF_EXTENDEDKEY
        }
        if transition.is_up() {
            flags |= KEYEVENTF_KEYUP;
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wScan: scancode.ext_value(),
                    dwFlags: flags,
                    dwExtraInfo: SELF_MARKER.as_ptr() as usize,
                    ..Default::default()
                },
            },
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct KeyActionSequence {
    pub(crate) actions: Vec<KeyAction>,
}

impl KeyActionSequence {
    pub(crate) fn from(actions: Vec<KeyAction>) -> Self {
        Self { actions }
    }

    pub(crate) fn send(&self) {
        let inputs: Vec<INPUT> = self.actions.iter().map(|a| a.create_input()).collect();
        unsafe { SendInput(inputs.as_slice(), size_of::<INPUT>() as i32) };
    }
}

impl PartialEq for KeyActionSequence {
    fn eq(&self, other: &KeyActionSequence) -> bool {
        slices_equal(&self.actions, &other.actions)
    }
}

impl Serialize for KeyActionSequence {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Ok(self.actions.serialize(serializer)?)
    }
}

impl<'de> Deserialize<'de> for KeyActionSequence {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from(Vec::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {

    //     #[test]
    //     fn parse_key_actions_by_name() {
    //         let actual = KeyActionSequence::parse(&[
    //             "VK_A UP".to_string(),
    //             "VK_B DOWN".to_string(),
    //             "SC_A UP".to_string(),
    //             "SC_B DOWN".to_string(),
    //         ])
    //             .unwrap();
    //
    //         let expected = KeyActionSequence::from(vec![
    //             KeyAction {
    //                 key_id: AsVirtualKey(VirtualKey::by_name("VK_A").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsVirtualKey(VirtualKey::by_name("VK_B").unwrap()),
    //                 transition: Down,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_A").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_B").unwrap()),
    //                 transition: Down,
    //             },
    //         ]);
    //
    //         assert_eq!(actual, expected)
    //     }
    //
    //     #[test]
    //     fn parse_key_actions_vk_by_code() {
    //         let actual = KeyActionSequence::parse(&[
    //             "VK_0x1C UP".to_string(),
    //             "VK_0x30 DOWN".to_string(),
    //             "SC_0xE01C UP".to_string(),
    //             "SC_0xE030 DOWN".to_string(),
    //         ])
    //             .unwrap();
    //
    //         let expected = KeyActionSequence::from(vec![
    //             KeyAction {
    //                 key_id: AsVirtualKey(VirtualKey::by_name("VK_CONVERT").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsVirtualKey(VirtualKey::by_name("VK_0").unwrap()),
    //                 transition: Down,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_NUM_ENTER").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_VOL_UP").unwrap()),
    //                 transition: Down,
    //             },
    //         ]);
    //
    //         assert_eq!(actual, expected)
    //     }
    //
    //     #[test]
    //     fn parse_key_actions_symbols() {
    //         let actual = KeyActionSequence::parse(&[
    //             "] UP".to_string(),
    //             "= UP".to_string(),
    //             "+ UP".to_string(),
    //             "\\ DOWN".to_string(),
    //         ])
    //             .unwrap();
    //
    //         let expected = KeyActionSequence::from(vec![
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_R_BRACKET").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_EQUALITY").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_EQUALITY").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_BACKSLASH").unwrap()),
    //                 transition: Down,
    //             },
    //         ]);
    //
    //         assert_eq!(actual, expected)
    //     }
    //
    //     #[test]
    //     fn parse_key_actions_no_key_id_type() {
    //         let actual = KeyActionSequence::parse(&[
    //             "A UP".to_string(),
    //             "ENTER UP".to_string(),
    //             "RETURN UP".to_string(),
    //             "0x0D UP".to_string(),
    //             "0xE01C UP".to_string(),
    //         ])
    //             .unwrap();
    //
    //         let expected = KeyActionSequence::from(vec![
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_A").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_ENTER").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsVirtualKey(VirtualKey::by_name("VK_RETURN").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsVirtualKey(VirtualKey::by_name("VK_RETURN").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_NUM_ENTER").unwrap()),
    //                 transition: Up,
    //             },
    //         ]);
    //
    //         assert_eq!(actual, expected)
    //     }
    //
    //     #[test]
    //     #[should_panic]
    //     fn parse_key_actions_no_transition() {
    //         let text = ["VK_A".to_string()];
    //         let sequence = KeyActionSequence::parse(&text).unwrap();
    //         dbg!(sequence);
    //         todo!();
    //     }
    //
}
