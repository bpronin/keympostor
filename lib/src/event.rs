use crate::action::KeyAction;
use crate::key::Key;
use crate::modifiers::KeyModifiers::All;
use crate::rules::KeyTransformRule;
use crate::state::KeyboardState;
use crate::transition::KeyTransition;
use crate::transition::KeyTransition::{Down, Up};
use crate::trigger::KeyTrigger;
use log::warn;
use std::fmt::{Display, Formatter};
use windows::Win32::UI::WindowsAndMessaging::{
    KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP, LLMHF_INJECTED,
    LLMHF_LOWER_IL_INJECTED, MSLLHOOKSTRUCT, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN,
    WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN,
    WM_XBUTTONUP,
};
use Key::{LeftButton, MiddleButton, RightButton, WheelX, WheelY};

pub(crate) static SELF_EVENT_MARKER: usize = 497298395;

#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent {
    pub trigger: KeyTrigger,
    pub rule: Option<KeyTransformRule>,
    pub time: u32,
    pub is_injected: bool,
    pub is_private: bool,
}

impl KeyEvent {
    pub(crate) fn from_kbd_input(input: KBDLLHOOKSTRUCT, keyboard_state: KeyboardState) -> Self {
        Self {
            time: input.time,
            is_injected: input.flags.contains(LLKHF_INJECTED),
            is_private: input.dwExtraInfo == SELF_EVENT_MARKER,
            trigger: KeyTrigger {
                action: build_action_from_kbd_input(input),
                modifiers: All(keyboard_state),
            },
            rule: None,
        }
    }

    pub(crate) fn from_mouse_input(
        msg: u32,
        input: MSLLHOOKSTRUCT,
        keyboard_state: KeyboardState,
    ) -> Self {
        Self {
            time: input.time,
            is_injected: (input.flags & (LLMHF_INJECTED | LLMHF_LOWER_IL_INJECTED)) != 0,
            is_private: input.dwExtraInfo == SELF_EVENT_MARKER,
            trigger: KeyTrigger {
                action: build_action_from_mouse_input(msg, input),
                modifiers: All(keyboard_state),
            },
            rule: None,
        }
    }
}

impl Display for KeyEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} T:{:09} {} {}",
            self.trigger,
            self.time,
            if self.is_injected { "INJ" } else { "" },
            if self.is_private { "PRV" } else { "" },
        )
    }
}

pub(crate) fn build_action_from_kbd_input(input: KBDLLHOOKSTRUCT) -> KeyAction {
    KeyAction {
        key: Key::from_code(
            input.vkCode as u8,
            input.scanCode as u8,
            input.flags.contains(LLKHF_EXTENDED),
        ),
        transition: KeyTransition::from_bool(!input.flags.contains(LLKHF_UP)),
    }
}

pub(crate) fn build_action_from_mouse_input(msg: u32, input: MSLLHOOKSTRUCT) -> KeyAction {
    match msg {
        WM_LBUTTONDOWN => KeyAction::new(LeftButton, Down),
        WM_LBUTTONUP => KeyAction::new(LeftButton, Up),
        WM_RBUTTONDOWN => KeyAction::new(RightButton, Down),
        WM_RBUTTONUP => KeyAction::new(RightButton, Up),
        WM_MBUTTONDOWN => KeyAction::new(MiddleButton, Down),
        WM_MBUTTONUP => KeyAction::new(MiddleButton, Up),
        WM_XBUTTONDOWN => KeyAction::new(build_mouse_x_button_key(input), Down),
        WM_XBUTTONUP => KeyAction::new(build_mouse_x_button_key(input), Up),
        WM_MOUSEWHEEL => KeyAction::new(WheelY, build_mouse_wheel_transition(input)),
        WM_MOUSEHWHEEL => KeyAction::new(WheelX, build_mouse_wheel_transition(input)),
        _ => panic!("Illegal mouse message: `{}`", msg),
    }
}

#[inline(always)]
// #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn build_mouse_wheel_transition(input: MSLLHOOKSTRUCT) -> KeyTransition {
    let delta = (input.mouseData >> 16) as i16;
    KeyTransition::from_bool(delta < 0)
}

#[inline(always)]
fn build_mouse_x_button_key(input: MSLLHOOKSTRUCT) -> Key {
    match (input.mouseData >> 16) as u16 {
        1 => Key::Xbutton1,
        2 => Key::Xbutton2,
        b => {
            warn!("Unsupported mouse x-button: `{b}`");
            Key::Xbutton1
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::action::KeyAction;
    use crate::event::KeyEvent;
    use crate::key::Key;
    use crate::modifiers::KeyModifiers;
    use crate::state::tests::kb_state_from_keys;
    use crate::trigger::KeyTrigger;
    use std::str::FromStr;

    #[macro_export]
    macro_rules! key_event {
        ($action:literal, $state:expr) => {
            KeyEvent {
                trigger: KeyTrigger {
                    action: KeyAction::from_str($action).unwrap(),
                    modifiers: KeyModifiers::All($state.clone()),
                },
                time: 0,
                is_injected: false,
                is_private: false,
                rule: None,
            }
        };
    }

    #[test]
    fn test_key_event_display() {
        let state = kb_state_from_keys(&[Key::LeftShift]);
        let event = key_event!("A↓", state);

        assert_eq!(format!("{}", event), "[LEFT_SHIFT] A↓ T:000000000  ");
    }
}
