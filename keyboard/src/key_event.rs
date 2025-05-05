use crate::key::{KeyCode, ScanCode, VirtualKey, MAX_SCAN_CODE, MAX_VK_CODE};
use crate::key_action::{KeyAction, KeyActionPattern};
use crate::key_event::KeyTransition::Up;
use log::warn;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use windows::Win32::UI::WindowsAndMessaging::{
    KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP,
};
use KeyCode::{SC, VK};
use KeyTransition::Down;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum KeyTransition {
    #[serde(alias = "UP", alias = "up")]
    Up,
    #[serde(alias = "DOWN", alias = "down")]
    Down,
}

impl KeyTransition {
    fn from_bool(up: bool) -> KeyTransition {
        if up { Up } else { Down }
    }

    pub fn is_up(&self) -> bool {
        matches!(*self, Up)
    }

    pub fn is_down(&self) -> bool {
        matches!(*self, Down)
    }
}

impl Default for KeyTransition {
    fn default() -> Self {
        Up
    }
}

impl Display for KeyTransition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Up => '↑',
                Down => '↓',
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct KeyEvent {
    kb: KBDLLHOOKSTRUCT,
}

impl KeyEvent {
    pub fn from_kb(kb: KBDLLHOOKSTRUCT) -> Self {
        Self { kb }
    }

    pub fn time(&self) -> u32 {
        self.kb.time
    }

    pub fn virtual_key(&self) -> &'static VirtualKey {
        VirtualKey::by_code(self.kb.vkCode as u8).unwrap()
    }

    pub fn scan_code(&self) -> &'static ScanCode {
        ScanCode::by_code(
            self.kb.scanCode as u8,
            self.kb.flags.contains(LLKHF_EXTENDED),
        )
        .unwrap()
    }

    pub fn as_virtual_key_action(&self) -> KeyAction {
        KeyAction {
            key: VK(self.virtual_key()),
            transition: self.transition(),
        }
    }

    pub fn as_scan_code_action(&self) -> KeyAction {
        KeyAction {
            key: SC(self.scan_code()),
            transition: self.transition(),
        }
    }

    pub fn transition(&self) -> KeyTransition {
        KeyTransition::from_bool(self.kb.flags.contains(LLKHF_UP))
    }

    pub fn is_injected(&self) -> bool {
        self.kb.flags.contains(LLKHF_INJECTED)
    }

    pub fn flags(&self) -> u32 {
        self.kb.flags.0
    }

    pub fn is_private(&self) -> bool {
        self.is_injected() && (self.kb.dwExtraInfo as *const u8 == SELF_KEY_EVENT_MARKER.as_ptr())
    }

    pub fn is_valid(&self) -> bool {
        if self.kb.scanCode > MAX_SCAN_CODE as u32 {
            warn!("Ignored invalid scan code: 0x{:02X}.", self.kb.scanCode);
            false
        } else if self.kb.vkCode > MAX_VK_CODE as u32 {
            warn!("Ignored invalid virtual key: 0x{:02X}.", self.kb.vkCode);
            false
        } else if self.kb.time == 0 {
            warn!("Ignored invalid time: {}.", self.kb.time);
            false
        } else {
            true
        }
    }
}

/// A marker to detect self generated keyboard events.
/// Must be exactly `static` not `const`! Because of `const` ptrs may point at different addresses.
/// Content does not matter.
static SELF_KEY_EVENT_MARKER: &str = "self";

#[cfg(test)]
mod tests {
    use crate::key::KeyCode::{SC, VK};
    use crate::key::{ScanCode, VirtualKey};
    use crate::key_action::KeyAction;
    use crate::key_event::KeyTransition::{Down, Up};
    use crate::key_event::{KeyEvent, KeyTransition, SELF_KEY_EVENT_MARKER};
    use windows::Win32::UI::WindowsAndMessaging::{
        KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP,
    };

    #[test]
    fn test_key_transition_display() {
        assert_eq!("↓", format!("{}", Down));
        assert_eq!("↑", format!("{}", Up));
    }

    #[test]
    fn test_key_transition_serialize() {
        let source = Down;
        let json = serde_json::to_string_pretty(&source).unwrap();

        println!("{}", json);

        let actual = serde_json::from_str::<KeyTransition>(&json).unwrap();
        assert_eq!(source, actual);

        let source = Up;
        let json = serde_json::to_string_pretty(&source).unwrap();

        println!("{}", json);

        let actual = serde_json::from_str::<KeyTransition>(&json).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_event() {
        let kb = KBDLLHOOKSTRUCT {
            vkCode: 0x0D,
            scanCode: 0x1C,
            flags: LLKHF_UP | LLKHF_INJECTED | LLKHF_EXTENDED,
            time: 1000,
            dwExtraInfo: SELF_KEY_EVENT_MARKER.as_ptr() as usize,
        };

        let actual = KeyEvent::from_kb(kb);

        assert_eq!("SC_NUM_ENTER", actual.scan_code().name);
        assert_eq!("VK_RETURN", actual.virtual_key().name);
        assert_eq!(1000, actual.time());
        assert_eq!(Up, actual.transition());
        assert_eq!(145, actual.flags());
        assert!(actual.is_private());
        assert!(actual.is_injected());
        assert!(actual.is_valid());
    }

    #[test]
    fn test_key_event_as_action() {
        let kb = KBDLLHOOKSTRUCT {
            vkCode: 0x0D,
            scanCode: 0x1C,
            flags: LLKHF_UP | LLKHF_INJECTED | LLKHF_EXTENDED,
            time: 1000,
            dwExtraInfo: SELF_KEY_EVENT_MARKER.as_ptr() as usize,
        };

        let actual = KeyEvent::from_kb(kb).as_virtual_key_action();
        let expected = KeyAction {
            key: VK(VirtualKey::by_code(0x0D).unwrap()),
            transition: Up,
        };
        assert_eq!(expected, actual);
        
        
        let actual = KeyEvent::from_kb(kb).as_scan_code_action();
        let expected = KeyAction {
            key: SC(ScanCode::by_code(0x1C, true).unwrap()),
            transition: Up,
        };
        assert_eq!(expected, actual);
    }
}
