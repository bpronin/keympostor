use crate::keyboard::key_action::KeyAction;
use crate::keyboard::key_modifiers::KeyboardState;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct KeyTrigger {
    pub(crate) action: KeyAction,
    pub(crate) state: KeyboardState,
}

impl Display for KeyTrigger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.state, self.action)
    }
}

#[cfg(test)]
mod tests {
    use crate::key_action;
    use crate::keyboard::key_modifiers::KeyboardState::{All, Any};
    use crate::keyboard::key_modifiers::KM_LSHIFT;
    use crate::keyboard::key_modifiers::KM_NONE;
    use crate::keyboard::key_trigger::KeyAction;
    use crate::keyboard::key_trigger::KeyTrigger;

    #[macro_export]
    macro_rules! key_trigger {
        ($text:literal) => {
            $text.parse::<KeyTrigger>().unwrap()
        };
    }

    #[test]
    fn test_key_trigger_display() {
        assert_eq!(
            "[LEFT_SHIFT]A↓",
            KeyTrigger {
                action: key_action!("A↓"),
                state: All(KM_LSHIFT),
            }
            .to_string()
        );

        assert_eq!(
            "[]A↓",
            KeyTrigger {
                action: key_action!("A↓"),
                state: All(KM_NONE),
            }
            .to_string()
        );

        assert_eq!(
            "[*]A↓",
            KeyTrigger {
                action: key_action!("A↓"),
                state: Any,
            }
            .to_string()
        );
    }
}
