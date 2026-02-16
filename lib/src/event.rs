use crate::action::KeyAction;
use crate::error::KeyError;
use crate::key::Key;
use crate::key_err;
use crate::modifiers::KeyModifiers::All;
use crate::rules::KeyTransformRule;
use crate::state::KeyboardState;
use crate::transition::KeyTransition;
use crate::transition::KeyTransition::{Down, Up};
use crate::trigger::KeyTrigger;
use std::fmt::{Display, Formatter};
use windows::Win32::UI::WindowsAndMessaging::{
    KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP, LLMHF_INJECTED,
    LLMHF_LOWER_IL_INJECTED, MSLLHOOKSTRUCT, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN,
    WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN,
    WM_XBUTTONUP,
};

pub(crate) static SELF_EVENT_MARKER: usize = 497298395;

#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent {
    pub action: KeyAction,
    pub modifiers: KeyboardState,
    pub rule: Option<KeyTransformRule>,
    pub time: u32,
    pub is_injected: bool,
    pub is_private: bool,
}

impl KeyEvent {
    pub(crate) fn from_key_input(
        input: KBDLLHOOKSTRUCT,
        keyboard_state: &KeyboardState,
    ) -> KeyEvent {
        Self {
            action: KeyAction {
                key: Key::from_code(
                    input.vkCode as u8,
                    input.scanCode as u8,
                    input.flags.contains(LLKHF_EXTENDED),
                ),
                transition: KeyTransition::from_bool(!input.flags.contains(LLKHF_UP)),
            },
            modifiers: keyboard_state.clone(),
            is_injected: input.flags.contains(LLKHF_INJECTED),
            is_private: input.dwExtraInfo == SELF_EVENT_MARKER,
            time: input.time,
            rule: None,
        }
    }

    pub(crate) fn from_mouse_input(
        msg: u32,
        input: MSLLHOOKSTRUCT,
        keyboard_state: &KeyboardState,
    ) -> Result<KeyEvent, KeyError> {
        let event = match msg {
            WM_LBUTTONDOWN => Self::button_event(Key::LeftButton, Down, input, keyboard_state),
            WM_LBUTTONUP => Self::button_event(Key::LeftButton, Up, input, keyboard_state),
            WM_RBUTTONDOWN => Self::button_event(Key::RightButton, Down, input, keyboard_state),
            WM_RBUTTONUP => Self::button_event(Key::RightButton, Up, input, keyboard_state),
            WM_MBUTTONDOWN => Self::button_event(Key::MiddleButton, Down, input, keyboard_state),
            WM_MBUTTONUP => Self::button_event(Key::MiddleButton, Up, input, keyboard_state),
            WM_XBUTTONDOWN => Self::x_button_event(Down, input, keyboard_state)?,
            WM_XBUTTONUP => Self::x_button_event(Up, input, keyboard_state)?,
            WM_MOUSEWHEEL => Self::wheel_event(Key::WheelY, input, keyboard_state),
            WM_MOUSEHWHEEL => Self::wheel_event(Key::WheelX, input, keyboard_state),
            _ => return key_err!("Unsupported mouse event: `{}`", msg),
        };

        Ok(event)
    }

    fn wheel_event(key: Key, input: MSLLHOOKSTRUCT, keyboard_state: &KeyboardState) -> KeyEvent {
        let d = (input.mouseData >> 16) as i16;
        Self::mouse_event(
            KeyAction::new(key, KeyTransition::from_bool(d < 0)),
            input,
            keyboard_state,
        )
    }

    fn button_event(
        key: Key,
        transition: KeyTransition,
        input: MSLLHOOKSTRUCT,
        keyboard_state: &KeyboardState,
    ) -> KeyEvent {
        Self::mouse_event(KeyAction::new(key, transition), input, keyboard_state)
    }

    fn x_button_event(
        transition: KeyTransition,
        input: MSLLHOOKSTRUCT,
        keyboard_state: &KeyboardState,
    ) -> Result<KeyEvent, KeyError> {
        let key = match (input.mouseData >> 16) as u16 {
            1 => Key::Xbutton1,
            2 => Key::Xbutton2,
            b => {
                return key_err!("Unsupported mouse x-button: `{b}`");
            }
        };
        Ok(Self::mouse_event(
            KeyAction::new(key, transition),
            input,
            keyboard_state,
        ))
    }

    fn mouse_event(
        action: KeyAction,
        input: MSLLHOOKSTRUCT,
        keyboard_state: &KeyboardState,
    ) -> KeyEvent {
        Self {
            action,
            modifiers: keyboard_state.clone(),
            is_injected: (input.flags & (LLMHF_INJECTED | LLMHF_LOWER_IL_INJECTED)) != 0,
            is_private: input.dwExtraInfo == SELF_EVENT_MARKER,
            time: input.time,
            rule: None,
        }
    }

    pub fn as_trigger(&self) -> KeyTrigger {
        KeyTrigger {
            action: self.action,
            modifiers: All(self.modifiers),
        }
    }
}

impl Display for KeyEvent {
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
    use std::str::FromStr;
use crate::event::KeyEvent;
    use crate::kb_state;
    use crate::state::KeyboardState;

    #[macro_export]
    macro_rules! key_event {
        ($action:literal, $state:expr) => {
            KeyEvent {
                action: $action.parse().unwrap(),
                modifiers: $state.clone(),
                time: 0,
                is_injected: false,
                is_private: false,
                rule: None,
            }
        };
    }

    #[test]
    fn test_key_event_display() {
        let keyboard_state = kb_state!("LEFT_SHIFT");
        let event = key_event!("A↓", &keyboard_state);

        assert_eq!(format!("{}", event), "[LEFT_SHIFT] A↓ T:000000000  ");
    }
}
