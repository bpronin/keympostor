use crate::keyboard::key_action::KeyActionSequence;
use crate::keyboard::key_trigger::KeyTrigger;
use crate::write_joined;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct KeyTransformRule {
    pub(crate) trigger: KeyTrigger,
    pub(crate) actions: KeyActionSequence,
}

impl Display for KeyTransformRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("{} : {}", self.trigger, self.actions), f)
    }
}

#[derive(Debug, Eq, PartialEq, Default)]
pub(crate) struct KeyTransformRules {
    pub(crate) items: Vec<KeyTransformRule>,
}

impl Display for KeyTransformRules {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_joined!(f, &self.items, "\n")
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct KeyTransformProfile {
    pub(crate) title: String,
    pub(crate) rules: KeyTransformRules,
}

impl KeyTransformProfile {
    pub(crate) fn load(path: &str) -> Result<Self, String> {
        toml::from_str(
            &fs::read_to_string(&path)
                .map_err(|e| format!("Unable to read {} file. {}", path, e))?,
        )
        .map_err(|e| format!("Unable to parse {}. {}", path, e))
    }

    pub(crate) fn save(&self, path: &str) -> Result<(), String> {
        fs::write(
            path,
            toml::to_string_pretty(self)
                .map_err(|e| format!("Unable to serialize {}. {}", path, e))?,
        )
        .map_err(|e| format!("Unable to write {} file. {}", path, e))
    }
}

impl Default for KeyTransformProfile {
    fn default() -> Self {
        Self {
            title: "No profile".to_string(),
            rules: Default::default(),
        }
    }
}

impl Display for KeyTransformProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.title, self.rules)
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key_action::KeyActionSequence;
    use crate::keyboard::key_trigger::KeyTrigger;
    use crate::keyboard::transform_rules::KeyTransformRule;
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

    #[macro_export]
    macro_rules! key_profile {
        ($text:expr) => {
            $text.parse::<KeyTransformProfile>().unwrap()
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
