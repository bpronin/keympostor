use crate::key::{KeyCode, ScanCode, VirtualKey};
use crate::key_event::KeyTransition::Up;
use std::fmt::{Display, Formatter};
use KeyTransition::Down;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum KeyTransition {
    #[serde(alias = "UP", alias = "up")]
    Up,
    #[serde(alias = "DOWN", alias = "down")]
    Down,
}

impl KeyTransition {
    pub(crate) fn is_up(&self) -> bool {
        matches!(*self, Up)
    }

    pub(crate) fn is_down(&self) -> bool {
        matches!(*self, Down)
    }
}

impl Default for KeyTransition {
    fn default() -> Self {
        Up
    }
}

impl Display for KeyTransition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Up => '↑',
                Down => '↓',
            }
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct KeyEvent {
    pub virtual_key: VirtualKey,
    pub scan_code: ScanCode,
    pub transition: KeyTransition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum KeyEventPattern {
    Sequence(Vec<KeyEvent>),
    Chord(Vec<KeyEvent>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct KeyAction {
    pub key_code: KeyCode,
    pub transition: KeyTransition,
}

#[derive(Debug, Eq, PartialEq)]
pub struct KeyTransformRule {
    pub trigger: KeyEventPattern,
    pub action: Vec<KeyAction>,
}
