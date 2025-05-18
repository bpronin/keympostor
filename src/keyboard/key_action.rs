use crate::keyboard::key::Key;
use crate::keyboard::key_action::KeyTransition::{Down, Up};
use crate::keyboard::key_event::SELF_EVENT_MARKER;
use crate::write_joined;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
    KEYEVENTF_SCANCODE, VIRTUAL_KEY,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) enum KeyTransition {
    #[serde(alias = "UP", alias = "up")]
    Up,
    #[serde(alias = "DOWN", alias = "down")]
    Down,
}

impl KeyTransition {
    pub(crate) fn from_bool(up: bool) -> KeyTransition {
        if up { Up } else { Down }
    }

    pub(crate) fn is_up(&self) -> bool {
        matches!(*self, Up)
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

impl FromStr for KeyTransition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.trim().chars();
        let symbol = chars.next().ok_or("Key transition symbol is empty.")?;
        if chars.next().is_none() {
            match symbol {
                '↑' | '^' => Ok(Up),
                '↓' | '*' => Ok(Down),
                _ => Err(format!("Illegal key transition symbol `{}`.", s)),
            }
        } else {
            Err(format!("Key transition symbols `{}` is too long.", s))
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct KeyAction {
    pub(crate) key: Key,
    pub(crate) transition: KeyTransition,
}

impl KeyAction {
    fn new(key_name: &str, transition_symbol: char) -> Result<Self, String> {
        Ok(Self {
            key: key_name.parse()?,
            transition: transition_symbol.to_string().parse()?,
        })
    }

    fn create_input(&self) -> INPUT {
        let virtual_key = self.key.virtual_key();
        let scan_code = self.key.scan_code();

        let mut flags = KEYEVENTF_SCANCODE;
        if scan_code.is_extended {
            flags |= KEYEVENTF_EXTENDEDKEY
        }
        if self.transition.is_up() {
            flags |= KEYEVENTF_KEYUP;
        }

        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(virtual_key.value as u16),
                    wScan: scan_code.ext_value(),
                    dwFlags: flags,
                    dwExtraInfo: SELF_EVENT_MARKER.as_ptr() as usize,
                    ..Default::default()
                },
            },
        }
    }
}

impl Display for KeyAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.key, self.transition)
    }
}

impl FromStr for KeyAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let st = s.trim();

        let transition_symbol = st
            .chars()
            .last()
            .ok_or(&format!("Error parsing key action. String is empty. `{s}`"))?;

        let key_name = st.strip_suffix(transition_symbol).ok_or(&format!(
            "Invalid key action suffix: `{transition_symbol}`."
        ))?;

        Self::new(key_name, transition_symbol)
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct KeyActionSequence {
    pub(crate) actions: Vec<KeyAction>,
}

impl KeyActionSequence {
    pub(crate) fn create_input(&self) -> Vec<INPUT> {
        self.actions.iter().map(|a| a.create_input()).collect()
    }
}

impl Display for KeyActionSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_joined!(f, &self.actions, " → ")
    }
}

impl FromStr for KeyActionSequence {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let actions = s
            .split(|ch| ['→', '>'].contains(&ch))
            .flat_map(|part| {
                let part = part.trim();

                let (prefix, suffixes) = part
                    .char_indices()
                    .find(|(_, ch)| ['↑', '↓', '^', '*'].contains(ch))
                    .map(|(ix, _)| part.split_at(ix))
                    .unwrap_or((part, "↓↑"));

                suffixes
                    .chars()
                    .map(move |suffix| KeyAction::new(prefix, suffix))
            })
            .collect::<Result<Vec<_>, Self::Err>>()?;

        Ok(Self { actions })
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key::ScanCode;
    use crate::keyboard::key_action::Key;
    use crate::keyboard::key_action::KeyTransition::{Down, Up};
    use crate::keyboard::key_action::{KeyAction, KeyActionSequence, KeyTransition};
    use crate::keyboard::key_event::SELF_EVENT_MARKER;
    use crate::{assert_not, key, sc_key};
    use serde::{Deserialize, Serialize};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT_KEYBOARD, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VK_RETURN,
    };

    #[macro_export]
    macro_rules! key_action {
        ($text:literal) => {
            $text.parse::<KeyAction>().unwrap()
        };
    }

    #[macro_export]
    macro_rules! key_action_seq {
        ($text:literal) => {
            $text.parse::<KeyActionSequence>().unwrap()
        };
    }

    #[test]
    fn test_key_transition_display() {
        assert_eq!("↓", format!("{}", Down));
        assert_eq!("↑", format!("{}", Up));
    }

    #[test]
    fn test_key_transition_basics() {
        assert_eq!(Up, KeyTransition::default());
        assert_eq!(Up, KeyTransition::from_bool(true));
        assert!(Up.is_up());
        assert_not!(Down.is_up());
    }

    #[test]
    fn test_key_transition_parse() {
        assert_eq!(Down, "↓".parse().unwrap());
        assert_eq!(Up, "↑".parse().unwrap());
        assert_eq!(Down, "*".parse().unwrap());
        assert_eq!(Up, "^".parse().unwrap());
    }

    #[test]
    #[should_panic]
    fn test_key_transition_parse_fails_illegal() {
        assert_eq!(Down, "BANANA".parse().unwrap());
    }

    #[test]
    #[should_panic]
    fn test_key_transition_parse_fails_empty() {
        assert_eq!(Down, "".parse().unwrap());
    }

    #[test]
    #[should_panic]
    fn test_key_transition_parse_fails_to_long() {
        assert_eq!(Down, "↑↑↑".parse().unwrap());
    }

    #[test]
    fn test_key_transition_serialize() {
        /* TOML requires wrapper */
        #[derive(Debug, Serialize, Deserialize)]
        struct Wrapper {
            value: KeyTransition,
        }

        let source = Wrapper { value: Down };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<Wrapper>(&text).unwrap();
        assert_eq!(source.value, actual.value);

        let source = Wrapper { value: Up };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<Wrapper>(&text).unwrap();
        assert_eq!(source.value, actual.value);
    }

    #[test]
    fn test_key_action_display() {
        let actual = KeyAction {
            key: key!("ENTER"),
            transition: Down,
        };
        assert_eq!("ENTER↓", format!("{}", actual));

        let actual = KeyAction {
            key: key!("NUM_ENTER"),
            transition: Up,
        };
        assert_eq!("NUM_ENTER↑", format!("{}", actual));
    }

    #[test]
    fn test_key_action_parse() {
        let expected = KeyAction {
            key: key!("ENTER"),
            transition: Down,
        };
        assert_eq!(expected, "ENTER↓".parse().unwrap());

        let expected = KeyAction {
            key: key!("F3"),
            transition: Down,
        };
        assert_eq!(expected, " F3\n*".parse().unwrap());
    }

    #[test]
    fn test_key_action_serialize() {
        let source = KeyAction {
            key: key!("ENTER"),
            transition: Down,
        };
        let text = toml::to_string_pretty(&source).unwrap();

        let actual = toml::from_str::<KeyAction>(&text).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_action_create_input() {
        let actual = key_action!("ENTER*").create_input();
        unsafe {
            assert_eq!(INPUT_KEYBOARD, actual.r#type);
            assert_eq!(VK_RETURN, actual.Anonymous.ki.wVk);
            assert_eq!(sc_key!("SC_ENTER").ext_value(), actual.Anonymous.ki.wScan);
            assert_eq!(KEYEVENTF_SCANCODE, actual.Anonymous.ki.dwFlags);
            assert_eq!(
                SELF_EVENT_MARKER.as_ptr(),
                actual.Anonymous.ki.dwExtraInfo as *const u8
            );
        };

        let actual = key_action!("NUM_ENTER^").create_input();
        unsafe {
            assert_eq!(INPUT_KEYBOARD, actual.r#type);
            assert_eq!(VK_RETURN, actual.Anonymous.ki.wVk);
            assert_eq!(
                sc_key!("SC_NUM_ENTER").ext_value(),
                actual.Anonymous.ki.wScan
            );
            assert_eq!(
                KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
                actual.Anonymous.ki.dwFlags
            );
            assert_eq!(
                SELF_EVENT_MARKER.as_ptr(),
                actual.Anonymous.ki.dwExtraInfo as *const u8
            );
        };
    }

    #[test]
    fn test_key_action_sequence_display() {
        let actual = key_action_seq!("ENTER↓ → SHIFT↑");

        assert_eq!("ENTER↓ → SHIFT↑", format!("{}", actual));
    }

    #[test]
    fn test_key_action_sequence_serialize() {
        let source = key_action_seq!("ENTER↓ → SHIFT↓");
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();
        
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_action_sequence_parse_expand_transition() {
        let expected = key_action_seq!("A↓ → A↑");

        assert_eq!(expected, "A↓↑".parse().unwrap());
        assert_eq!(expected, "A".parse().unwrap());
    }
}
