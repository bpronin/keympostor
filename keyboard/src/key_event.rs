use crate::key::{Key, ScanCode, VirtualKey};
use crate::key_action::{KeyAction, KeyTransition};
use crate::key_const::{MAX_SCAN_CODE, MAX_VK_CODE};
use crate::key_transform_rule::KeyTransformRule;
use log::warn;
use std::fmt::{Display, Formatter};
use windows::Win32::UI::WindowsAndMessaging::{
    KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP,
};

/// A marker to detect self generated keyboard events.
/// Must be exactly `static` not `const`! Because of `const` ptrs may point at different addresses.
/// Content does not matter.
pub(crate) static SELF_EVENT_MARKER: &str = "banana";

#[derive(Debug, PartialEq)]
pub struct KeyEvent<'a> {
    kb: KBDLLHOOKSTRUCT,
    pub rule: Option<&'a KeyTransformRule>,
}

impl KeyEvent<'_> {
    pub(crate) fn new(kb: KBDLLHOOKSTRUCT) -> Self {
        Self { kb, rule: None }
    }

    pub fn time(&self) -> u32 {
        self.kb.time
    }

    pub fn action(&self) -> KeyAction {
        KeyAction {
            key: self.key(),
            transition: self.transition(),
        }
    }

    fn key(&self) -> Key {
        Key {
            vk_code: self.kb.vkCode as u8,
            scan_code: self.kb.scanCode as u8,
            is_ext_scan_code: self.kb.flags.contains(LLKHF_EXTENDED),
        }
    }

    fn transition(&self) -> KeyTransition {
        KeyTransition::from_bool(self.kb.flags.contains(LLKHF_UP))
    }

    pub fn is_injected(&self) -> bool {
        self.kb.flags.contains(LLKHF_INJECTED)
    }

    pub fn is_private(&self) -> bool {
        self.is_injected() && (self.kb.dwExtraInfo as *const u8 == SELF_EVENT_MARKER.as_ptr())
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

    pub fn fmt_kb(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "T:{:9} | VK: 0x{:02X} | SC: 0x{:02X} | F: 0b{:08b} | EX: 0x{:X}",
            self.kb.time, self.kb.vkCode, self.kb.scanCode, self.kb.flags.0, self.kb.dwExtraInfo,
        )
    }
}

impl Display for KeyEvent<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "T:{:9} | {:22} | {:16} | {:1} | {:3} | {:3}",
            self.time(),
            self.key().virtual_key(),
            self.key().scan_code(),
            self.transition(),
            if self.is_injected() { "INJ" } else { "" },
            if self.is_private() { "PRV" } else { "" },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::key_action::KeyTransition::Up;
    use crate::key_event::{KeyEvent, SELF_EVENT_MARKER};
    use windows::Win32::UI::WindowsAndMessaging::{
        KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP,
    };

    #[macro_export]
    macro_rules! key_event {
        ($vk_code:expr, $is_up:expr) => {
            KeyEvent::new(KBDLLHOOKSTRUCT {
                vkCode: $vk_code as u32,
                flags: if $is_up { LLKHF_UP } else { Default::default() },
                ..Default::default()
            })
        };
    }

    #[test]
    fn test_key_event() {
        let kb = KBDLLHOOKSTRUCT {
            vkCode: 0x0D,
            scanCode: 0x1C,
            flags: LLKHF_UP | LLKHF_INJECTED | LLKHF_EXTENDED,
            time: 1000,
            dwExtraInfo: SELF_EVENT_MARKER.as_ptr() as usize,
        };

        let actual = KeyEvent::new(kb);

        assert_eq!(1000, actual.time());
        assert_eq!("SC_NUM_ENTER", actual.key().scan_code().name);
        assert_eq!("VK_RETURN", actual.key().virtual_key().name);
        assert_eq!(Up, actual.transition());
        assert!(actual.is_private());
        assert!(actual.is_injected());
        assert!(actual.is_valid());
    }

    // #[test]
    // fn test_key_event_as_action() {
    //     let kb = KBDLLHOOKSTRUCT {
    //         vkCode: 0x0D,
    //         scanCode: 0x1C,
    //         flags: LLKHF_UP | LLKHF_INJECTED | LLKHF_EXTENDED,
    //         time: 1000,
    //         dwExtraInfo: SELF_EVENT_MARKER.as_ptr() as usize,
    //     };
    //
    //     let actual = KeyEvent::new(kb).as_virtual_key_action();
    //     let expected = KeyAction {
    //         keys: vec![VK(VirtualKey::from_code(0x0D).unwrap())],
    //         transition: Up,
    //     };
    //     assert_eq!(expected, actual);
    //
    //     let actual = KeyEvent::new(kb).as_scan_code_action();
    //     let expected = KeyAction {
    //         keys: vec![SC(ScanCode::from_code(0x1C, true).unwrap())],
    //         transition: Up,
    //     };
    //     assert_eq!(expected, actual);
    // }
}
