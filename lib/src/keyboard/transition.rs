use crate::keyboard::transition::KeyTransition::{Distance, Down, Up};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum KeyTransition {
    Up,
    Down,
    Distance(i16, i16),
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
            Distance(x, y) => write!(f, "({}:{})", x, y),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::transition::KeyTransition;
    use crate::keyboard::transition::KeyTransition::{Distance, Down, Up};
    use crate::utils::test::SerdeWrapper;

    #[test]
    fn test_key_transition_display() {
        assert_eq!("↓", format!("{}", Down));
        assert_eq!("↑", format!("{}", Up));
        assert_eq!("(0:100)", format!("{}", Distance(0, 100)));
        assert_eq!("(-100:0)", format!("{}", Distance(-100, 0)));
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
