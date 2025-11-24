use crate::key_err;
use crate::keyboard::action::KeyAction;
use crate::keyboard::error::KeyError;
use crate::keyboard::key::{key_by_code, Key, KEY_XBUTTON1, KEY_XBUTTON2};
use crate::keyboard::modifiers::ModifierKeys;
use crate::keyboard::rules::KeyTransformRule;
use crate::keyboard::transition::KeyTransition;
use std::fmt::{Display, Formatter};
use windows::Win32::UI::WindowsAndMessaging::{
    KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP, LLMHF_INJECTED,
    LLMHF_LOWER_IL_INJECTED, MSLLHOOKSTRUCT,
};

pub(crate) static SELF_EVENT_MARKER: usize = 497298395;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct KeyEvent<'a> {
    pub action: KeyAction,
    pub modifiers: ModifierKeys,
    pub rule: Option<&'a KeyTransformRule>,
    pub time: u32,
    pub is_injected: bool,
    pub is_private: bool,
}

impl<'a> KeyEvent<'a> {

    pub fn new_key_event(input: KBDLLHOOKSTRUCT, keyboard_state: &[bool; 256]) -> KeyEvent<'a> {
        Self {
            action: KeyAction {
                key: key_by_code(
                    input.vkCode as u8,
                    input.scanCode as u8,
                    input.flags.contains(LLKHF_EXTENDED),
                ),
                transition: KeyTransition::from_bool(!input.flags.contains(LLKHF_UP)),
            },
            modifiers: ModifierKeys::from(keyboard_state),
            is_injected: input.flags.contains(LLKHF_INJECTED),
            is_private: input.dwExtraInfo == SELF_EVENT_MARKER,
            time: input.time,
            rule: None,
        }
    }

    pub(crate) fn new_mouse_event(
        input: &MSLLHOOKSTRUCT,
        key: &'static Key,
        transition: KeyTransition,
        keyboard_state: &[bool; 256]
    ) -> KeyEvent<'a> {
        Self {
            action: KeyAction { key, transition },
            modifiers: ModifierKeys::from(keyboard_state),
            is_injected: (input.flags & (LLMHF_INJECTED | LLMHF_LOWER_IL_INJECTED)) != 0,
            is_private: input.dwExtraInfo == SELF_EVENT_MARKER,
            time: input.time,
            rule: None,
        }
    }

    pub fn new_x_button_event(
        input: &MSLLHOOKSTRUCT,
        transition: KeyTransition,
        keyboard_state: &[bool; 256]
    ) -> Result<KeyEvent<'a>, KeyError> {
        let key = match (input.mouseData >> 16) as u16 {
            1 => &KEY_XBUTTON1,
            2 => &KEY_XBUTTON2,
            b => {
                return key_err!("Unsupported mouse x-button: `{b}`");
            }
        };
        Ok(Self::new_mouse_event(input, key, transition, keyboard_state))
    }
}

impl Display for KeyEvent<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} T:{:09} {} {}",
            self.modifiers,
            self.action,
            self.time,
            if self.is_injected { "INJ" } else { "" },
            if self.is_private { "PRV" } else { "" },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::event::KeyEvent;
    use crate::keyboard::modifiers::ModifierKeys;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_LSHIFT;

    #[macro_export]
    macro_rules! key_event {
        ($action:literal, $state:expr) => {
            KeyEvent {
                action: $action.parse().unwrap(),
                modifiers: ModifierKeys::from($state),
                time: 0,
                is_injected: false,
                is_private: false,
                rule: None,
            }
        };
    }

    #[test]
    fn test_key_event_display() {
        let mut keyboard_state = [false; 256];
        keyboard_state[VK_LSHIFT.0 as usize] = true;
        let event = key_event!("A↓", &keyboard_state);

        assert_eq!(format!("{}", event), "[LEFT_SHIFT] A↓ T:000000000  ");
    }
}
