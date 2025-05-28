use crate::keyboard::key_action::KeyAction;
use crate::keyboard::key_modifiers::KeyModifiers;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyTrigger {
    pub action: KeyAction,
    pub modifiers: KeyModifiers,
}

impl Display for KeyTrigger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("{}{}", self.modifiers, self.action), f)
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key_modifiers::KeyModifiers::{All, Any};
    use crate::keyboard::key_modifiers::KM_LSHIFT;
    use crate::keyboard::key_modifiers::KM_NONE;
    use crate::keyboard::key_trigger::KeyAction;
    use crate::keyboard::key_trigger::KeyTrigger;
    use crate::key_action;

    #[macro_export]
    macro_rules! key_trigger {
        ($text:literal) => {
            $text.parse::<KeyTrigger>().unwrap()
        };
    }

    #[test]
    fn test_key_trigger_display() {
        let actual = KeyTrigger {
            action: key_action!("A↓"),
            modifiers: All(KM_LSHIFT),
        };
        assert_eq!("[LEFT_SHIFT]A↓", format!("{}", actual));

        let actual = KeyTrigger {
            action: key_action!("A↓"),
            modifiers: All(KM_NONE),
        };
        assert_eq!("[]A↓", format!("{}", actual));

        let actual = KeyTrigger {
            action: key_action!("A↓"),
            modifiers: Any,
        };
        assert_eq!("A↓", format!("{}", actual));

        let actual = KeyTrigger {
            action: key_action!("A↓"),
            modifiers: All(KM_LSHIFT),
        };
        assert_eq!("|      [LEFT_SHIFT]A↓|", format!("|{:>20}|", actual));
    }
}
