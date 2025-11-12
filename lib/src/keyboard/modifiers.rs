use crate::keyboard::error::KeyError;
use crate::keyboard::modifiers::KeyModifiers::All;
use crate::{deserialize_from_string, serialize_to_string, write_joined};
use core::ops;
use ops::BitOr;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_RMENU, VK_RSHIFT,
    VK_RWIN,
};

pub(crate) const KM_NONE: ModifierKeys = ModifierKeys(0);
pub(crate) const KM_LSHIFT: ModifierKeys = ModifierKeys(1);
pub(crate) const KM_RSHIFT: ModifierKeys = ModifierKeys(1 << 1);
pub(crate) const KM_LCTRL: ModifierKeys = ModifierKeys(1 << 2);
pub(crate) const KM_RCTRL: ModifierKeys = ModifierKeys(1 << 3);
pub(crate) const KM_LALT: ModifierKeys = ModifierKeys(1 << 4);
pub(crate) const KM_RALT: ModifierKeys = ModifierKeys(1 << 5);
pub(crate) const KM_LWIN: ModifierKeys = ModifierKeys(1 << 6);
pub(crate) const KM_RWIN: ModifierKeys = ModifierKeys(1 << 7);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct ModifierKeys(u8);

impl ModifierKeys {
    pub(crate) const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn to_string_short(&self) -> String {
        let mut text: [char; 8] = ['.'; 8];

        if self.contains(KM_LSHIFT) {
            text[0] = 'S';
        }
        if self.contains(KM_RSHIFT) {
            text[7] = 'S';
        }
        if self.contains(KM_LCTRL) {
            text[1] = 'C';
        }
        if self.contains(KM_RCTRL) {
            text[6] = 'C';
        }
        if self.contains(KM_LALT) {
            text[2] = 'A';
        }
        if self.contains(KM_RALT) {
            text[5] = 'A';
        }
        if self.contains(KM_LWIN) {
            text[3] = 'W';
        }
        if self.contains(KM_RWIN) {
            text[4] = 'W';
        }

        text.iter().collect()
    }
}

static MODIFIER_KEYS: [VIRTUAL_KEY; 8] = [
    VK_LSHIFT,
    VK_RSHIFT,
    VK_LCONTROL,
    VK_RCONTROL,
    VK_LMENU,
    VK_RMENU,
    VK_LWIN,
    VK_RWIN,
];

impl BitOr for ModifierKeys {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl Display for ModifierKeys {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut names: Vec<&str> = Vec::new();

        if self.contains(KM_LSHIFT) {
            names.push("LEFT_SHIFT");
        }
        if self.contains(KM_RSHIFT) {
            names.push("RIGHT_SHIFT");
        }
        if self.contains(KM_LCTRL) {
            names.push("LEFT_CTRL");
        }
        if self.contains(KM_RCTRL) {
            names.push("RIGHT_CTRL");
        }
        if self.contains(KM_LALT) {
            names.push("LEFT_ALT");
        }
        if self.contains(KM_RALT) {
            names.push("RIGHT_ALT");
        }
        if self.contains(KM_LWIN) {
            names.push("LEFT_WIN");
        }
        if self.contains(KM_RWIN) {
            names.push("RIGHT_WIN");
        }

        if !names.is_empty() {
            write_joined!(f, names, " + ")
        } else {
            Ok(())
        }
    }
}

impl From<&[bool; 256]> for ModifierKeys {
    fn from(keyboard_state: &[bool; 256]) -> Self {
        let value = (0..MODIFIER_KEYS.len())
            .filter(|modifier_index| {
                let vk_code = MODIFIER_KEYS[*modifier_index].0;
                keyboard_state[vk_code as usize]
            })
            .fold(0, |acc, flag_index| acc | (1 << flag_index));

        Self(value as u8)
    }
}

impl FromStr for ModifierKeys {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ts = s.trim();
        if ts.is_empty() {
            Ok(KM_NONE)
        } else {
            let result = ts.split('+').fold(KM_NONE, |acc, part| {
                acc | match part.trim() {
                    "LEFT_SHIFT" => KM_LSHIFT,
                    "RIGHT_SHIFT" => KM_RSHIFT,
                    "SHIFT" => KM_LSHIFT | KM_RSHIFT,
                    "LEFT_CTRL" => KM_LCTRL,
                    "RIGHT_CTRL" => KM_RCTRL,
                    "CTRL" => KM_LCTRL | KM_RCTRL,
                    "LEFT_ALT" => KM_LALT,
                    "RIGHT_ALT" => KM_RALT,
                    "ALT" => KM_LALT | KM_RALT,
                    "LEFT_WIN" => KM_LWIN,
                    "RIGHT_WIN" => KM_RWIN,
                    "WIN" => KM_LWIN | KM_RWIN,
                    &_ => KM_NONE,
                }
            });

            if result != KM_NONE {
                Ok(result)
            } else {
                Err(KeyError::new(&format!(
                    "Error parsing key modifiers: `{ts}`"
                )))
            }
        }
    }
}

impl Serialize for ModifierKeys {
    serialize_to_string!();
}

impl<'de> Deserialize<'de> for ModifierKeys {
    deserialize_from_string!();
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum KeyModifiers {
    Any,
    All(ModifierKeys),
}

impl Display for KeyModifiers {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let All(modifiers) = self {
            write!(f, "[{}]", modifiers)
        } else {
            Ok(())
        }
    }
}

impl FromStr for KeyModifiers {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        /* `Any` is parsed outside from `None` */
        Ok(All(ModifierKeys::from_str(s.trim())?))
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::modifiers::KeyModifiers::{All, Any};
    use crate::keyboard::modifiers::{
        KeyModifiers, ModifierKeys, KM_LALT, KM_LCTRL, KM_LSHIFT, KM_LWIN, KM_NONE, KM_RALT, KM_RCTRL,
        KM_RSHIFT, KM_RWIN,
    };
    use std::str::FromStr;
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LCONTROL, VK_LSHIFT, VK_RSHIFT, VK_RWIN};
    use crate::utils::test::SerdeWrapper;

    #[macro_export]
    macro_rules! key_mod {
        ($text:literal) => {
            $text.parse::<ModifierKeys>().unwrap()
        };
    }

    #[test]
    fn test_key_modifiers_display() {
        assert_eq!("", KM_NONE.to_string());

        assert_eq!("LEFT_SHIFT + RIGHT_WIN", (KM_LSHIFT | KM_RWIN).to_string());
        assert_eq!("RIGHT_CTRL + LEFT_ALT", (KM_LALT | KM_RCTRL).to_string());

        // assert_eq!(
        //     "SHIFT + CTRL + ALT + WIN",
        //     (KM_LSHIFT | KM_RSHIFT | KM_LWIN | KM_RWIN | KM_LALT | KM_RALT | KM_LCTRL | KM_RCTRL)
        //         .to_string()
        // );
    }

    #[test]
    fn test_key_modifiers_display_short() {
        assert_eq!("........", KM_NONE.to_string_short());
        assert_eq!("S...W...", (KM_LSHIFT | KM_RWIN).to_string_short());
        assert_eq!("..A...C.", (KM_LALT | KM_RCTRL).to_string_short());

        assert_eq!(
            "SCAWWACS",
            (KM_LSHIFT | KM_RSHIFT | KM_LWIN | KM_RWIN | KM_LALT | KM_RALT | KM_LCTRL | KM_RCTRL)
                .to_string_short()
        );
    }

    #[test]
    fn test_key_modifiers_capture() {
        let mut keys = [false; 256];
        assert_eq!(KM_NONE, ModifierKeys::from(&keys));

        keys[VK_LSHIFT.0 as usize] = true;
        keys[VK_RSHIFT.0 as usize] = true;
        keys[VK_LCONTROL.0 as usize] = true;
        keys[VK_RWIN.0 as usize] = true;

        assert_eq!(
            KM_LSHIFT | KM_RSHIFT | KM_LCTRL | KM_RWIN,
            ModifierKeys::from(&keys)
        );
    }

    #[test]
    fn test_keyboard_state_display() {
        assert_eq!(
            "[LEFT_SHIFT + RIGHT_WIN]",
            KeyModifiers::All(KM_LSHIFT | KM_RWIN).to_string()
        );
        assert_eq!("[]", KeyModifiers::All(KM_NONE).to_string());
        assert_eq!("", KeyModifiers::Any.to_string());
    }

    #[test]
    fn test_key_modifiers_from_str() {
        assert_eq!(All(KM_NONE), KeyModifiers::from_str("").unwrap());

        assert_eq!(
            All(KM_LSHIFT | KM_RSHIFT | KM_RWIN),
            KeyModifiers::from_str("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN").unwrap()
        );
    }

    #[test]
    fn test_key_modifiers_from_str_fails() {
        assert!(KeyModifiers::from_str("BANANA").is_err());
    }

    #[test]
    fn test_key_modifier_keys_serialize() {
        let source = SerdeWrapper::new(key_mod!("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_modifiers_serialize() {
        let source = SerdeWrapper::new(All(key_mod!("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN")));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);

        let source = SerdeWrapper::new(Any);
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }
}
