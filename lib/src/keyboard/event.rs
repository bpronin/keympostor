use crate::keyboard::action::KeyAction;
use crate::keyboard::modifiers::ModifierKeys;
use crate::keyboard::rules::KeyTransformRule;
use std::fmt::{Display, Formatter};
use windows::Win32::Foundation::LPARAM;

/// A marker to detect self-generated keyboard events.
/// Must be exactly `static` not `const`! Because of `const` ptrs may point at different addresses.
/// Content does not matter.
pub(crate) static SELF_EVENT_MARKER: &str = "banana";

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
    pub fn from_l_param(l_param: isize) -> &'a KeyEvent<'a> {
        unsafe { &*(l_param as *const KeyEvent) }
    }
}

impl Into<LPARAM> for KeyEvent<'_> {
    fn into(self) -> LPARAM {
        LPARAM(&self as *const KeyEvent as isize)
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
