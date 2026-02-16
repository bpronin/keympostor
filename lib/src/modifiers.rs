use crate::error::KeyError;
use crate::state::KeyboardState;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum KeyModifiers {
    Any,
    All(KeyboardState),
}

impl Display for KeyModifiers {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Self::All(modifiers) = self {
            write!(f, "[{}]", modifiers)
        } else {
            Ok(())
        }
    }
}

impl FromStr for KeyModifiers {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self::Any);
        }

        let part = s.trim().trim_start_matches('[').trim_end_matches(']');
        Ok(Self::All(KeyboardState::from_str(part)?))
    }
}

#[cfg(test)]
mod tests {
    use crate::key::Key;
    use crate::modifiers::KeyModifiers;
    use crate::state::tests::kb_state_from_keys;
    use crate::state::KeyboardState;
    use crate::utils::test::SerdeWrapper;
    use std::str::FromStr;

    #[test]
    fn test_key_modifiers_to_str() {
        assert_eq!(
            "[RIGHT_WIN + LEFT_SHIFT + RIGHT_SHIFT]",
            KeyModifiers::All(kb_state_from_keys(&[
                Key::LeftShift,
                Key::RightShift,
                Key::RightWin
            ])).to_string()
        );

        assert_eq!(
            "[]",
            KeyModifiers::All(KeyboardState::default()).to_string()
        );

        assert_eq!(
            "",
            KeyModifiers::Any.to_string()
        );
    }

    #[test]
    fn test_key_modifiers_from_str() {
        assert_eq!(
            Ok(KeyModifiers::All(kb_state_from_keys(&[
                Key::LeftShift,
                Key::RightShift,
                Key::RightWin
            ]))),
            KeyModifiers::from_str("[LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN]")
        );

        assert_eq!(
            Ok(KeyModifiers::All(KeyboardState::default())),
            KeyModifiers::from_str("[]")
        );

        assert_eq!(
            Ok(KeyModifiers::Any),
            KeyModifiers::from_str("")
        );
    }

    #[test]
    fn test_key_modifiers_from_str_fails() {
        assert!(KeyModifiers::from_str("BANANA").is_err());
    }

    #[test]
    fn test_key_modifiers_serialize() {
        let source = SerdeWrapper::new(kb_state_from_keys(&[
            Key::LeftShift,
            Key::RightShift,
            Key::RightWin
        ]));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);

        let source = SerdeWrapper::new(KeyModifiers::Any);
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }
}
