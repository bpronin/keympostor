use crate::trigger::KeyTrigger;
use std::fmt::{Display, Formatter, Write};

#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent {
    pub trigger: KeyTrigger,
    pub time: u32,
    pub is_injected: bool,
    pub is_private: bool,
}

impl Display for KeyEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        write!(s, "{}", self.trigger)?;
        if self.is_injected {
            write!(s, " INJECTED")?;
        }
        if self.is_private {
            write!(s, " PRIVATE")?;
        }
        f.pad(&s)
    }
}

#[cfg(test)]
mod tests {
    use crate::event::KeyEvent;
    use crate::key_trigger;
    use crate::trigger::KeyTrigger;
    use std::str::FromStr;

    #[test]
    fn test_key_event_display() {
        let event = KeyEvent {
            trigger: key_trigger!("[LEFT_SHIFT] A↓"),
            time: 0,
            is_injected: false,
            is_private: false,
        };
        assert_eq!("|     [LEFT_SHIFT] A↓|", format!("|{:>20}|", event));

        let event = KeyEvent {
            trigger: key_trigger!("[LEFT_SHIFT] A↓"),
            time: 0,
            is_injected: true,
            is_private: false,
        };
        assert_eq!(
            "|                [LEFT_SHIFT] A↓ INJECTED|",
            format!("|{:>40}|", event)
        );

        let event = KeyEvent {
            trigger: key_trigger!("[LEFT_SHIFT] A↓"),
            time: 0,
            is_injected: true,
            is_private: true,
        };
        assert_eq!(
            "|        [LEFT_SHIFT] A↓ INJECTED PRIVATE|",
            format!("|{:>40}|", event)
        );
    }
}
