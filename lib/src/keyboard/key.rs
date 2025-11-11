use crate::keyboard::consts::{KEY_MAP, SCAN_CODES, VIRTUAL_KEYS};
use crate::keyboard::error::KeyError;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, LLKHF_EXTENDED};
use crate::{deserialize_from_string, serialize_to_string};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct VirtualKey {
    pub(crate) value: u8,
    pub(crate) name: &'static str,
}

impl VirtualKey {
    pub(crate) fn from_code(code: u8) -> Result<VirtualKey, KeyError> {
        VIRTUAL_KEYS
            .get(code as usize)
            .ok_or(KeyError::new(&format!(
                "Illegal virtual key code `{}`.",
                code
            )))
            .copied()
    }

    pub(crate) fn code_name(&self) -> String {
        format!("VC_0x{:02X}", self.value)
    }
}

impl Display for VirtualKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.name, f)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ScanCode {
    pub(crate) value: u8,
    pub(crate) is_extended: bool,
    pub(crate) name: &'static str,
}

impl ScanCode {
    pub(crate) fn from_code(code: u8, extended: bool) -> Result<ScanCode, KeyError> {
        SCAN_CODES
            .get(code as usize)
            .ok_or(KeyError::new(&format!("Illegal scan code `{}`.", code)))?
            .get(extended as usize)
            .ok_or(KeyError::new(&format!(
                "Illegal extended scan code `{}`.",
                code
            )))
            .copied()
    }

    pub(crate) fn ext_value(&self) -> u16 {
        if self.is_extended {
            self.value as u16 | 0xE0 << 8
        } else {
            self.value as u16
        }
    }

    pub(crate) fn code_name(&self) -> String {
        format!("SC_0x{:04X}", self.ext_value())
    }
}

impl Display for ScanCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.name, f)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Key {
    pub vk_code: u8,
    pub scan_code: u8,
    pub is_ext_scan_code: bool,
}

impl Key {
    pub(crate) fn from_keyboard_input(input: &KBDLLHOOKSTRUCT) -> Self {
        Self {
            vk_code: input.vkCode as u8,
            scan_code: input.scanCode as u8,
            is_ext_scan_code: input.flags.contains(LLKHF_EXTENDED),
        }
    }

    pub(crate) fn from_name(s: &str) -> Result<Self, KeyError> {
        KEY_MAP.with(|keys| keys.by_name(s.trim()))
    }

    pub(crate) fn name(&self) -> &'static str {
        KEY_MAP.with(|k| k.name_of(self))
    }

    pub fn virtual_key(&self) -> VirtualKey {
        VirtualKey::from_code(self.vk_code).unwrap()
    }

    pub fn scan_code(&self) -> ScanCode {
        ScanCode::from_code(self.scan_code, self.is_ext_scan_code).unwrap()
    }

    pub(crate) fn code_name(&self) -> String {
        format!(
            "{} - {}",
            self.virtual_key().code_name(),
            self.scan_code().code_name()
        )
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.name(), f)
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use serde::{Deserialize, Serialize};
    use crate::append_prefix;
    use crate::keyboard::key::{Key, ScanCode, VirtualKey};
    use crate::keyboard::consts::{SCAN_CODES, VIRTUAL_KEYS};
    use crate::keyboard::error::KeyError;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        MapVirtualKeyW, MAPVK_VK_TO_VSC_EX, MAPVK_VSC_TO_VK_EX,
    };

    impl VirtualKey {
        pub(crate) fn from_name(name: &str) -> Result<VirtualKey, KeyError> {
            let vk_name = append_prefix!(name, "VK_");
            let position = VIRTUAL_KEYS.iter().position(|probe| probe.name == vk_name);

            if let Some(ix) = position {
                Ok(VIRTUAL_KEYS[ix])
            } else {
                Err(KeyError::new(&format!(
                    "Illegal virtual key name `{}`.",
                    name
                )))
            }
        }

        pub(crate) fn from_code_name(s: &str) -> Result<VirtualKey, KeyError> {
            let src = s
                .strip_prefix("VK_0x")
                .ok_or(KeyError::new(&"No `VK_0x` prefix."))?;
            let code = u8::from_str_radix(src, 16).map_err(|e| {
                KeyError::new(&format!("Error parsing virtual key code `{}`. {}", s, e))
            })?;
            Self::from_code(code)
        }

        pub(crate) fn to_scan_code(&self) -> Result<ScanCode, KeyError> {
            let ext_code = unsafe { MapVirtualKeyW(self.value as u32, MAPVK_VK_TO_VSC_EX) };
            if ext_code > 0 {
                let code = ext_code as u8;
                let is_extended = ext_code & 0xE000 != 0;
                ScanCode::from_code(code, is_extended)
            } else {
                Err(KeyError::new(&format!(
                    "Unable to convert virtual key {self} to scancode."
                )))
            }
        }
    }

    impl ScanCode {
        pub(crate) fn from_name(name: &str) -> Result<ScanCode, KeyError> {
            let sc_name = append_prefix!(name, "SC_");
            SCAN_CODES
                .iter()
                .flatten()
                .find(|sc| sc.name == sc_name)
                .ok_or(KeyError::new(&format!(
                    "Illegal scan code name `{}`.",
                    name
                )))
                .copied()
        }

        pub(crate) fn from_code_name(s: &str) -> Result<ScanCode, KeyError> {
            let code = u16::from_str_radix(
                s.strip_prefix("SC_0x")
                    .ok_or(KeyError::new("No `SC_0x` prefix."))?,
                16,
            )
            .map_err(|e| KeyError::new(&format!("Error parsing scan code `{}`. {}", s, e)))?;
            Self::from_ext_code(code)
        }

        pub(crate) fn from_ext_code(ext_code: u16) -> Result<ScanCode, KeyError> {
            Self::from_code(ext_code as u8, ext_code & 0xE000 == 0xE000)
        }

        pub(crate) fn to_virtual_key(&self) -> Result<VirtualKey, KeyError> {
            let vk_code = unsafe { MapVirtualKeyW(self.ext_value() as u32, MAPVK_VSC_TO_VK_EX) };
            if vk_code > 0 {
                VirtualKey::from_code(vk_code as u8)
            } else {
                Err(KeyError::new(&format!(
                    "Unable to convert scancode {self} to virtual key."
                )))
            }
        }
    }

    #[macro_export]
    macro_rules! vk_key {
        ($text:literal) => {
            VirtualKey::from_name($text).unwrap()
        };
    }

    #[macro_export]
    macro_rules! sc_key {
        ($text:literal) => {
            ScanCode::from_name($text).unwrap()
        };
    }

    #[macro_export]
    macro_rules! key {
        ($text:literal) => {
            Key::from_name($text).unwrap()
        };
    }

    #[test]
    fn test_vk_from_code() {
        assert_eq!("VK_RETURN", VirtualKey::from_code(0x0D).unwrap().name);
    }

    #[test]
    fn test_vk_from_name() {
        assert_eq!(
            "VK_RETURN",
            VirtualKey::from_name("VK_RETURN").unwrap().name
        );
    }

    #[test]
    fn test_vk_from_code_name() {
        assert_eq!(
            "VK_RETURN",
            VirtualKey::from_code_name("VK_0x0D").unwrap().name
        );
    }

    #[test]
    fn test_vk_display() {
        assert_eq!(
            "VK_RETURN",
            format!("{}", VirtualKey::from_name("VK_RETURN").unwrap())
        );
    }

    #[test]
    fn test_vk_to_scan_code() {
        assert_eq!(
            sc_key!("SC_ENTER"),
            vk_key!("VK_RETURN").to_scan_code().unwrap()
        );

        assert_eq!(
            sc_key!("SC_RIGHT_WINDOWS"),
            vk_key!("VK_RWIN").to_scan_code().unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_vk_to_scan_code_fails() {
        vk_key!("VK_LBUTTON").to_scan_code().unwrap();
    }

    #[test]
    fn test_sc_from_code() {
        assert_eq!("SC_ENTER", ScanCode::from_code(0x1C, false).unwrap().name);
        assert_eq!(
            "SC_CALCULATOR",
            ScanCode::from_code(0x21, true).unwrap().name
        );
    }

    #[test]
    fn test_sc_from_name() {
        let actual = ScanCode::from_name("SC_ENTER").unwrap();
        assert_eq!(0x1C, actual.value);
        assert_eq!(false, actual.is_extended);

        let actual = ScanCode::from_name("SC_CALCULATOR").unwrap();
        assert_eq!(0x21, actual.value);
        assert_eq!(true, actual.is_extended);
    }

    #[test]
    fn test_sc_from_code_name() {
        assert_eq!(
            "SC_ENTER",
            ScanCode::from_code_name("SC_0x001C").unwrap().name
        );
        assert_eq!(
            "SC_BACKTICK",
            ScanCode::from_code_name("SC_0xE029").unwrap().name
        );
    }

    #[test]
    fn test_sc_from_ext_code() {
        let actual = ScanCode::from_ext_code(0x1C).unwrap();
        assert_eq!(0x1C, actual.value);
        assert_eq!(false, actual.is_extended);

        let actual = ScanCode::from_ext_code(0xE021).unwrap();
        assert_eq!(0x21, actual.value);
        assert_eq!(true, actual.is_extended);
    }

    #[test]
    fn test_sc_ext_value() {
        assert_eq!(0x1C, ScanCode::from_ext_code(0x1C).unwrap().ext_value());
        assert_eq!(0xE021, ScanCode::from_ext_code(0xE021).unwrap().ext_value());
    }

    #[test]
    fn test_sc_to_virtual_key() {
        assert_eq!(
            vk_key!("VK_RETURN"),
            sc_key!("SC_ENTER").to_virtual_key().unwrap()
        );

        assert_eq!(
            vk_key!("VK_RETURN"),
            sc_key!("SC_NUM_ENTER").to_virtual_key().unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_sc_to_virtual_key_fails() {
        sc_key!("SC_F24").to_virtual_key().unwrap();
    }

    #[test]
    fn test_sc_display() {
        assert_eq!("SC_ENTER", format!("{}", sc_key!("SC_ENTER")));
    }

    #[test]
    fn test_key_display() {
        assert_eq!(
            "ENTER",
            format!(
                "{}",
                Key {
                    vk_code: 0x0D,
                    scan_code: 0x1C,
                    is_ext_scan_code: false,
                }
            )
        );

        assert_eq!(
            "NUM_ENTER",
            format!(
                "{}",
                Key {
                    vk_code: 0x0D,
                    scan_code: 0x1C,
                    is_ext_scan_code: true,
                }
            )
        );
    }

    #[test]
    fn test_key_from_str() {
        assert_eq!(
            Key {
                vk_code: 0x0D,
                scan_code: 0x1C,
                is_ext_scan_code: false,
            },
            Key::from_str("ENTER").unwrap()
        );

        assert_eq!(
            Key {
                vk_code: 0x0D,
                scan_code: 0x1C,
                is_ext_scan_code: true,
            },
            Key::from_str("NUM_ENTER").unwrap()
        );

        assert_eq!(
            Key {
                vk_code: 0x72,
                scan_code: 0x3D,
                is_ext_scan_code: false,
            },
            Key::from_str("F3").unwrap()
        );
    }

    #[test]
    fn test_key_from_str_fails() {
        assert!(Key::from_str("BANANA").is_err());
    }

    /* TOML requires root node to be annotated as #[derive(Serialize, Deserialize)] */
    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Wrapper<T> {
        value: T,
    }

    #[test]
    fn test_key_serialize() {
        let source = Wrapper {
            value: key!("ENTER"),
        };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);

        let source = Wrapper {
            value: key!("NUM_ENTER"),
        };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }

}
