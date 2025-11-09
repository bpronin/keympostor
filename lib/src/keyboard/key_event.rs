use crate::keyboard::key_action::KeyAction;
use crate::keyboard::key_modifiers::KeyModifiersState;
use crate::keyboard::transform_rules::KeyTransformRule;
use std::fmt::{Display, Formatter};

/// A marker to detect self-generated keyboard events.
/// Must be exactly `static` not `const`! Because of `const` ptrs may point at different addresses.
/// Content does not matter.
pub(crate) static SELF_EVENT_MARKER: &str = "banana";

#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent<'a> {
    pub action: KeyAction,
    pub modifiers: KeyModifiersState,
    pub rule: Option<&'a KeyTransformRule>,
    pub time: u32,
    pub is_injected: bool,
    pub is_private: bool,
}

impl Display for KeyEvent<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:8}] {:20} | {:22} | {:16} | {:1} | {:3} | {:3} | T:{:9} |",
            self.modifiers,
            self.action.key,
            self.action.key.virtual_key(),
            self.action.key.scan_code(),
            self.action.transition,
            if self.is_injected { "INJ" } else { "" },
            if self.is_private { "PRV" } else { "" },
            self.time,
        )
    }
}

#[cfg(test)]
mod tests {

    #[macro_export]
    macro_rules! key_event {
        ($action:literal, $state:expr) => {
            KeyEvent {
                action: $action.parse().unwrap(),
                modifiers: KeyModifiersState::from($state),
                time: 0,
                is_injected: false,
                is_private: false,
                rule: None,
            }
        };
    }
}
