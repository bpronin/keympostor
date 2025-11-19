use crate::keyboard::consts::{SCAN_CODES, VIRTUAL_KEYS};
use crate::keyboard::key::Key;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct VirtualKey {
    pub(crate) value: u8,
    pub(crate) name: &'static str,
}

impl VirtualKey {
    pub(crate) fn hex_code(&self) -> String {
        format!("VC_0x{:02X}", self.value)
    }
}

impl FromStr for VirtualKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        VIRTUAL_KEYS
            .iter()
            .filter(|x| x.name == s)
            .next()
            .ok_or(())
            .copied()
    }
}

impl From<u8> for VirtualKey {
    fn from(code: u8) -> Self {
        VIRTUAL_KEYS[code as usize]
    }
}

impl From<&Key> for VirtualKey {
    fn from(key: &Key) -> Self {
        Self::from(key.vk_code)
    }
}

impl Display for VirtualKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.name, f)
    }
}

#[macro_export]
macro_rules! vk_key {
    ($text:literal) => {
        VirtualKey::from_str($text).unwrap()
    };
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ScanCode {
    pub(crate) value: u8,
    pub(crate) is_extended: bool,
    pub(crate) name: &'static str,
}

impl ScanCode {
    pub(crate) fn ext_value(&self) -> u16 {
        if self.is_extended {
            self.value as u16 | 0xE0 << 8
        } else {
            self.value as u16
        }
    }

    pub(crate) fn hex_code(&self) -> String {
        format!("SC_0x{:04X}", self.ext_value())
    }
}

impl Display for ScanCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.name, f)
    }
}

impl From<(u8, bool)> for ScanCode {
    fn from(code: (u8, bool)) -> Self {
        SCAN_CODES[code.0 as usize][code.1 as usize]
    }
}

impl From<&Key> for ScanCode {
    fn from(key: &Key) -> Self {
        Self::from(key.scan_code)
    }
}

impl FromStr for ScanCode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SCAN_CODES
            .iter()
            .flatten()
            .filter(|x| x.name == s)
            .next()
            .ok_or(())
            .copied()
    }
}

#[macro_export]
macro_rules! sc_key {
    ($text:literal) => {
        ScanCode::from_str($text).unwrap()
    };
}

#[cfg(test)]
mod tests {
    use crate::keyboard::code::ScanCode;
    use crate::keyboard::code::VirtualKey;
    use crate::{sc_key, vk_key};
    use std::str::FromStr;

    #[test]
    fn test_vk_from_code() {
        assert_eq!("VK_RETURN", VirtualKey::from(0x0D).name);
    }

    #[test]
    fn test_vk_from_str() {
        assert_eq!("VK_RETURN", VirtualKey::from_str("VK_RETURN").unwrap().name);
    }

    #[test]
    fn test_vk_display() {
        assert_eq!("VK_RETURN", format!("{}", vk_key!("VK_RETURN")));
    }

    #[test]
    fn test_sc_from_code() {
        assert_eq!("SC_ENTER", ScanCode::from((0x1C, false)).name);
        assert_eq!("SC_CALCULATOR", ScanCode::from((0x21, true)).name);
    }

    #[test]
    fn test_sc_from_str() {
        let actual = ScanCode::from_str("SC_ENTER").unwrap();
        assert_eq!(0x1C, actual.value);
        assert_eq!(false, actual.is_extended);

        let actual = ScanCode::from_str("SC_CALCULATOR").unwrap();
        assert_eq!(0x21, actual.value);
        assert_eq!(true, actual.is_extended);
    }

    // #[test]
    // fn test_sc_from_code_name() {
    //     assert_eq!("SC_ENTER", ScanCode::from_str("SC_0x001C").unwrap().name);
    //     assert_eq!("SC_BACKTICK", ScanCode::from_str("SC_0xE029").unwrap().name);
    // }

    #[test]
    fn test_sc_ext_value() {
        assert_eq!(0x1C, sc_key!("SC_ENTER").ext_value());
        assert_eq!(0xE01D, sc_key!("SC_RIGHT_CTRL").ext_value());
    }

    #[test]
    fn test_sc_display() {
        assert_eq!("SC_ENTER", format!("{}", sc_key!("SC_ENTER")));
    }
}
