use crate::action::KeyAction;
use crate::error::KeyError;
use crate::key::{
    KEY_LEFT_BUTTON, KEY_MIDDLE_BUTTON, KEY_MOUSE_X, KEY_MOUSE_Y, KEY_RIGHT_BUTTON, KEY_WHEEL_X,
    KEY_WHEEL_Y, KEY_XBUTTON1, KEY_XBUTTON2, Key, key_by_code,
};
use crate::key_err;
use crate::modifiers::ModifierKeys;
use crate::rules::KeyTransformRule;
use crate::state::KeyboardState;
use crate::transition::KeyTransition;
use crate::transition::KeyTransition::{Down, Up};
use std::fmt::{Display, Formatter};
use windows::Win32::UI::WindowsAndMessaging::{
    KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP, LLMHF_INJECTED,
    LLMHF_LOWER_IL_INJECTED, MSLLHOOKSTRUCT, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN,
    WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN,
    WM_XBUTTONUP,
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
    pub distance: Option<u32>, /* for mouse events only */
}

impl<'a> KeyEvent<'a> {
    pub(crate) fn new_key_event(input: KBDLLHOOKSTRUCT, keyboard_state: &KeyboardState) -> KeyEvent<'a> {
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
            distance: None,
        }
    }

    pub(crate) fn new_mouse_event(
        msg: u32,
        input: MSLLHOOKSTRUCT,
        keyboard_state: &KeyboardState,
    ) -> Result<KeyEvent<'a>, KeyError> {
        let event = match msg {
            WM_LBUTTONDOWN => Self::button_event(&KEY_LEFT_BUTTON, Down, input, keyboard_state),
            WM_LBUTTONUP => Self::button_event(&KEY_LEFT_BUTTON, Up, input, keyboard_state),
            WM_RBUTTONDOWN => Self::button_event(&KEY_RIGHT_BUTTON, Down, input, keyboard_state),
            WM_RBUTTONUP => Self::button_event(&KEY_RIGHT_BUTTON, Up, input, keyboard_state),
            WM_MBUTTONDOWN => Self::button_event(&KEY_MIDDLE_BUTTON, Down, input, keyboard_state),
            WM_MBUTTONUP => Self::button_event(&KEY_MIDDLE_BUTTON, Up, input, keyboard_state),
            WM_XBUTTONDOWN => Self::x_button_event(Down, input, keyboard_state)?,
            WM_XBUTTONUP => Self::x_button_event(Up, input, keyboard_state)?,
            WM_MOUSEWHEEL => Self::wheel_event(&KEY_WHEEL_Y, input, keyboard_state),
            WM_MOUSEHWHEEL => Self::wheel_event(&KEY_WHEEL_X, input, keyboard_state),
            _ => return key_err!("Unsupported mouse event: `{}`", msg),
        };

        Ok(event)
    }

    pub(crate) fn new_mouse_move_events(
        input: MSLLHOOKSTRUCT,
        dx: i32,
        dy: i32,
        keyboard_state: &KeyboardState,
    ) -> (KeyEvent<'a>, KeyEvent<'a>) {
        (
            Self::mouse_move_event(&KEY_MOUSE_X, dx, input, keyboard_state),
            Self::mouse_move_event(&KEY_MOUSE_Y, dy, input, keyboard_state),
        )
    }

    fn mouse_move_event(
        key: &'static Key,
        delta: i32,
        input: MSLLHOOKSTRUCT,
        keyboard_state: &KeyboardState,
    ) -> KeyEvent<'a> {
        Self::mouse_event(
            KeyAction {
                key,
                transition: KeyTransition::from_bool(delta > 0),
            },
            Some(delta.abs() as u32),
            input,
            keyboard_state,
        )
    }

    fn wheel_event(
        key: &'static Key,
        input: MSLLHOOKSTRUCT,
        keyboard_state: &KeyboardState,
    ) -> KeyEvent<'a> {
        let d = (input.mouseData >> 16) as i16;
        Self::mouse_event(
            KeyAction::new(key, KeyTransition::from_bool(d < 0)),
            Some(d.abs() as u32),
            input,
            keyboard_state,
        )
    }

    fn button_event(
        key: &'static Key,
        transition: KeyTransition,
        input: MSLLHOOKSTRUCT,
        keyboard_state: &KeyboardState,
    ) -> KeyEvent<'a> {
        Self::mouse_event(KeyAction::new(key, transition), None, input, keyboard_state)
    }

    fn x_button_event(
        transition: KeyTransition,
        input: MSLLHOOKSTRUCT,
        keyboard_state: &KeyboardState,
    ) -> Result<KeyEvent<'a>, KeyError> {
        let key = match (input.mouseData >> 16) as u16 {
            1 => &KEY_XBUTTON1,
            2 => &KEY_XBUTTON2,
            b => {
                return key_err!("Unsupported mouse x-button: `{b}`");
            }
        };
        Ok(Self::mouse_event(
            KeyAction::new(key, transition),
            None,
            input,
            keyboard_state,
        ))
    }

    fn mouse_event(
        action: KeyAction,
        distance: Option<u32>,
        input: MSLLHOOKSTRUCT,
        keyboard_state: &KeyboardState,
    ) -> KeyEvent<'a> {
        Self {
            action,
            modifiers: ModifierKeys::from(keyboard_state),
            is_injected: (input.flags & (LLMHF_INJECTED | LLMHF_LOWER_IL_INJECTED)) != 0,
            is_private: input.dwExtraInfo == SELF_EVENT_MARKER,
            time: input.time,
            rule: None,
            distance,
        }
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
    use crate::event::KeyEvent;
    use crate::modifiers::ModifierKeys;
    use crate::state::KeyboardState;
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
                distance: None,
            }
        };
    }

    #[test]
    fn test_key_event_display() {
        let mut keyboard_state = KeyboardState::new();
        keyboard_state.set(VK_LSHIFT.0 as u8, true);
        let event = key_event!("A↓", &keyboard_state);

        assert_eq!(format!("{}", event), "[LEFT_SHIFT] A↓ T:000000000  ");
    }
}
