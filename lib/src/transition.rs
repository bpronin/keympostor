use crate::error::KeyError;
use crate::key_err;
use crate::transition::KeyTransition::{Down, Up};
use std::fmt::{Display, Formatter, Write};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum KeyTransition {
    Up,
    Down,
}

impl KeyTransition {
    #[inline(always)]
    pub(crate) const fn is_transition_char(char: char) -> bool {
        match char {
            '*' | '↓' | '^' | '↑' => true,
            _ => false,
        }
    }

    pub(crate) fn from_char(char: char) -> Result<Self, KeyError> {
        match char {
            '*' | '↓' => Ok(Down),
            '^' | '↑' => Ok(Up),
            _ => key_err!("Invalid transition character: `{char}`"),
        }
    }
}

impl Default for KeyTransition {
    fn default() -> Self {
        Up
    }
}

impl Display for KeyTransition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Up => f.write_char('↑'),
            Down => f.write_char('↓'),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::transition::KeyTransition;
    use crate::transition::KeyTransition::{Down, Up};

    #[test]
    fn test_key_transition_display() {
        assert_eq!("↓", format!("{}", Down));
        assert_eq!("↑", format!("{}", Up));
    }

    #[test]
    fn test_key_transition_basics() {
        assert_eq!(Up, KeyTransition::default());
    }
}
