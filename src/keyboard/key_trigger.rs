use crate::keyboard::key_action::KeyAction;
use crate::keyboard::key_modifiers::{KeyModifiers, KM_ALL};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use windows::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct KeyTrigger {
    pub(crate) action: KeyAction,
    pub(crate) modifiers: KeyModifiers,
}

impl KeyTrigger {
    pub(crate) fn from_keyboard_input(input: &KBDLLHOOKSTRUCT, keyboard_state: [u8; 256]) -> Self {
        Self {
            action: KeyAction::from_keyboard_input(input),
            modifiers: KeyModifiers::from_keyboard_state(keyboard_state),
        }
    }
}

impl Display for KeyTrigger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.modifiers != KM_ALL {
            write!(f, "[{}]", self.modifiers)?;
        }
        write!(f, "{}", self.action)
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key_modifiers::{KM_ALL, KM_NONE};
    use crate::keyboard::key_trigger::KeyAction;
    use crate::keyboard::key_trigger::KeyModifiers;
    use crate::keyboard::key_trigger::KeyTrigger;
    use crate::{key_action, key_mod};
    
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
                modifiers: key_mod!("LEFT_SHIFT"),
            }
            .to_string()
        );

        assert_eq!(
            "[]A↓",
            KeyTrigger {
                action: key_action!("A↓"),
                modifiers: KM_NONE,
            }
            .to_string()
        );

        assert_eq!(
            "A↓",
            KeyTrigger {
                action: key_action!("A↓"),
                modifiers: KM_ALL,
            }
            .to_string()
        );
    }
}
