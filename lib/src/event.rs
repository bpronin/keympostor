use crate::trigger::KeyTrigger;
use crate::utils::if_else;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent {
    pub trigger: KeyTrigger,
    pub time: u32,
    pub is_injected: bool,
    pub is_private: bool,
}

impl Display for KeyEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.trigger,
            if_else(self.is_injected, "INJECTED", ""),
            if_else(self.is_private, "PRIVATE", ""),
        )
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
