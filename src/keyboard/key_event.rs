use crate::keyboard::key_trigger::KeyTrigger;
use crate::keyboard::transform_rules::KeyTransformRule;
use std::fmt::{Display, Formatter};
use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, LLKHF_INJECTED};

/// A marker to detect self generated keyboard events.
/// Must be exactly `static` not `const`! Because of `const` ptrs may point at different addresses.
/// Content does not matter.
pub(crate) static SELF_EVENT_MARKER: &str = "banana";

#[derive(Debug, PartialEq)]
pub(crate) struct KeyEvent<'a> {
    pub(crate) trigger: KeyTrigger,
    pub(crate) rule: Option<&'a KeyTransformRule>,
    pub(crate) time: u32,
    pub(crate) is_injected: bool,
    pub(crate) is_private: bool,
}

impl KeyEvent<'_> {
    pub(crate) fn new(input: &KBDLLHOOKSTRUCT, keyboard_state: [u8; 256]) -> Self {
        Self {
            trigger: KeyTrigger::from_keyboard_input(input, keyboard_state),
            rule: None,
            time: input.time,
            is_injected: input.flags.contains(LLKHF_INJECTED),
            is_private: input.dwExtraInfo as *const u8 == SELF_EVENT_MARKER.as_ptr(),
        }
    }
}

impl Display for KeyEvent<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:8}] {:20} | {:22} | {:16} | {:1} | {:3} | {:3} | T:{:9} |",
            self.trigger.modifiers,
            self.trigger.action.key,
            self.trigger.action.key.virtual_key(),
            self.trigger.action.key.scan_code(),
            self.trigger.action.transition,
            if self.is_injected { "INJ" } else { "" },
            if self.is_private { "PRV" } else { "" },
            self.time,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key_action::KeyTransition::Up;
    use crate::keyboard::key_event::{KeyEvent, SELF_EVENT_MARKER};
    use windows::Win32::UI::WindowsAndMessaging::{
        KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP,
    };

    #[allow(dead_code)]
    pub(crate) fn fmt_kb_input(input: &KBDLLHOOKSTRUCT) -> String {
        format!(
            "T:{:9} | VK: 0x{:02X} | SC: 0x{:02X} | F: 0b{:08b} | EX: 0x{:X}",
            input.time, input.vkCode, input.scanCode, input.flags.0, input.dwExtraInfo,
        )
    }

    #[macro_export]
    macro_rules! key_event {
        ($action:literal, $modifiers:expr) => {
            KeyEvent {
                trigger: KeyTrigger {
                    action: $action.parse().unwrap(),
                    modifiers: $modifiers,
                },
                time: 0,
                is_injected: false,
                is_private: false,
                rule: None,
            }
        };
    }

    #[test]
    fn test_key_event_basics() {
        let actual = KeyEvent::new(
            &KBDLLHOOKSTRUCT {
                vkCode: 0x0D,
                scanCode: 0x1C,
                flags: LLKHF_UP | LLKHF_INJECTED | LLKHF_EXTENDED,
                time: 1000,
                dwExtraInfo: SELF_EVENT_MARKER.as_ptr() as usize,
            },
            [0; 256],
        );

        assert_eq!(1000, actual.time);
        assert_eq!("SC_NUM_ENTER", actual.trigger.action.key.scan_code().name);
        assert_eq!("VK_RETURN", actual.trigger.action.key.virtual_key().name);
        assert_eq!(Up, actual.trigger.action.transition);
        assert!(actual.is_private);
        assert!(actual.is_injected);
    }

    // #[test]
    // fn test_key_event_from_trigger() {
    //     init_logger();
    //
    //     let actual = KeyEvent::from_trigger(&key_trigger!("A↓"));
    //
    //     assert_eq!(0, actual.time());
    //     assert_eq!("SC_A", actual.key().scan_code().name);
    //     assert_eq!("VK_A", actual.key().virtual_key().name);
    //     assert_eq!(Down, actual.transition());
    //     // assert_not!(actual.is_valid());
    //     assert_not!(actual.is_private());
    //     assert_not!(actual.is_injected());
    // }
    //
    // #[test]
    // fn test_key_event_trigger() {
    //     assert_eq!(key_trigger!("A↓"), key_event!("A↓", KM_NONE).trigger);
    // }
}
