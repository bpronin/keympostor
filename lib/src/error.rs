use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
#[derive(PartialEq)]
pub struct KeyError {
    pub message: String,
}

impl Default for KeyError {
    fn default() -> Self {
        Self {
            message: "Keyboard crate error".into(),
        }
    }
}

impl Display for KeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for KeyError {}

#[macro_export]
macro_rules! key_error {
    ($($arg:tt)*) => {
        KeyError{ message: format!($($arg)*) }
    }
}

#[macro_export]
macro_rules! key_err {
    ($($arg:tt)*) => {
        Err(KeyError{ message: format!($($arg)*) })
    }
}
