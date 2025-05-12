use crate::key_action::KeyAction;
use crate::key_modifiers::{KeyModifiers, KM_NONE};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyTrigger {
    pub action: KeyAction,
    #[serde(default = "default_modifiers")]
    pub modifiers: KeyModifiers,
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
                modifiers: KM_NONE,
            })
        }
    }
}

fn default_modifiers() -> KeyModifiers {
    KM_NONE
}

#[cfg(test)]
mod tests {
    use crate::key_trigger::KeyAction;
    use crate::key_trigger::KeyModifiers;
    use crate::key_trigger::KeyTrigger;
    use crate::{key_act, key_mod};

    #[macro_export]
    macro_rules! key_trig {
        ($text:literal) => {
            $text.parse::<KeyTrigger>().unwrap()
        };
    }

    #[test]
    fn test_parse() {
        let expected = KeyTrigger {
            action: key_act!("A*"),
            modifiers: key_mod!("SHIFT"),
        };
        
        assert_eq!(expected, key_trig!("[SHIFT]A*"));
    }
}
