use crate::keyboard::key::Key;
use crate::keyboard::key_action::{KeyAction, KeyTransition};
use crate::keyboard::key_modifiers::KeyModifiers;
use crate::keyboard::transform_rules::KeyTransformRule;
use std::fmt::{Display, Formatter};
use windows::Win32::UI::WindowsAndMessaging::{
    KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP,
};

/// A marker to detect self generated keyboard events.
/// Must be exactly `static` not `const`! Because of `const` ptrs may point at different addresses.
/// Content does not matter.
pub(crate) static SELF_EVENT_MARKER: &str = "banana";

#[derive(Debug, PartialEq)]
pub(crate) struct KeyEvent<'a> {
    kb: KBDLLHOOKSTRUCT,
    pub(crate) rule: Option<&'a KeyTransformRule>,
    pub(crate) modifiers: KeyModifiers,
}

impl KeyEvent<'_> {
    pub(crate) fn new(kb: KBDLLHOOKSTRUCT, keyboard_state: [u8; 256]) -> Self {
        Self {
            kb,
            modifiers: KeyModifiers::from_keyboard_state(keyboard_state),
            rule: None,
        }
    }

    pub(crate) fn time(&self) -> u32 {
        self.kb.time
    }

    pub(crate) fn action(&self) -> KeyAction {
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

    pub(crate) fn is_injected(&self) -> bool {
        self.kb.flags.contains(LLKHF_INJECTED)
    }

    pub(crate) fn is_private(&self) -> bool {
        self.is_injected() && (self.kb.dwExtraInfo as *const u8 == SELF_EVENT_MARKER.as_ptr())
    }

    #[allow(dead_code)]
    pub(crate) fn fmt_kb(&self) -> String {
        format!(
            "T:{:9} | VK: 0x{:02X} | SC: 0x{:02X} | F: 0b{:08b} | EX: 0x{:X}",
            self.kb.time, self.kb.vkCode, self.kb.scanCode, self.kb.flags.0, self.kb.dwExtraInfo,
        )
    }
}

impl Display for KeyEvent<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:8}] {:20} | {:22} | {:16} | {:1} | {:3} | {:3} | T:{:9} |",
            self.modifiers,
            self.key(),
            self.key().virtual_key(),
            self.key().scan_code(),
            self.transition(),
            if self.is_injected() { "INJ" } else { "" },
            if self.is_private() { "PRV" } else { "" },
            self.time(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key_action::KeyAction;
    use crate::keyboard::key_action::KeyTransition::{Down, Up};
    use crate::keyboard::key_event::{KeyEvent, SELF_EVENT_MARKER};
    use crate::keyboard::key_modifiers::{KeyModifiers, KM_NONE};
    use crate::keyboard::tests::init_logger;
    use crate::{assert_not, key_action};
    use windows::Win32::UI::WindowsAndMessaging::{
        KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP,
    };

    impl KeyEvent<'_> {
        pub(crate) fn from_action(key_action: &KeyAction, modifiers: KeyModifiers) -> Self {
            let mut flags = Default::default();
            if key_action.transition == Up {
                flags |= LLKHF_UP
            };
            if key_action.key.is_ext_scan_code {
                flags |= LLKHF_EXTENDED
            };

            let kb = KBDLLHOOKSTRUCT {
                vkCode: key_action.key.vk_code as u32,
                scanCode: key_action.key.scan_code as u32,
                flags,
                ..Default::default()
            };

            Self {
                kb,
                rule: None,
                modifiers,
            }
        }
    }

    #[macro_export]
    macro_rules! key_event {
        ($action:literal, $modifiers:expr) => {
            KeyEvent::from_action(&$action.parse().unwrap(), $modifiers)
        };
    }

    #[test]
    fn test_key_event_basics() {
        let actual = KeyEvent::new(
            KBDLLHOOKSTRUCT {
                vkCode: 0x0D,
                scanCode: 0x1C,
                flags: LLKHF_UP | LLKHF_INJECTED | LLKHF_EXTENDED,
                time: 1000,
                dwExtraInfo: SELF_EVENT_MARKER.as_ptr() as usize,
            },
            [0; 256],
        );

        assert_eq!(1000, actual.time());
        assert_eq!("SC_NUM_ENTER", actual.key().scan_code().name);
        assert_eq!("VK_RETURN", actual.key().virtual_key().name);
        assert_eq!(Up, actual.transition());
        assert!(actual.is_private());
        assert!(actual.is_injected());
    }

    #[test]
    fn test_key_event_from_action() {
        init_logger();

        let actual = KeyEvent::from_action(&key_action!("A↓"), KM_NONE);

        assert_eq!(0, actual.time());
        assert_eq!("SC_A", actual.key().scan_code().name);
        assert_eq!("VK_A", actual.key().virtual_key().name);
        assert_eq!(Down, actual.transition());
        // assert_not!(actual.is_valid());
        assert_not!(actual.is_private());
        assert_not!(actual.is_injected());
    }

    #[test]
    fn test_key_event_action() {
        assert_eq!(key_action!("A↓"), key_event!("A↓", KM_NONE).action());
    }
}
