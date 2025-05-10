use crate::append_prefix;
use crate::key::KeyCode::{SC, VK};
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KeyCode {
    VK(&'static VirtualKey),
    SC(&'static ScanCode),
}

impl KeyCode {}

impl KeyCode {
    // pub(crate) fn is_scan_code(&self) -> bool {
    //     matches!(*self, SC(_))
    // }

    // pub(crate) fn is_virtual_key(&self) -> bool {
    //     matches!(*self, VK(_))
    // }

    // pub(crate) fn as_virtual_key(&self) -> Option<&'static VirtualKey> {
    //     match self {
    //         VK(vk) => Some(vk),
    //         SC(sc) => sc.to_virtual_key(),
    //     }
    // }

    // pub(crate) fn as_virtual_key(&self) -> Result<&'static VirtualKey, String> {
    //     match self {
    //         VK(vk) => Ok(vk),
    //         SC(sc) => sc.to_virtual_key(),
    //     }
    // }

    // pub(crate) fn as_scan_code(&self) -> Result<&'static ScanCode, String> {
    //     match self {
    //         VK(_) => Err(format!("Illegal key code `{}`.", self)),
    //         SC(sc) => Ok(sc),
    //     }
    // }
}

impl Display for KeyCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            VK(vk) => Display::fmt(&vk, f),
            SC(sc) => Display::fmt(&sc, f),
        }
    }
}

impl FromStr for KeyCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kc = if let Ok(vk) = VirtualKey::from_text(s) {
            VK(vk)
        } else {
            SC(ScanCode::from_text(s)?)
        };
        Ok(kc)
    }
}

impl Serialize for KeyCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            VK(vk) => vk.name,
            SC(sc) => sc.name,
        };

        Ok(s.serialize(serializer)?)
    }
}

impl<'de> Deserialize<'de> for KeyCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(String::deserialize(deserializer)?
            .parse()
            .map_err(|e| de::Error::custom(format!("Error parsing key code.\n{}", e)))?)
    }
}

pub const MAX_VK_CODE: usize = 256;

macro_rules! new_vk {
    ($code:literal, $name:literal) => {
        VirtualKey {
            value: $code,
            name: $name,
        }
    };
}

static VIRTUAL_KEYS: [VirtualKey; MAX_VK_CODE] = [
    new_vk!(0x00, "UNASSIGNED"),
    new_vk!(0x01, "VK_LBUTTON"),
    new_vk!(0x02, "VK_RBUTTON"),
    new_vk!(0x03, "VK_CANCEL"),
    new_vk!(0x04, "VK_MBUTTON"),
    new_vk!(0x05, "VK_XBUTTON1"),
    new_vk!(0x06, "VK_XBUTTON2"),
    new_vk!(0x07, "UNASSIGNED"),
    new_vk!(0x08, "VK_BACK"),
    new_vk!(0x09, "VK_TAB"),
    new_vk!(0x0A, "UNASSIGNED"),
    new_vk!(0x0B, "UNASSIGNED"),
    new_vk!(0x0C, "VK_CLEAR"),
    new_vk!(0x0D, "VK_RETURN"),
    new_vk!(0x0E, "UNASSIGNED"),
    new_vk!(0x0F, "UNASSIGNED"),
    new_vk!(0x10, "VK_SHIFT"),
    new_vk!(0x11, "VK_CONTROL"),
    new_vk!(0x12, "VK_MENU"),
    new_vk!(0x13, "VK_PAUSE"),
    new_vk!(0x14, "VK_CAPITAL"),
    new_vk!(0x15, "VK_KANA"),
    new_vk!(0x16, "VK_IME_ON"),
    new_vk!(0x17, "VK_JUNJA"),
    new_vk!(0x18, "VK_FINAL"),
    new_vk!(0x19, "VK_HANJA"),
    new_vk!(0x1A, "VK_IME_OFF"),
    new_vk!(0x1B, "VK_ESCAPE"),
    new_vk!(0x1C, "VK_CONVERT"),
    new_vk!(0x1D, "VK_NONCONVERT"),
    new_vk!(0x1E, "VK_ACCEPT"),
    new_vk!(0x1F, "VK_MODECHANGE"),
    new_vk!(0x20, "VK_SPACE"),
    new_vk!(0x21, "VK_PRIOR"),
    new_vk!(0x22, "VK_NEXT"),
    new_vk!(0x23, "VK_END"),
    new_vk!(0x24, "VK_HOME"),
    new_vk!(0x25, "VK_LEFT"),
    new_vk!(0x26, "VK_UP"),
    new_vk!(0x27, "VK_RIGHT"),
    new_vk!(0x28, "VK_DOWN"),
    new_vk!(0x29, "VK_SELECT"),
    new_vk!(0x2A, "VK_PRINT"),
    new_vk!(0x2B, "VK_EXECUTE"),
    new_vk!(0x2C, "VK_SNAPSHOT"),
    new_vk!(0x2D, "VK_INSERT"),
    new_vk!(0x2E, "VK_DELETE"),
    new_vk!(0x2F, "VK_HELP"),
    new_vk!(0x30, "VK_0"),
    new_vk!(0x31, "VK_1"),
    new_vk!(0x32, "VK_2"),
    new_vk!(0x33, "VK_3"),
    new_vk!(0x34, "VK_4"),
    new_vk!(0x35, "VK_5"),
    new_vk!(0x36, "VK_6"),
    new_vk!(0x37, "VK_7"),
    new_vk!(0x38, "VK_8"),
    new_vk!(0x39, "VK_9"),
    new_vk!(0x3A, "UNASSIGNED"),
    new_vk!(0x3B, "UNASSIGNED"),
    new_vk!(0x3C, "UNASSIGNED"),
    new_vk!(0x3D, "UNASSIGNED"),
    new_vk!(0x3E, "UNASSIGNED"),
    new_vk!(0x3F, "UNASSIGNED"),
    new_vk!(0x40, "UNASSIGNED"),
    new_vk!(0x41, "VK_A"),
    new_vk!(0x42, "VK_B"),
    new_vk!(0x43, "VK_C"),
    new_vk!(0x44, "VK_D"),
    new_vk!(0x45, "VK_E"),
    new_vk!(0x46, "VK_F"),
    new_vk!(0x47, "VK_G"),
    new_vk!(0x48, "VK_H"),
    new_vk!(0x49, "VK_I"),
    new_vk!(0x4A, "VK_J"),
    new_vk!(0x4B, "VK_K"),
    new_vk!(0x4C, "VK_L"),
    new_vk!(0x4D, "VK_M"),
    new_vk!(0x4E, "VK_N"),
    new_vk!(0x4F, "VK_O"),
    new_vk!(0x50, "VK_P"),
    new_vk!(0x51, "VK_Q"),
    new_vk!(0x52, "VK_R"),
    new_vk!(0x53, "VK_S"),
    new_vk!(0x54, "VK_T"),
    new_vk!(0x55, "VK_U"),
    new_vk!(0x56, "VK_V"),
    new_vk!(0x57, "VK_W"),
    new_vk!(0x58, "VK_X"),
    new_vk!(0x59, "VK_Y"),
    new_vk!(0x5A, "VK_Z"),
    new_vk!(0x5B, "VK_LWIN"),
    new_vk!(0x5C, "VK_RWIN"),
    new_vk!(0x5D, "VK_APPS"),
    new_vk!(0x5E, "UNASSIGNED"),
    new_vk!(0x5F, "VK_SLEEP"),
    new_vk!(0x60, "VK_NUMPAD0"),
    new_vk!(0x61, "VK_NUMPAD1"),
    new_vk!(0x62, "VK_NUMPAD2"),
    new_vk!(0x63, "VK_NUMPAD3"),
    new_vk!(0x64, "VK_NUMPAD4"),
    new_vk!(0x65, "VK_NUMPAD5"),
    new_vk!(0x66, "VK_NUMPAD6"),
    new_vk!(0x67, "VK_NUMPAD7"),
    new_vk!(0x68, "VK_NUMPAD8"),
    new_vk!(0x69, "VK_NUMPAD9"),
    new_vk!(0x6A, "VK_MULTIPLY"),
    new_vk!(0x6B, "VK_ADD"),
    new_vk!(0x6C, "VK_SEPARATOR"),
    new_vk!(0x6D, "VK_SUBTRACT"),
    new_vk!(0x6E, "VK_DECIMAL"),
    new_vk!(0x6F, "VK_DIVIDE"),
    new_vk!(0x70, "VK_F1"),
    new_vk!(0x71, "VK_F2"),
    new_vk!(0x72, "VK_F3"),
    new_vk!(0x73, "VK_F4"),
    new_vk!(0x74, "VK_F5"),
    new_vk!(0x75, "VK_F6"),
    new_vk!(0x76, "VK_F7"),
    new_vk!(0x77, "VK_F8"),
    new_vk!(0x78, "VK_F9"),
    new_vk!(0x79, "VK_F10"),
    new_vk!(0x7A, "VK_F11"),
    new_vk!(0x7B, "VK_F12"),
    new_vk!(0x7C, "VK_F13"),
    new_vk!(0x7D, "VK_F14"),
    new_vk!(0x7E, "VK_F15"),
    new_vk!(0x7F, "VK_F16"),
    new_vk!(0x80, "VK_F17"),
    new_vk!(0x81, "VK_F18"),
    new_vk!(0x82, "VK_F19"),
    new_vk!(0x83, "VK_F20"),
    new_vk!(0x84, "VK_F21"),
    new_vk!(0x85, "VK_F22"),
    new_vk!(0x86, "VK_F23"),
    new_vk!(0x87, "VK_F24"),
    new_vk!(0x88, "UNASSIGNED"),
    new_vk!(0x89, "UNASSIGNED"),
    new_vk!(0x8A, "UNASSIGNED"),
    new_vk!(0x8B, "UNASSIGNED"),
    new_vk!(0x8C, "UNASSIGNED"),
    new_vk!(0x8D, "UNASSIGNED"),
    new_vk!(0x8E, "UNASSIGNED"),
    new_vk!(0x8F, "UNASSIGNED"),
    new_vk!(0x90, "VK_NUMLOCK"),
    new_vk!(0x91, "VK_SCROLL"),
    new_vk!(0x92, "UNASSIGNED"),
    new_vk!(0x93, "UNASSIGNED"),
    new_vk!(0x94, "UNASSIGNED"),
    new_vk!(0x95, "UNASSIGNED"),
    new_vk!(0x96, "UNASSIGNED"),
    new_vk!(0x97, "UNASSIGNED"),
    new_vk!(0x98, "UNASSIGNED"),
    new_vk!(0x99, "UNASSIGNED"),
    new_vk!(0x9A, "UNASSIGNED"),
    new_vk!(0x9B, "UNASSIGNED"),
    new_vk!(0x9C, "UNASSIGNED"),
    new_vk!(0x9D, "UNASSIGNED"),
    new_vk!(0x9E, "UNASSIGNED"),
    new_vk!(0x9F, "UNASSIGNED"),
    new_vk!(0xA0, "VK_LSHIFT"),
    new_vk!(0xA1, "VK_RSHIFT"),
    new_vk!(0xA2, "VK_LCONTROL"),
    new_vk!(0xA3, "VK_RCONTROL"),
    new_vk!(0xA4, "VK_LMENU"),
    new_vk!(0xA5, "VK_RMENU"),
    new_vk!(0xA6, "VK_BROWSER_BACK"),
    new_vk!(0xA7, "VK_BROWSER_FORWARD"),
    new_vk!(0xA8, "VK_BROWSER_REFRESH"),
    new_vk!(0xA9, "VK_BROWSER_STOP"),
    new_vk!(0xAA, "VK_BROWSER_SEARCH"),
    new_vk!(0xAB, "VK_BROWSER_FAVORITES"),
    new_vk!(0xAC, "VK_BROWSER_HOME"),
    new_vk!(0xAD, "VK_VOLUME_MUTE"),
    new_vk!(0xAE, "VK_VOLUME_DOWN"),
    new_vk!(0xAF, "VK_VOLUME_UP"),
    new_vk!(0xB0, "VK_MEDIA_NEXT_TRACK"),
    new_vk!(0xB1, "VK_MEDIA_PREV_TRACK"),
    new_vk!(0xB2, "VK_MEDIA_STOP"),
    new_vk!(0xB3, "VK_MEDIA_PLAY_PAUSE"),
    new_vk!(0xB4, "VK_LAUNCH_MAIL"),
    new_vk!(0xB5, "VK_LAUNCH_MEDIA_SELECT"),
    new_vk!(0xB6, "VK_LAUNCH_APP1"),
    new_vk!(0xB7, "VK_LAUNCH_APP2"),
    new_vk!(0xB8, "UNASSIGNED"),
    new_vk!(0xB9, "UNASSIGNED"),
    new_vk!(0xBA, "VK_OEM_1"),
    new_vk!(0xBB, "VK_OEM_PLUS"),
    new_vk!(0xBC, "VK_OEM_COMMA"),
    new_vk!(0xBD, "VK_OEM_MINUS"),
    new_vk!(0xBE, "VK_OEM_PERIOD"),
    new_vk!(0xBF, "VK_OEM_2"),
    new_vk!(0xC0, "VK_OEM_3"),
    new_vk!(0xC1, "UNASSIGNED"),
    new_vk!(0xC2, "UNASSIGNED"),
    new_vk!(0xC3, "UNASSIGNED"),
    new_vk!(0xC4, "UNASSIGNED"),
    new_vk!(0xC5, "UNASSIGNED"),
    new_vk!(0xC6, "UNASSIGNED"),
    new_vk!(0xC7, "UNASSIGNED"),
    new_vk!(0xC8, "UNASSIGNED"),
    new_vk!(0xC9, "UNASSIGNED"),
    new_vk!(0xCA, "UNASSIGNED"),
    new_vk!(0xCB, "UNASSIGNED"),
    new_vk!(0xCC, "UNASSIGNED"),
    new_vk!(0xCD, "UNASSIGNED"),
    new_vk!(0xCE, "UNASSIGNED"),
    new_vk!(0xCF, "UNASSIGNED"),
    new_vk!(0xD0, "UNASSIGNED"),
    new_vk!(0xD1, "UNASSIGNED"),
    new_vk!(0xD2, "UNASSIGNED"),
    new_vk!(0xD3, "UNASSIGNED"),
    new_vk!(0xD4, "UNASSIGNED"),
    new_vk!(0xD5, "UNASSIGNED"),
    new_vk!(0xD6, "UNASSIGNED"),
    new_vk!(0xD7, "UNASSIGNED"),
    new_vk!(0xD8, "UNASSIGNED"),
    new_vk!(0xD9, "UNASSIGNED"),
    new_vk!(0xDA, "UNASSIGNED"),
    new_vk!(0xDB, "VK_OEM_4"),
    new_vk!(0xDC, "VK_OEM_5"),
    new_vk!(0xDD, "VK_OEM_6"),
    new_vk!(0xDE, "VK_OEM_7"),
    new_vk!(0xDF, "VK_OEM_8"),
    new_vk!(0xE0, "UNASSIGNED"),
    new_vk!(0xE1, "UNASSIGNED"),
    new_vk!(0xE2, "VK_OEM_102"),
    new_vk!(0xE3, "UNASSIGNED"),
    new_vk!(0xE4, "UNASSIGNED"),
    new_vk!(0xE5, "VK_PROCESSKEY"),
    new_vk!(0xE6, "UNASSIGNED"),
    new_vk!(0xE7, "VK_PACKET"),
    new_vk!(0xE8, "UNASSIGNED"),
    new_vk!(0xE9, "UNASSIGNED"),
    new_vk!(0xEA, "UNASSIGNED"),
    new_vk!(0xEB, "UNASSIGNED"),
    new_vk!(0xEC, "UNASSIGNED"),
    new_vk!(0xED, "UNASSIGNED"),
    new_vk!(0xEE, "UNASSIGNED"),
    new_vk!(0xEF, "UNASSIGNED"),
    new_vk!(0xF0, "UNASSIGNED"),
    new_vk!(0xF1, "UNASSIGNED"),
    new_vk!(0xF2, "UNASSIGNED"),
    new_vk!(0xF3, "UNASSIGNED"),
    new_vk!(0xF4, "UNASSIGNED"),
    new_vk!(0xF5, "UNASSIGNED"),
    new_vk!(0xF6, "VK_ATTN"),
    new_vk!(0xF7, "VK_CRSEL"),
    new_vk!(0xF8, "VK_EXSEL"),
    new_vk!(0xF9, "VK_EREOF"),
    new_vk!(0xFA, "VK_PLAY"),
    new_vk!(0xFB, "VK_ZOOM"),
    new_vk!(0xFC, "VK_NONAME"),
    new_vk!(0xFD, "VK_PA1"),
    new_vk!(0xFE, "VK_OEM_CLEAR"),
    new_vk!(0xFF, "VK__none_"),
];

pub const MAX_SCAN_CODE: usize = 136;

macro_rules! new_sc {
    ($code:literal, $name:literal, $ext_code:literal, $ext_name:literal) => {
        [
            ScanCode {
                value: $code,
                is_extended: false,
                name: $name,
            },
            ScanCode {
                value: $ext_code,
                is_extended: true,
                name: $ext_name,
            },
        ]
    };
}

static SCAN_CODES: [[ScanCode; 2]; MAX_SCAN_CODE] = [
    new_sc!(0x00, "UNASSIGNED", 0x00, "UNASSIGNED"),
    new_sc!(0x01, "SC_ESC", 0x01, "SC_"),
    new_sc!(0x02, "SC_1", 0x02, "SC_1"),
    new_sc!(0x03, "SC_2", 0x03, "SC_2"),
    new_sc!(0x04, "SC_3", 0x04, "SC_3"),
    new_sc!(0x05, "SC_4", 0x05, "SC_4"),
    new_sc!(0x06, "SC_5", 0x06, "SC_5"),
    new_sc!(0x07, "SC_6", 0x07, "SC_6"),
    new_sc!(0x08, "SC_7", 0x08, "SC_7"),
    new_sc!(0x09, "SC_8", 0x09, "SC_8"),
    new_sc!(0x0A, "SC_9", 0x0A, "SC_9"),
    new_sc!(0x0B, "SC_0", 0x0B, "SC_0"),
    new_sc!(0x0C, "SC_MINUS", 0x0C, "SC_MINUS"),
    new_sc!(0x0D, "SC_EQ", 0x0D, "SC_EQ"),
    new_sc!(0x0E, "SC_BACKSPACE", 0x0E, "SC"),
    new_sc!(0x0F, "SC_TAB", 0x0F, "SC_	"),
    new_sc!(0x10, "SC_Q", 0x10, "SC_Q"),
    new_sc!(0x11, "SC_W", 0x11, "SC_W"),
    new_sc!(0x12, "SC_E", 0x12, "SC_E"),
    new_sc!(0x13, "SC_R", 0x13, "SC_R"),
    new_sc!(0x14, "SC_T", 0x14, "SC_T"),
    new_sc!(0x15, "SC_Y", 0x15, "SC_Y"),
    new_sc!(0x16, "SC_U", 0x16, "SC_U"),
    new_sc!(0x17, "SC_I", 0x17, "SC_I"),
    new_sc!(0x18, "SC_O", 0x18, "SC_O"),
    new_sc!(0x19, "SC_P", 0x19, "SC_P"),
    new_sc!(0x1A, "SC_L_BRACKET", 0x1A, "SC_L_BRACKET"),
    new_sc!(0x1B, "SC_R_BRACKET", 0x1B, "SC_R_BRACKET"),
    new_sc!(0x1C, "SC_ENTER", 0x1C, "SC_NUM_ENTER"),
    new_sc!(0x1D, "SC_CTRL", 0x1D, "SC_RIGHT_CTRL"),
    new_sc!(0x1E, "SC_A", 0x1E, "SC_A"),
    new_sc!(0x1F, "SC_S", 0x1F, "SC_S"),
    new_sc!(0x20, "SC_D", 0x20, "SC_VOL_MUTE"),
    new_sc!(0x21, "SC_F", 0x21, "SC_CALCULATOR"),
    new_sc!(0x22, "SC_G", 0x22, "SC_G"),
    new_sc!(0x23, "SC_H", 0x23, "SC_H"),
    new_sc!(0x24, "SC_J", 0x24, "SC_J"),
    new_sc!(0x25, "SC_K", 0x25, "SC_K"),
    new_sc!(0x26, "SC_L", 0x26, "SC_L"),
    new_sc!(0x27, "SC_SEMICOLON", 0x27, "SC_SEMICOLON"),
    new_sc!(0x28, "SC_APOSTROPHE", 0x28, "SC_APOSTROPHE"),
    new_sc!(0x29, "SC_BACKTICK", 0x29, "SC_BACKTICK"),
    new_sc!(0x2A, "SC_SHIFT", 0x2A, "UNASSIGNED"),
    new_sc!(0x2B, "SC_BACKSLASH", 0x2B, "SC_BRIGHTNESS"),
    new_sc!(0x2C, "SC_Z", 0x2C, "SC_Z"),
    new_sc!(0x2D, "SC_X", 0x2D, "SC_X"),
    new_sc!(0x2E, "SC_C", 0x2E, "SC_VOLUME_DOWN"),
    new_sc!(0x2F, "SC_V", 0x2F, "SC_V"),
    new_sc!(0x30, "SC_B", 0x30, "SC_VOLUME_UP"),
    new_sc!(0x31, "SC_N", 0x31, "SC_N"),
    new_sc!(0x32, "SC_M", 0x32, "SC_M"),
    new_sc!(0x33, "SC_COMMA", 0x33, "SC_COMMA"),
    new_sc!(0x34, "SC_DOT", 0x34, "SC_DOT"),
    new_sc!(0x35, "SC_SLASH", 0x35, "SC_NUM_SLASH"),
    new_sc!(0x36, "SC_RIGHT_SHIFT", 0x36, "SC_RIGHT_SHIFT"),
    new_sc!(0x37, "SC_NUM_MUL", 0x37, "SC_PRNT_SCRN"),
    new_sc!(0x38, "SC_ALT", 0x38, "SC_RIGHT_ALT"),
    new_sc!(0x39, "SC_SPACE", 0x39, "SC__"),
    new_sc!(0x3A, "SC_CAPS_LOCK", 0x3A, "UNASSIGNED"),
    new_sc!(0x3B, "SC_F1", 0x3B, "UNASSIGNED"),
    new_sc!(0x3C, "SC_F2", 0x3C, "UNASSIGNED"),
    new_sc!(0x3D, "SC_F3", 0x3D, "UNASSIGNED"),
    new_sc!(0x3E, "SC_F4", 0x3E, "UNASSIGNED"),
    new_sc!(0x3F, "SC_F5", 0x3F, "UNASSIGNED"),
    new_sc!(0x40, "SC_F6", 0x40, "UNASSIGNED"),
    new_sc!(0x41, "SC_F7", 0x41, "UNASSIGNED"),
    new_sc!(0x42, "SC_F8", 0x42, "UNASSIGNED"),
    new_sc!(0x43, "SC_F9", 0x43, "UNASSIGNED"),
    new_sc!(0x44, "SC_F10", 0x44, "UNASSIGNED"),
    new_sc!(0x45, "SC_PAUSE", 0x45, "SC_NUM_LOCK"),
    new_sc!(0x46, "SC_SCROLL_LOCK", 0x46, "SC_BREAK"),
    new_sc!(0x47, "SC_NUM_7", 0x47, "SC_HOME"),
    new_sc!(0x48, "SC_NUM_8", 0x48, "SC_UP"),
    new_sc!(0x49, "SC_NUM_9", 0x49, "SC_PAGE_UP"),
    new_sc!(0x4A, "SC_NUM_MINUS", 0x4A, "SC_MINUS"),
    new_sc!(0x4B, "SC_NUM_4", 0x4B, "SC_LEFT"),
    new_sc!(0x4C, "SC_NUM_5", 0x4C, "UNASSIGNED"),
    new_sc!(0x4D, "SC_NUM_6", 0x4D, "SC_RIGHT"),
    new_sc!(0x4E, "SC_NUM_PLUS", 0x4E, "SC_PLUS"),
    new_sc!(0x4F, "SC_NUM_1", 0x4F, "SC_END"),
    new_sc!(0x50, "SC_NUM_2", 0x50, "SC_DOWN"),
    new_sc!(0x51, "SC_NUM_3", 0x51, "SC_PAGE_DOWN"),
    new_sc!(0x52, "SC_NUM_0", 0x52, "SC_INSERT"),
    new_sc!(0x53, "SC_NUM_DEL", 0x53, "SC_DELETE"),
    new_sc!(0x54, "SC_SYS_REQ", 0x54, "SC_<00>"),
    new_sc!(0x55, "UNASSIGNED", 0x55, "UNASSIGNED"),
    new_sc!(0x56, "SC_BACKSLASH", 0x56, "SC_HELP"),
    new_sc!(0x57, "SC_F11", 0x57, "UNASSIGNED"),
    new_sc!(0x58, "SC_F12", 0x58, "UNASSIGNED"),
    new_sc!(0x59, "UNASSIGNED", 0x59, "UNASSIGNED"),
    new_sc!(0x5A, "UNASSIGNED", 0x5A, "UNASSIGNED"),
    new_sc!(0x5B, "UNASSIGNED", 0x5B, "SC_LEFT_WINDOWS"),
    new_sc!(0x5C, "UNASSIGNED", 0x5C, "SC_RIGHT_WINDOWS"),
    new_sc!(0x5D, "UNASSIGNED", 0x5D, "SC_APPLICATION"),
    new_sc!(0x5E, "UNASSIGNED", 0x5E, "UNASSIGNED"),
    new_sc!(0x5F, "UNASSIGNED", 0x5F, "UNASSIGNED"),
    new_sc!(0x60, "UNASSIGNED", 0x60, "UNASSIGNED"),
    new_sc!(0x61, "UNASSIGNED", 0x61, "UNASSIGNED"),
    new_sc!(0x62, "UNASSIGNED", 0x62, "UNASSIGNED"),
    new_sc!(0x63, "UNASSIGNED", 0x63, "UNASSIGNED"),
    new_sc!(0x64, "UNASSIGNED", 0x64, "UNASSIGNED"),
    new_sc!(0x65, "UNASSIGNED", 0x65, "UNASSIGNED"),
    new_sc!(0x66, "UNASSIGNED", 0x66, "UNASSIGNED"),
    new_sc!(0x67, "UNASSIGNED", 0x67, "UNASSIGNED"),
    new_sc!(0x68, "UNASSIGNED", 0x68, "UNASSIGNED"),
    new_sc!(0x69, "UNASSIGNED", 0x69, "UNASSIGNED"),
    new_sc!(0x6A, "UNASSIGNED", 0x6A, "UNASSIGNED"),
    new_sc!(0x6B, "UNASSIGNED", 0x6B, "UNASSIGNED"),
    new_sc!(0x6C, "UNASSIGNED", 0x6C, "UNASSIGNED"),
    new_sc!(0x6D, "UNASSIGNED", 0x6D, "UNASSIGNED"),
    new_sc!(0x6E, "UNASSIGNED", 0x6E, "UNASSIGNED"),
    new_sc!(0x6F, "UNASSIGNED", 0x6F, "UNASSIGNED"),
    new_sc!(0x70, "UNASSIGNED", 0x70, "UNASSIGNED"),
    new_sc!(0x71, "UNASSIGNED", 0x71, "UNASSIGNED"),
    new_sc!(0x72, "UNASSIGNED", 0x72, "UNASSIGNED"),
    new_sc!(0x73, "UNASSIGNED", 0x73, "UNASSIGNED"),
    new_sc!(0x74, "UNASSIGNED", 0x74, "UNASSIGNED"),
    new_sc!(0x75, "UNASSIGNED", 0x75, "UNASSIGNED"),
    new_sc!(0x76, "UNASSIGNED", 0x76, "UNASSIGNED"),
    new_sc!(0x77, "UNASSIGNED", 0x77, "UNASSIGNED"),
    new_sc!(0x78, "UNASSIGNED", 0x78, "UNASSIGNED"),
    new_sc!(0x79, "UNASSIGNED", 0x79, "UNASSIGNED"),
    new_sc!(0x7A, "UNASSIGNED", 0x7A, "UNASSIGNED"),
    new_sc!(0x7B, "UNASSIGNED", 0x7B, "UNASSIGNED"),
    new_sc!(0x7C, "SC_F13", 0x7C, "SC_	"),
    new_sc!(0x7D, "SC_F14", 0x7D, "UNASSIGNED"),
    new_sc!(0x7E, "SC_F15", 0x7E, "UNASSIGNED"),
    new_sc!(0x7F, "SC_F16", 0x7F, "UNASSIGNED"),
    new_sc!(0x80, "SC_F17", 0x80, "UNASSIGNED"),
    new_sc!(0x81, "SC_F18", 0x81, "UNASSIGNED"),
    new_sc!(0x82, "SC_F19", 0x82, "UNASSIGNED"),
    new_sc!(0x83, "SC_F20", 0x83, "UNASSIGNED"),
    new_sc!(0x84, "SC_F21", 0x84, "UNASSIGNED"),
    new_sc!(0x85, "SC_F22", 0x85, "UNASSIGNED"),
    new_sc!(0x86, "SC_F23", 0x86, "UNASSIGNED"),
    new_sc!(0x87, "SC_F24", 0x87, "UNASSIGNED"),
];

#[cfg(test)]
mod tests {
    use crate::key::KeyCode::{SC, VK};
    use crate::key::{KeyCode, ScanCode, VirtualKey};
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
            $text.parse::<KeyCode>().unwrap()
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
        let actual = KeyCode::from_str("VK_RETURN").unwrap();
        assert!(matches!(actual, VK(_)));
        if let VK(vk) = actual {
            assert_eq!("VK_RETURN", vk.name);
        }

        let actual = KeyCode::from_str("SC_ENTER").unwrap();
        assert!(matches!(actual, SC(_)));
        if let SC(sc) = actual {
            assert_eq!("SC_ENTER", sc.name);
        }

        let actual = KeyCode::from_str("`").unwrap();
        assert!(matches!(actual, SC(_)));
        if let SC(sc) = actual {
            assert_eq!("SC_BACKTICK", sc.name);
        }
    }

    #[test]
    #[should_panic]
    fn test_key_code_parse_fails() {
        KeyCode::from_str("↑").unwrap();
    }

    #[test]
    fn test_key_code_display() {
        assert_eq!("SC_ENTER", format!("{}", key!("SC_ENTER")));
        assert_eq!("VK_RETURN", format!("{}", key!("VK_RETURN")));
    }

    #[test]
    fn test_key_code_serialize() {
        let source = key!("SC_ENTER");
        let json = serde_json::to_string_pretty(&source).unwrap();

        // dbg!(&json);

        let actual = serde_json::from_str::<KeyCode>(&json).unwrap();
        assert_eq!(source, actual);

        let source = key!("VK_RETURN");
        let json = serde_json::to_string_pretty(&source).unwrap();

        // dbg!(&json);

        let actual = serde_json::from_str::<KeyCode>(&json).unwrap();
        assert_eq!(source, actual);
    }
}
