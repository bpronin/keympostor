use crate::key::KeyCode;
use crate::key_event::{KeyEvent, KeyTransition};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyAction {
    pub key: KeyCode,
    pub transition: KeyTransition,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyActionSequence {
    actions: Vec<KeyAction>,
}

#[derive(Debug, PartialEq)]
pub enum KeyActionPattern {
    Sequence(Vec<KeyEvent>),
    Chord(Vec<KeyEvent>),
}

#[derive(Debug, PartialEq)]
pub struct KeyTransformRule {
    pub trigger: KeyActionPattern,
    pub action: KeyActionSequence,
}

#[cfg(test)]
mod tests {
    use KeyCode::VK;
    use crate::key::{KeyCode, VirtualKey};
    use crate::key_action::KeyAction;
    use crate::key_event::KeyTransition::Down;

    #[test]
    fn test_key_action_serialize() {
        let source = KeyAction {
            key: VK(VirtualKey::by_name("VK_RETURN").unwrap()),
            transition: Down,
        };
        let json = serde_json::to_string_pretty(&source).unwrap();

        println!("{}", json);

        let actual = serde_json::from_str::<KeyAction>(&json).unwrap();
        assert_eq!(source, actual);
    }
}
