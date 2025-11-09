use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct KeyError {
    message: String,
}

impl Default for KeyError {
    fn default() -> Self {
        Self {
            message: "Keyboard crate error".into(),
        }
    }
}

impl KeyError {
    pub(crate) fn new(message: &str) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for KeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for KeyError {}