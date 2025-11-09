use crate::keyboard::action::KeyActionSequence;
use crate::keyboard::trigger::KeyTrigger;
use crate::write_joined;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyTransformRule {
    pub trigger: KeyTrigger,
    pub actions: KeyActionSequence,
}

impl Display for KeyTransformRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&format!("{} : {}", self.trigger, self.actions), f)
    }
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct KeyTransformRules {
    pub items: Vec<KeyTransformRule>,
}

impl Display for KeyTransformRules {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_joined!(f, &self.items, "\n")
    }
}

#[cfg(test)]
pub mod tests {
    use crate::keyboard::action::KeyActionSequence;
    use crate::keyboard::trigger::KeyTrigger;
    use crate::keyboard::rules::KeyTransformRule;
    use crate::{key_action_seq, key_trigger};

    #[macro_export]
    macro_rules! key_rule {
        ($text:literal) => {
            $text.parse::<KeyTransformRule>().unwrap()
        };
    }

    #[macro_export]
    macro_rules! key_rules {
        ($text:literal) => {
            $text.parse::<KeyTransformRules>().unwrap()
        };
    }

    #[test]
    fn test_key_transform_rule_display() {
        let actual = KeyTransformRule {
            trigger: key_trigger!("[LEFT_SHIFT] ENTER ↓"),
            actions: key_action_seq!("ENTER↓"),
        };

        assert_eq!(
            "|        [LEFT_SHIFT]ENTER↓ : ENTER↓|",
            format!("|{:>35}|", actual)
        );
    }

    #[test]
    fn test_key_transform_rule_trigger() {
        assert_eq!(
            key_trigger!("[LEFT_CTRL + LEFT_SHIFT] ENTER↓"),
            key_rule!("[LEFT_CTRL + LEFT_SHIFT] ENTER↓ : ENTER↓").trigger
        );
    }
}
