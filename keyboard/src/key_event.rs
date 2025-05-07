use crate::key::{KeyCode, ScanCode, VirtualKey, MAX_SCAN_CODE, MAX_VK_CODE};
use crate::key_action::KeyAction;
use crate::key_action::KeyTransition;
use log::warn;
use windows::Win32::UI::WindowsAndMessaging::{
    KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP,
};
use KeyCode::{SC, VK};

#[derive(Debug, PartialEq)]
pub struct KeyEvent {
    pub kb: KBDLLHOOKSTRUCT,
}

impl KeyEvent {
    pub fn time(&self) -> u32 {
        self.kb.time
    }

    pub fn virtual_key(&self) -> &'static VirtualKey {
        VirtualKey::from_code(self.kb.vkCode as u8).unwrap()
    }

    pub fn scan_code(&self) -> &'static ScanCode {
        ScanCode::from_code(
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
pub static SELF_KEY_EVENT_MARKER: &str = "self";

#[cfg(test)]
mod tests {
    use crate::key::KeyCode::{SC, VK};
    use crate::key::{ScanCode, VirtualKey};
    use crate::key_action::KeyAction;
    use crate::key_action::KeyTransition::Up;
    use crate::key_event::{KeyEvent, SELF_KEY_EVENT_MARKER};
    use windows::Win32::UI::WindowsAndMessaging::{
        KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP,
    };

    #[macro_export]
    macro_rules! key_event {
        ($vk_code:expr, $is_up:expr) => {
            KeyEvent {
                kb: KBDLLHOOKSTRUCT {
                    vkCode: $vk_code as u32,
                    flags: if $is_up { LLKHF_UP } else { Default::default() },
                    ..Default::default()
                },
            }
        };
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

        let actual = KeyEvent { kb };

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

        let actual = KeyEvent { kb }.as_virtual_key_action();
        let expected = KeyAction {
            key: VK(VirtualKey::from_code(0x0D).unwrap()),
            transition: Up,
        };
        assert_eq!(expected, actual);

        let actual = KeyEvent { kb }.as_scan_code_action();
        let expected = KeyAction {
            key: SC(ScanCode::from_code(0x1C, true).unwrap()),
            transition: Up,
        };
        assert_eq!(expected, actual);
    }
}
