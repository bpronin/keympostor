use std::fmt::{Display, Formatter};
use crate::key_transition::KeyTransition::{Down, Up};
use serde::{Deserialize, Serialize};
use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, LLKHF_UP};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KeyTransition {
    #[serde(alias = "UP", alias = "up")]
    Up,
    #[serde(alias = "DOWN", alias = "down")]
    Down,
}

impl Default for KeyTransition {
    fn default() -> Self {
        Up
    }
}

impl KeyTransition {
    pub(crate) fn from_kb(kb: &KBDLLHOOKSTRUCT) -> KeyTransition {
        if kb.flags.contains(LLKHF_UP) {
            Up
        } else {
            Down
        }
    }

    pub(crate) fn is_up(self) -> bool {
        match self {
            Up => true,
            Down => false,
        }
    }
}

impl Display for KeyTransition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Up => '↑',
            Down => '↓'
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::key_transition::KeyTransition;

    #[test]
    fn key_transition_display() {
        assert_eq!("↓",format!("{}", KeyTransition::Down));
        assert_eq!("↑",format!("{}", KeyTransition::Up));
    }
}
