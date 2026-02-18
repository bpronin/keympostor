use crate::error::KeyError;
use crate::state::KeyboardState;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use crate::modifiers::KeyModifiers::{All, Any};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum KeyModifiers {
    Any,
    All(KeyboardState),
}

impl Display for KeyModifiers {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            All(m) => write!(f, "[{}]", m),
            Any => Ok(()),
        }
    }
}

impl FromStr for KeyModifiers {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Any);
        }

        let part = s.trim().trim_start_matches('[').trim_end_matches(']');
        Ok(All(KeyboardState::from_str(part)?))
    }
}

#[cfg(test)]
mod tests {
    use crate::key::Key;
    use crate::modifiers::KeyModifiers;
    use crate::state::tests::kb_state_from_keys;
    use crate::state::KeyboardState;
    use std::str::FromStr;
    use crate::modifiers::KeyModifiers::{All, Any};

    #[test]
    fn test_key_modifiers_to_str() {
        assert_eq!(
            "[RIGHT_WIN + LEFT_SHIFT + RIGHT_SHIFT]",
            All(kb_state_from_keys(&[
                Key::LeftShift,
                Key::RightShift,
                Key::RightWin
            ]))
            .to_string()
        );

        assert_eq!(
            "[]",
            All(KeyboardState::default()).to_string()
        );

        assert_eq!("", Any.to_string());
    }

    #[test]
    fn test_key_modifiers_from_str() {
        assert_eq!(
            Ok(All(kb_state_from_keys(&[
                Key::LeftShift,
                Key::RightShift,
                Key::RightWin
            ]))),
            KeyModifiers::from_str("[LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN]")
        );

        assert_eq!(
            Ok(All(KeyboardState::default())),
            KeyModifiers::from_str("[]")
        );

        assert_eq!(Ok(Any), KeyModifiers::from_str(""));
    }

    #[test]
    fn test_key_modifiers_from_str_fails() {
        assert!(KeyModifiers::from_str("BANANA").is_err());
    }
}
