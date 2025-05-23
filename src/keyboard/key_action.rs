use crate::keyboard::key::Key;
use crate::keyboard::key_action::KeyTransition::{Down, Up};
use crate::keyboard::key_event::SELF_EVENT_MARKER;
use crate::write_joined;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
    KEYEVENTF_SCANCODE, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, LLKHF_UP};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) enum KeyTransition {
    #[serde(alias = "UP", alias = "up")]
    Up,
    #[serde(alias = "DOWN", alias = "down")]
    Down,
}

impl KeyTransition {
    pub(crate) fn from_keyboard_input(input: &KBDLLHOOKSTRUCT) -> Self {
        if input.flags.contains(LLKHF_UP) {
            Up
        } else {
            Down
        }
    }

    pub(crate) fn is_up(&self) -> bool {
        matches!(*self, Up)
    }
}

impl Default for KeyTransition {
    fn default() -> Self {
        Up
    }
}

impl Display for KeyTransition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Up => Display::fmt(&'↑', f),
            Down => Display::fmt(&'↓', f),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct KeyAction {
    pub(crate) key: Key,
    pub(crate) transition: KeyTransition,
}

impl KeyAction {
    pub(crate) fn from_keyboard_input(input: &KBDLLHOOKSTRUCT) -> Self {
        Self {
            key: Key::from_keyboard_input(input),
            transition: KeyTransition::from_keyboard_input(input),
        }
    }

    fn create_input(&self) -> INPUT {
        let virtual_key = self.key.virtual_key();
        let scan_code = self.key.scan_code();

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
                    wVk: VIRTUAL_KEY(virtual_key.value as u16),
                    wScan: scan_code.ext_value(),
                    dwFlags: flags,
                    dwExtraInfo: SELF_EVENT_MARKER.as_ptr() as usize,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct KeyActionSequence {
    pub(crate) actions: Vec<KeyAction>,
}

impl KeyActionSequence {
    pub(crate) fn create_input(&self) -> Vec<INPUT> {
        self.actions.iter().map(|a| a.create_input()).collect()
    }
}

impl Display for KeyActionSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_joined!(f, &self.actions, " → ")
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key::ScanCode;
    use crate::keyboard::key_action::Key;
    use crate::keyboard::key_action::KeyTransition::{Down, Up};
    use crate::keyboard::key_action::{KeyAction, KeyActionSequence, KeyTransition};
    use crate::keyboard::key_event::SELF_EVENT_MARKER;
    use crate::{assert_not, key, sc_key};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT_KEYBOARD, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VK_RETURN,
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

    #[test]
    fn test_key_transition_display() {
        assert_eq!("↓", format!("{}", Down));
        assert_eq!("↑", format!("{}", Up));
    }

    #[test]
    fn test_key_transition_basics() {
        assert_eq!(Up, KeyTransition::default());
        assert_eq!(Up, if true { Up } else { Down });
        assert!(Up.is_up());
        assert_not!(Down.is_up());
    }

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
    }

    #[test]
    fn test_key_action_create_input() {
        let actual = key_action!("ENTER*").create_input();
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

        let actual = key_action!("NUM_ENTER^").create_input();
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
    fn test_key_action_sequence_display() {
        let actual = key_action_seq!("ENTER↓ → SHIFT↑");

        assert_eq!("ENTER↓ → SHIFT↑", format!("{}", actual));
    }
}
