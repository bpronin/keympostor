use crate::key::KeyCode;
use crate::key_event::{KeyEventPattern, KeyTransition};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Eq, PartialEq)]
pub struct KeyAction {
    pub key_code: KeyCode,
    pub transition: KeyTransition,
}

#[derive(Debug, PartialEq)]
pub struct KeyTransformRule {
    pub trigger: KeyEventPattern,
    pub action: Vec<KeyAction>,
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {}
}
