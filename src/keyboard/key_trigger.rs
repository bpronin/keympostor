use crate::keyboard::key_action::KeyAction;
use crate::keyboard::key_modifiers::{KeyModifiers, KM_ALL, KM_NONE};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct KeyTrigger {
    pub(crate) action: KeyAction,
    pub(crate) modifiers: KeyModifiers,
}

impl Display for KeyTrigger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.modifiers != KM_NONE {
            write!(f, "[{}]", self.modifiers)?;
        }
        write!(f, "{}", self.action)
    }
}

impl FromStr for KeyTrigger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix('[') {
            let mut parts = s.split(']');
            Ok(Self {
                modifiers: parts.next().unwrap().parse()?,
                action: parts.next().unwrap().parse()?,
            })
        } else {
            Ok(Self {
                action: s.parse()?,
                modifiers: KM_ALL,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::keyboard::key_trigger::KeyAction;
    use crate::keyboard::key_trigger::KeyModifiers;
    use crate::keyboard::key_trigger::KeyTrigger;
    use crate::{key_action, key_mod};
    use crate::keyboard::key_modifiers::{KM_ALL, KM_NONE};

    #[macro_export]
    macro_rules! key_trigger {
        ($text:literal) => {
            $text.parse::<KeyTrigger>().unwrap()
        };
    }

    #[test]
    fn test_key_trigger_parse() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                modifiers: key_mod!("LEFT_SHIFT"),
            },
            KeyTrigger::from_str("[LEFT_SHIFT] A*").unwrap()
        );
        
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                modifiers: KM_NONE,
            },
            KeyTrigger::from_str("[] A*").unwrap()
        );
        
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                modifiers: KM_ALL,
            },
            KeyTrigger::from_str("A*").unwrap()
        );
    }
}
