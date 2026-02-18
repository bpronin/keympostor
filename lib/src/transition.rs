use crate::transition::KeyTransition::{Down, Up};
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum KeyTransition {
    Up,
    Down,
}

impl KeyTransition {
    pub(crate) fn from_bool(value: bool) -> Self {
        if value { Down } else { Up }
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
            Up => Display::fmt(&'↑', f),
            Down => Display::fmt(&'↓', f),
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

    // #[test]
    // fn test_key_transition_serialize() {
    //     let source = SerdeWrapper::new(Down);
    //     let text = toml::to_string(&source).unwrap();
    //     let actual = toml::from_str(&text).unwrap();
    //
    //     assert_eq!(source, actual);
    //     assert_eq!(actual.value, Down);
    //
    //     let source = SerdeWrapper::new(Up);
    //     let text = toml::to_string(&source).unwrap();
    //     let actual = toml::from_str(&text).unwrap();
    //
    //     assert_eq!(source, actual);
    //     assert_eq!(actual.value, Up);
    //
    //     let source = SerdeWrapper::new(Distance(100, 200));
    //     let text = toml::to_string(&source).unwrap();
    //     let actual = toml::from_str(&text).unwrap();
    //
    //     assert_eq!(source, actual);
    //     assert_eq!(actual.value, Distance(100, 200));
    // }
}
