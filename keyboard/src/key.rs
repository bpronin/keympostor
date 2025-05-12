use crate::append_prefix;
use crate::key_const::{KEYS, SCAN_CODES, VIRTUAL_KEYS};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    MapVirtualKeyW, OemKeyScan, MAPVK_VK_TO_VSC_EX, MAPVK_VSC_TO_VK_EX,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct VirtualKey {
    pub value: u8,
    pub name: &'static str,
}

impl VirtualKey {
    pub fn from_code(code: u8) -> Result<&'static VirtualKey, String> {
        VIRTUAL_KEYS
            .get(code as usize)
            .ok_or(format!("Illegal virtual key code `{}`.", code))
    }

    pub fn from_name(name: &str) -> Result<&'static VirtualKey, String> {
        let vk_name = append_prefix!(name, "VK_");
        let position = VIRTUAL_KEYS.iter().position(|probe| probe.name == vk_name);

        if let Some(ix) = position {
            Ok(&VIRTUAL_KEYS[ix])
        } else {
            Err(format!("Illegal virtual key name `{}`.", name))
        }
    }

    pub fn from_code_name(s: &str) -> Result<&'static VirtualKey, String> {
        let src = s.strip_prefix("VK_0x").ok_or("No `VK_0x` prefix.")?;
        let code = u8::from_str_radix(src, 16)
            .map_err(|_| format!("Error parsing virtual key code `{}`.", s))?;
        Self::from_code(code)
    }

    fn from_text(s: &str) -> Result<&'static VirtualKey, String> {
        let st = s.trim();
        Self::from_code_name(st).or_else(|_| Self::from_name(st))
    }

    pub fn code_name(&self) -> String {
        format!("VC_0x{:02X}", self.value)
    }

    pub fn to_scan_code(&self) -> Result<&'static ScanCode, String> {
        let ext_code = unsafe { MapVirtualKeyW(self.value as u32, MAPVK_VK_TO_VSC_EX) };
        if ext_code > 0 {
            let code = ext_code as u8;
            let is_extended = ext_code & 0xE000 != 0;
            ScanCode::from_code(code, is_extended)
        } else {
            Err(format!("Unable to convert virtual key {self} to scancode."))
        }
    }
}

impl Display for VirtualKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.name, f)
    }
}

impl FromStr for VirtualKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_text(s).copied()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ScanCode {
    pub value: u8,
    pub is_extended: bool,
    pub name: &'static str,
}

impl ScanCode {
    pub(crate) fn from_code(code: u8, extended: bool) -> Result<&'static ScanCode, String> {
        SCAN_CODES
            .get(code as usize)
            .ok_or(format!("Illegal scan code `{}`.", code))?
            .get(extended as usize)
            .ok_or(format!("Illegal extended scan code `{}`.", code))
    }

    pub(crate) fn from_name(name: &str) -> Result<&'static ScanCode, String> {
        let sc_name = append_prefix!(name, "SC_");
        SCAN_CODES
            .iter()
            .flatten()
            .find(|sc| sc.name == sc_name)
            .ok_or(format!("Illegal scan code name `{}`.", name))
    }

    pub fn from_code_name(s: &str) -> Result<&'static ScanCode, String> {
        let code = u16::from_str_radix(s.strip_prefix("SC_0x").ok_or("No `SC_0x` prefix.")?, 16)
            .map_err(|_| format!("Error parsing scan code `{}`.", s))?;
        Self::from_ext_code(code)
    }

    pub(crate) fn from_ext_code(ext_code: u16) -> Result<&'static ScanCode, String> {
        Self::from_code(ext_code as u8, ext_code & 0xE000 != 0)
    }

    pub(crate) fn from_symbol(symbol: &str) -> Result<&'static ScanCode, String> {
        if symbol.len() == 1 {
            let ch = symbol.chars().next().unwrap();
            let oem_code = unsafe { OemKeyScan(ch as u16) };
            let ext_code = oem_code as u8;
            //todo? let is_shift = oem_code & 0x0001_0000 != 0;
            ScanCode::from_code(ext_code, false)
        } else {
            Err(format!("Illegal key symbol `{}`.", symbol))
        }
    }

    fn from_text(s: &str) -> Result<&'static ScanCode, String> {
        let st = s.trim();
        Self::from_code_name(st).or_else(|_| Self::from_name(st).or_else(|_| Self::from_symbol(st)))
    }

    pub fn ext_value(&self) -> u16 {
        if self.is_extended {
            self.value as u16 | 0xE0 << 8
        } else {
            self.value as u16
        }
    }

    pub fn code_name(&self) -> String {
        format!("SC_0x{:04X}", self.ext_value())
    }

    pub fn to_virtual_key(&self) -> Result<&'static VirtualKey, String> {
        let vk_code = unsafe { MapVirtualKeyW(self.ext_value() as u32, MAPVK_VSC_TO_VK_EX) };
        if vk_code > 0 {
            VirtualKey::from_code(vk_code as u8)
        } else {
            Err(format!("Unable to convert scancode {self} to virtual key."))
        }
    }
}

impl Display for ScanCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.name, f)
    }
}

impl FromStr for ScanCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_text(s).copied()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Key {
    pub(crate) vk_code: u8,
    pub(crate) scan_code: u8,
    pub(crate) is_ext_scan_code: bool,
}

impl Key {
    pub fn name(&self) -> &'static str {
        KEYS.name_of(self)
    }

    pub fn virtual_key(&self) -> &'static VirtualKey {
        VirtualKey::from_code(self.vk_code).unwrap()
    }

    pub fn scan_code(&self) -> &'static ScanCode {
        ScanCode::from_code(self.scan_code, self.is_ext_scan_code).unwrap()
    }

    pub(crate) fn code_name(&self) -> String {
        format!("{} - {}", self.virtual_key().code_name(), self.scan_code().code_name())
    }

}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for Key {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(*KEYS.by_name(s.trim()))
    }
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.name().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = String::deserialize(deserializer)?;
        let key = name.parse().map_err(de::Error::custom)?;
        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use crate::key::{Key, ScanCode, VirtualKey};
    use std::str::FromStr;

    #[macro_export]
    macro_rules! vk_key {
        ($text:literal) => {
            &$text.parse::<VirtualKey>().unwrap()
        };
    }

    #[macro_export]
    macro_rules! sc_key {
        ($text:literal) => {
            &$text.parse::<ScanCode>().unwrap()
        };
    }

    #[macro_export]
    macro_rules! key {
        ($text:literal) => {
            $text.parse::<Key>().unwrap()
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
        assert_eq!("VK_RETURN", VirtualKey::from_str("VK_0x0D").unwrap().name);
    }

    #[test]
    fn test_vk_parse() {
        assert_eq!("VK_RETURN", VirtualKey::from_str("VK_RETURN").unwrap().name);
        assert_eq!("VK_RETURN", VirtualKey::from_str("RETURN").unwrap().name);
        assert_eq!("VK_RETURN", VirtualKey::from_str("VK_0x0D").unwrap().name);
    }

    #[test]
    #[should_panic]
    fn test_vk_parse_fails() {
        VirtualKey::from_str("BANANA").unwrap();
    }

    #[test]
    fn test_vk_display() {
        assert_eq!(
            "VK_RETURN",
            format!("{}", VirtualKey::from_str("VK_RETURN").unwrap())
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
    fn test_sc_from_symbol() {
        let actual = ScanCode::from_symbol("A").unwrap();
        assert_eq!("SC_A", actual.name);

        let actual = ScanCode::from_symbol("`").unwrap();
        assert_eq!("SC_BACKTICK", actual.name);

        let actual = ScanCode::from_symbol("~").unwrap();
        assert_eq!("SC_BACKTICK", actual.name);
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
    fn test_sc_parse() {
        assert_eq!("SC_ENTER", ScanCode::from_str("SC_ENTER").unwrap().name);
        assert_eq!("SC_ENTER", ScanCode::from_str("ENTER").unwrap().name);
        assert_eq!(
            "SC_NUM_ENTER",
            ScanCode::from_str("SC_0xE01C").unwrap().name
        );
        assert_eq!("SC_BACKTICK", ScanCode::from_str("`").unwrap().name);
    }

    #[test]
    #[should_panic]
    fn test_sc_parse_fails() {
        ScanCode::from_str("BANANA").unwrap();
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
    fn test_key_code_parse() {
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
    }

    #[test]
    #[should_panic]
    fn test_key_parse_fails() {
        Key::from_str("BANANA").unwrap();
    }
    
    #[test]
    fn test_key_display() {
        assert_eq!("ENTER", format!("{}", Key {
            vk_code: 0x0D,
            scan_code: 0x1C,
            is_ext_scan_code: false,
        }));
        
        assert_eq!("NUM_ENTER", format!("{}", Key {
            vk_code: 0x0D,
            scan_code: 0x1C,
            is_ext_scan_code: true,
        }));
    }
    
    #[test]
    fn test_key_serialize() {
        let source = key!("ENTER");
        let json = serde_json::to_string_pretty(&source).unwrap();
    
        // dbg!(&json);
    
        let actual = serde_json::from_str::<Key>(&json).unwrap();
        assert_eq!(source, actual);
    
        let source = key!("NUM_ENTER");
        let json = serde_json::to_string_pretty(&source).unwrap();
    
        // dbg!(&json);
    
        let actual = serde_json::from_str::<Key>(&json).unwrap();
        assert_eq!(source, actual);
    }
}
