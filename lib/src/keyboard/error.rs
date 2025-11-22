use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct KeyError {
    message: String,
}

impl KeyError {
    pub(crate) fn new(message: &str) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn err<T>(message: &str) -> Result<T, KeyError> {
        Err::<T, KeyError>(Self::new(message))
    }
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
macro_rules! key_err {
    ($($arg:tt)*) => {
        KeyError::err(&format!($($arg)*))
    }
}
