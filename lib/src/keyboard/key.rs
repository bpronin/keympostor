use crate::keyboard::consts::KEY_MAP;
use crate::keyboard::error::KeyError;
use crate::{deserialize_from_string, serialize_to_string};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, LLKHF_EXTENDED};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Key {
    pub name: &'static str,
    pub vk_code: u8,
    pub scan_code: (u8, bool),
}

impl From<KBDLLHOOKSTRUCT> for Key {
    fn from(input: KBDLLHOOKSTRUCT) -> Self {
        let vk_code = input.vkCode as u8;
        let scan_code = (input.scanCode as u8, input.flags.contains(LLKHF_EXTENDED));
        let name = KEY_MAP.with(|k| k.name_of(&(vk_code, scan_code)));
        Self {
            vk_code,
            scan_code,
            name,
        }
    }
}

impl Key {
    pub(crate) fn from_name(s: &str) -> Result<Self, KeyError> {
        KEY_MAP.with(|keys| keys.by_name(s.trim()))
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.name, f)
    }
}

impl FromStr for Key {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_name(s)
    }
}

impl Serialize for Key {
    serialize_to_string!();
}

impl<'de> Deserialize<'de> for Key {
    deserialize_from_string!();
}

#[macro_export]
macro_rules! key {
    ($text:literal) => {
        Key::from_name($text).unwrap()
    };
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key::Key;
    use crate::utils::test::SerdeWrapper;
    use std::str::FromStr;

    #[test]
    fn test_key_display() {
        assert_eq!(
            "ENTER",
            format!(
                "{}",
                Key {
                    name: "ENTER",
                    vk_code: 0x0D,
                    scan_code: (0x1C, false),
                }
            )
        );

        assert_eq!(
            "NUM_ENTER",
            format!(
                "{}",
                Key {
                    name: "NUM_ENTER",
                    vk_code: 0x0D,
                    scan_code: (0x1C, true),
                }
            )
        );
    }

    #[test]
    fn test_key_from_str() {
        assert_eq!(
            Key {
                name: "ENTER",
                vk_code: 0x0D,
                scan_code: (0x1C, false),
            },
            Key::from_str("ENTER").unwrap()
        );

        assert_eq!(
            Key {
                name: "NUM_ENTER",
                vk_code: 0x0D,
                scan_code: (0x1C, true),
            },
            Key::from_str("NUM_ENTER").unwrap()
        );

        assert_eq!(
            Key {
                name: "F3",
                vk_code: 0x72,
                scan_code: (0x3D, false),
            },
            Key::from_str("F3").unwrap()
        );
    }

    #[test]
    fn test_key_from_str_fails() {
        assert!(Key::from_str("BANANA").is_err());
    }

    #[test]
    fn test_key_serialize() {
        let source = SerdeWrapper::new(key!("ENTER"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);

        let source = SerdeWrapper::new(key!("NUM_ENTER"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }
}
