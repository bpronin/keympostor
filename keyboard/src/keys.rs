use crate::keys::KeyCode::{SC, VK};
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct VirtualKey {
    pub value: u8,
    pub name: &'static str,
}

impl VirtualKey {
    pub(crate) fn parse(text: &str) -> Result<&'static VirtualKey, String> {
        Self::by_code_name(text).or_else(|_| Self::by_name(text))
    }

    pub fn by_code_name(text: &str) -> Result<&'static VirtualKey, String> {
        let code = u8::from_str_radix(text.strip_prefix("VK_0x").ok_or("No `VK_0x` prefix")?, 16)
            .map_err(|_| format!("Failed to parse virtual key code: {}.", text))?;
        Self::by_code(code)
    }

    pub fn by_code(code: u8) -> Result<&'static VirtualKey, String> {
        VIRTUAL_KEYS
            .get(code as usize)
            .ok_or(format!("Unsupported virtual key code: `{}`", code))
    }

    pub fn by_name(name: &str) -> Result<&'static VirtualKey, String> {
        let position = VIRTUAL_KEYS.iter().position(|probe| probe.name == name);

        if let Some(ix) = position {
            Ok(&VIRTUAL_KEYS[ix])
        } else {
            Err(format!("Unsupported virtual key name: `{}`", name))
        }
    }
}

impl Display for VirtualKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:02X}", &self.value,)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ScanCode {
    pub value: u8,
    pub is_extended: bool,
    pub name: &'static str,
}

impl ScanCode {
    fn parse(text: &str) -> Result<&'static ScanCode, String> {
        // .or(|e| ScanCode::parse(text))
        // if s.starts_with("SC_0x") {
        //     Self::parse_scancode_from_code(s)
        // } else if s.starts_with("VK_0x") {
        //     Self::parse_virtual_key_from_code(s)
        // } else if s.starts_with("SC_") {
        //     Self::parse_scancode_from_name(s)
        // } else if s.starts_with("VK_") {
        //     Self::parse_virtual_key_from_name(s)
        // } else {
        //     Self::parse_scancode_from_symbol(s).or_else(|_| {
        //         Self::parse(Some(&format!("VK_{}", s))).or_else(|_| {
        //             Self::parse(Some(&format!("SC_{}", s)))
        //         })
        //     })
        // }

        todo!()
    }

    pub(crate) fn by_code(code: u8, extended: bool) -> Result<&'static ScanCode, String> {
        SCANCODES
            .get(code as usize)
            .ok_or(format!("Unsupported scancode: `{}`", code))?
            .get(extended as usize)
            .ok_or(format!("Unsupported scancode: `{}`", code))
    }

    pub(crate) fn by_name(name: &str) -> Result<&'static ScanCode, String> {
        for row in &SCANCODES {
            let sc = &row[false as usize];
            if sc.name == name {
                return Ok(sc);
            }
            let sc = &row[true as usize];
            if sc.name == name {
                return Ok(sc);
            }
        }

        Err(format!("Unsupported scancode name: `{}`.", name))
    }

    pub(crate) fn by_ext_code(ext_code: u16) -> Result<&'static ScanCode, String> {
        Self::by_code(ext_code as u8, ext_code & 0xE000 != 0)
    }

    // pub(crate) fn by_symbol(symbol: char) -> Result<&'static ScanCode, String> {
    //     let ext_code = unsafe { OemKeyScan(symbol as u16) } as u16;
    //     ScanCode::by_ext_code(ext_code)
    // }

    pub(crate) fn ext_value(&self) -> u16 {
        if self.is_extended {
            self.value as u16 | 0xE0 << 8
        } else {
            self.value as u16
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum KeyCode {
    VK(&'static VirtualKey),
    SC(&'static ScanCode),
}

impl KeyCode {
    fn parse(text: &str) -> Result<Self, String> {
        VirtualKey::parse(text)
            .and_then(|vk| Ok(VK(vk)))
            .or_else(|_| ScanCode::parse(text).and_then(|sc| Ok(SC(sc))))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KeyTransition {
    Down,
    Up,
}

#[derive(Debug, Eq, PartialEq)]
pub struct KeyEvent {
    pub virtual_key: VirtualKey,
    pub scan_code: ScanCode,
    pub transition: KeyTransition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum KeyEventPattern {
    Sequence(Vec<KeyEvent>),
    Chord(Vec<KeyEvent>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct KeyAction {
    pub key_code: KeyCode,
    pub transition: KeyTransition,
}

#[derive(Debug, Eq, PartialEq)]
pub struct KeyTransformRule {
    pub trigger: KeyEventPattern,
    pub action: Vec<KeyAction>,
}

pub const MAX_VK_CODE: usize = 256;
pub const MAX_SCAN_CODE: usize = 136;

macro_rules! vk {
    ($code:literal, $name:literal) => {
        VirtualKey {
            value: $code,
            name: $name,
        }
    };
}

static VIRTUAL_KEYS: [VirtualKey; MAX_VK_CODE] = [
    vk!(0x00, "UNASSIGNED"),
    vk!(0x01, "VK_LBUTTON"),
    vk!(0x02, "VK_RBUTTON"),
    vk!(0x03, "VK_CANCEL"),
    vk!(0x04, "VK_MBUTTON"),
    vk!(0x05, "VK_XBUTTON1"),
    vk!(0x06, "VK_XBUTTON2"),
    vk!(0x07, "UNASSIGNED"),
    vk!(0x08, "VK_BACK"),
    vk!(0x09, "VK_TAB"),
    vk!(0x0A, "UNASSIGNED"),
    vk!(0x0B, "UNASSIGNED"),
    vk!(0x0C, "VK_CLEAR"),
    vk!(0x0D, "VK_RETURN"),
    vk!(0x0E, "UNASSIGNED"),
    vk!(0x0F, "UNASSIGNED"),
    vk!(0x10, "VK_SHIFT"),
    vk!(0x11, "VK_CONTROL"),
    vk!(0x12, "VK_MENU"),
    vk!(0x13, "VK_PAUSE"),
    vk!(0x14, "VK_CAPITAL"),
    vk!(0x15, "VK_KANA"),
    vk!(0x16, "VK_IME_ON"),
    vk!(0x17, "VK_JUNJA"),
    vk!(0x18, "VK_FINAL"),
    vk!(0x19, "VK_HANJA"),
    vk!(0x1A, "VK_IME_OFF"),
    vk!(0x1B, "VK_ESCAPE"),
    vk!(0x1C, "VK_CONVERT"),
    vk!(0x1D, "VK_NONCONVERT"),
    vk!(0x1E, "VK_ACCEPT"),
    vk!(0x1F, "VK_MODECHANGE"),
    vk!(0x20, "VK_SPACE"),
    vk!(0x21, "VK_PRIOR"),
    vk!(0x22, "VK_NEXT"),
    vk!(0x23, "VK_END"),
    vk!(0x24, "VK_HOME"),
    vk!(0x25, "VK_LEFT"),
    vk!(0x26, "VK_UP"),
    vk!(0x27, "VK_RIGHT"),
    vk!(0x28, "VK_DOWN"),
    vk!(0x29, "VK_SELECT"),
    vk!(0x2A, "VK_PRINT"),
    vk!(0x2B, "VK_EXECUTE"),
    vk!(0x2C, "VK_SNAPSHOT"),
    vk!(0x2D, "VK_INSERT"),
    vk!(0x2E, "VK_DELETE"),
    vk!(0x2F, "VK_HELP"),
    vk!(0x30, "VK_0"),
    vk!(0x31, "VK_1"),
    vk!(0x32, "VK_2"),
    vk!(0x33, "VK_3"),
    vk!(0x34, "VK_4"),
    vk!(0x35, "VK_5"),
    vk!(0x36, "VK_6"),
    vk!(0x37, "VK_7"),
    vk!(0x38, "VK_8"),
    vk!(0x39, "VK_9"),
    vk!(0x3A, "UNASSIGNED"),
    vk!(0x3B, "UNASSIGNED"),
    vk!(0x3C, "UNASSIGNED"),
    vk!(0x3D, "UNASSIGNED"),
    vk!(0x3E, "UNASSIGNED"),
    vk!(0x3F, "UNASSIGNED"),
    vk!(0x40, "UNASSIGNED"),
    vk!(0x41, "VK_A"),
    vk!(0x42, "VK_B"),
    vk!(0x43, "VK_C"),
    vk!(0x44, "VK_D"),
    vk!(0x45, "VK_E"),
    vk!(0x46, "VK_F"),
    vk!(0x47, "VK_G"),
    vk!(0x48, "VK_H"),
    vk!(0x49, "VK_I"),
    vk!(0x4A, "VK_J"),
    vk!(0x4B, "VK_K"),
    vk!(0x4C, "VK_L"),
    vk!(0x4D, "VK_M"),
    vk!(0x4E, "VK_N"),
    vk!(0x4F, "VK_O"),
    vk!(0x50, "VK_P"),
    vk!(0x51, "VK_Q"),
    vk!(0x52, "VK_R"),
    vk!(0x53, "VK_S"),
    vk!(0x54, "VK_T"),
    vk!(0x55, "VK_U"),
    vk!(0x56, "VK_V"),
    vk!(0x57, "VK_W"),
    vk!(0x58, "VK_X"),
    vk!(0x59, "VK_Y"),
    vk!(0x5A, "VK_Z"),
    vk!(0x5B, "VK_LWIN"),
    vk!(0x5C, "VK_RWIN"),
    vk!(0x5D, "VK_APPS"),
    vk!(0x5E, "UNASSIGNED"),
    vk!(0x5F, "VK_SLEEP"),
    vk!(0x60, "VK_NUMPAD0"),
    vk!(0x61, "VK_NUMPAD1"),
    vk!(0x62, "VK_NUMPAD2"),
    vk!(0x63, "VK_NUMPAD3"),
    vk!(0x64, "VK_NUMPAD4"),
    vk!(0x65, "VK_NUMPAD5"),
    vk!(0x66, "VK_NUMPAD6"),
    vk!(0x67, "VK_NUMPAD7"),
    vk!(0x68, "VK_NUMPAD8"),
    vk!(0x69, "VK_NUMPAD9"),
    vk!(0x6A, "VK_MULTIPLY"),
    vk!(0x6B, "VK_ADD"),
    vk!(0x6C, "VK_SEPARATOR"),
    vk!(0x6D, "VK_SUBTRACT"),
    vk!(0x6E, "VK_DECIMAL"),
    vk!(0x6F, "VK_DIVIDE"),
    vk!(0x70, "VK_F1"),
    vk!(0x71, "VK_F2"),
    vk!(0x72, "VK_F3"),
    vk!(0x73, "VK_F4"),
    vk!(0x74, "VK_F5"),
    vk!(0x75, "VK_F6"),
    vk!(0x76, "VK_F7"),
    vk!(0x77, "VK_F8"),
    vk!(0x78, "VK_F9"),
    vk!(0x79, "VK_F10"),
    vk!(0x7A, "VK_F11"),
    vk!(0x7B, "VK_F12"),
    vk!(0x7C, "VK_F13"),
    vk!(0x7D, "VK_F14"),
    vk!(0x7E, "VK_F15"),
    vk!(0x7F, "VK_F16"),
    vk!(0x80, "VK_F17"),
    vk!(0x81, "VK_F18"),
    vk!(0x82, "VK_F19"),
    vk!(0x83, "VK_F20"),
    vk!(0x84, "VK_F21"),
    vk!(0x85, "VK_F22"),
    vk!(0x86, "VK_F23"),
    vk!(0x87, "VK_F24"),
    vk!(0x88, "UNASSIGNED"),
    vk!(0x89, "UNASSIGNED"),
    vk!(0x8A, "UNASSIGNED"),
    vk!(0x8B, "UNASSIGNED"),
    vk!(0x8C, "UNASSIGNED"),
    vk!(0x8D, "UNASSIGNED"),
    vk!(0x8E, "UNASSIGNED"),
    vk!(0x8F, "UNASSIGNED"),
    vk!(0x90, "VK_NUMLOCK"),
    vk!(0x91, "VK_SCROLL"),
    vk!(0x92, "UNASSIGNED"),
    vk!(0x93, "UNASSIGNED"),
    vk!(0x94, "UNASSIGNED"),
    vk!(0x95, "UNASSIGNED"),
    vk!(0x96, "UNASSIGNED"),
    vk!(0x97, "UNASSIGNED"),
    vk!(0x98, "UNASSIGNED"),
    vk!(0x99, "UNASSIGNED"),
    vk!(0x9A, "UNASSIGNED"),
    vk!(0x9B, "UNASSIGNED"),
    vk!(0x9C, "UNASSIGNED"),
    vk!(0x9D, "UNASSIGNED"),
    vk!(0x9E, "UNASSIGNED"),
    vk!(0x9F, "UNASSIGNED"),
    vk!(0xA0, "VK_LSHIFT"),
    vk!(0xA1, "VK_RSHIFT"),
    vk!(0xA2, "VK_LCONTROL"),
    vk!(0xA3, "VK_RCONTROL"),
    vk!(0xA4, "VK_LMENU"),
    vk!(0xA5, "VK_RMENU"),
    vk!(0xA6, "VK_BROWSER_BACK"),
    vk!(0xA7, "VK_BROWSER_FORWARD"),
    vk!(0xA8, "VK_BROWSER_REFRESH"),
    vk!(0xA9, "VK_BROWSER_STOP"),
    vk!(0xAA, "VK_BROWSER_SEARCH"),
    vk!(0xAB, "VK_BROWSER_FAVORITES"),
    vk!(0xAC, "VK_BROWSER_HOME"),
    vk!(0xAD, "VK_VOLUME_MUTE"),
    vk!(0xAE, "VK_VOLUME_DOWN"),
    vk!(0xAF, "VK_VOLUME_UP"),
    vk!(0xB0, "VK_MEDIA_NEXT_TRACK"),
    vk!(0xB1, "VK_MEDIA_PREV_TRACK"),
    vk!(0xB2, "VK_MEDIA_STOP"),
    vk!(0xB3, "VK_MEDIA_PLAY_PAUSE"),
    vk!(0xB4, "VK_LAUNCH_MAIL"),
    vk!(0xB5, "VK_LAUNCH_MEDIA_SELECT"),
    vk!(0xB6, "VK_LAUNCH_APP1"),
    vk!(0xB7, "VK_LAUNCH_APP2"),
    vk!(0xB8, "UNASSIGNED"),
    vk!(0xB9, "UNASSIGNED"),
    vk!(0xBA, "VK_OEM_1"),
    vk!(0xBB, "VK_OEM_PLUS"),
    vk!(0xBC, "VK_OEM_COMMA"),
    vk!(0xBD, "VK_OEM_MINUS"),
    vk!(0xBE, "VK_OEM_PERIOD"),
    vk!(0xBF, "VK_OEM_2"),
    vk!(0xC0, "VK_OEM_3"),
    vk!(0xC1, "UNASSIGNED"),
    vk!(0xC2, "UNASSIGNED"),
    vk!(0xC3, "UNASSIGNED"),
    vk!(0xC4, "UNASSIGNED"),
    vk!(0xC5, "UNASSIGNED"),
    vk!(0xC6, "UNASSIGNED"),
    vk!(0xC7, "UNASSIGNED"),
    vk!(0xC8, "UNASSIGNED"),
    vk!(0xC9, "UNASSIGNED"),
    vk!(0xCA, "UNASSIGNED"),
    vk!(0xCB, "UNASSIGNED"),
    vk!(0xCC, "UNASSIGNED"),
    vk!(0xCD, "UNASSIGNED"),
    vk!(0xCE, "UNASSIGNED"),
    vk!(0xCF, "UNASSIGNED"),
    vk!(0xD0, "UNASSIGNED"),
    vk!(0xD1, "UNASSIGNED"),
    vk!(0xD2, "UNASSIGNED"),
    vk!(0xD3, "UNASSIGNED"),
    vk!(0xD4, "UNASSIGNED"),
    vk!(0xD5, "UNASSIGNED"),
    vk!(0xD6, "UNASSIGNED"),
    vk!(0xD7, "UNASSIGNED"),
    vk!(0xD8, "UNASSIGNED"),
    vk!(0xD9, "UNASSIGNED"),
    vk!(0xDA, "UNASSIGNED"),
    vk!(0xDB, "VK_OEM_4"),
    vk!(0xDC, "VK_OEM_5"),
    vk!(0xDD, "VK_OEM_6"),
    vk!(0xDE, "VK_OEM_7"),
    vk!(0xDF, "VK_OEM_8"),
    vk!(0xE0, "UNASSIGNED"),
    vk!(0xE1, "UNASSIGNED"),
    vk!(0xE2, "VK_OEM_102"),
    vk!(0xE3, "UNASSIGNED"),
    vk!(0xE4, "UNASSIGNED"),
    vk!(0xE5, "VK_PROCESSKEY"),
    vk!(0xE6, "UNASSIGNED"),
    vk!(0xE7, "VK_PACKET"),
    vk!(0xE8, "UNASSIGNED"),
    vk!(0xE9, "UNASSIGNED"),
    vk!(0xEA, "UNASSIGNED"),
    vk!(0xEB, "UNASSIGNED"),
    vk!(0xEC, "UNASSIGNED"),
    vk!(0xED, "UNASSIGNED"),
    vk!(0xEE, "UNASSIGNED"),
    vk!(0xEF, "UNASSIGNED"),
    vk!(0xF0, "UNASSIGNED"),
    vk!(0xF1, "UNASSIGNED"),
    vk!(0xF2, "UNASSIGNED"),
    vk!(0xF3, "UNASSIGNED"),
    vk!(0xF4, "UNASSIGNED"),
    vk!(0xF5, "UNASSIGNED"),
    vk!(0xF6, "VK_ATTN"),
    vk!(0xF7, "VK_CRSEL"),
    vk!(0xF8, "VK_EXSEL"),
    vk!(0xF9, "VK_EREOF"),
    vk!(0xFA, "VK_PLAY"),
    vk!(0xFB, "VK_ZOOM"),
    vk!(0xFC, "VK_NONAME"),
    vk!(0xFD, "VK_PA1"),
    vk!(0xFE, "VK_OEM_CLEAR"),
    vk!(0xFF, "VK__none_"),
];

macro_rules! sc {
    ($code:literal, $name:literal) => {
        ScanCode {
            value: $code,
            is_extended: false,
            name: $name,
        }
    };
}

macro_rules! ext_sc {
    ($code:literal, $name:literal) => {
        ScanCode {
            value: $code,
            is_extended: true,
            name: $name,
        }
    };
}

static SCANCODES: [[ScanCode; 2]; MAX_SCAN_CODE] = [
    [sc!(0x00, "UNASSIGNED"), ext_sc!(0x00, "UNASSIGNED")],
    [sc!(0x01, "SC_ESC"), ext_sc!(0x01, "SC_")],
    [sc!(0x02, "SC_1"), ext_sc!(0x02, "SC_1")],
    [sc!(0x03, "SC_2"), ext_sc!(0x03, "SC_2")],
    [sc!(0x04, "SC_3"), ext_sc!(0x04, "SC_3")],
    [sc!(0x05, "SC_4"), ext_sc!(0x05, "SC_4")],
    [sc!(0x06, "SC_5"), ext_sc!(0x06, "SC_5")],
    [sc!(0x07, "SC_6"), ext_sc!(0x07, "SC_6")],
    [sc!(0x08, "SC_7"), ext_sc!(0x08, "SC_7")],
    [sc!(0x09, "SC_8"), ext_sc!(0x09, "SC_8")],
    [sc!(0x0A, "SC_9"), ext_sc!(0x0A, "SC_9")],
    [sc!(0x0B, "SC_0"), ext_sc!(0x0B, "SC_0")],
    [sc!(0x0C, "SC_MINUS"), ext_sc!(0x0C, "SC_MINUS")],
    [sc!(0x0D, "SC_EQ"), ext_sc!(0x0D, "SC_EQ")],
    [sc!(0x0E, "SC_BACKSPACE"), ext_sc!(0x0E, "SC")],
    [sc!(0x0F, "SC_TAB"), ext_sc!(0x0F, "SC_	")],
    [sc!(0x10, "SC_Q"), ext_sc!(0x10, "SC_Q")],
    [sc!(0x11, "SC_W"), ext_sc!(0x11, "SC_W")],
    [sc!(0x12, "SC_E"), ext_sc!(0x12, "SC_E")],
    [sc!(0x13, "SC_R"), ext_sc!(0x13, "SC_R")],
    [sc!(0x14, "SC_T"), ext_sc!(0x14, "SC_T")],
    [sc!(0x15, "SC_Y"), ext_sc!(0x15, "SC_Y")],
    [sc!(0x16, "SC_U"), ext_sc!(0x16, "SC_U")],
    [sc!(0x17, "SC_I"), ext_sc!(0x17, "SC_I")],
    [sc!(0x18, "SC_O"), ext_sc!(0x18, "SC_O")],
    [sc!(0x19, "SC_P"), ext_sc!(0x19, "SC_P")],
    [sc!(0x1A, "SC_L_BRACKET"), ext_sc!(0x1A, "SC_L_BRACKET")],
    [sc!(0x1B, "SC_R_BRACKET"), ext_sc!(0x1B, "SC_R_BRACKET")],
    [sc!(0x1C, "SC_ENTER"), ext_sc!(0x1C, "SC_NUM_ENTER")],
    [sc!(0x1D, "SC_CTRL"), ext_sc!(0x1D, "SC_RIGHT_CTRL")],
    [sc!(0x1E, "SC_A"), ext_sc!(0x1E, "SC_A")],
    [sc!(0x1F, "SC_S"), ext_sc!(0x1F, "SC_S")],
    [sc!(0x20, "SC_D"), ext_sc!(0x20, "SC_VOL_MUTE")],
    [sc!(0x21, "SC_F"), ext_sc!(0x21, "SC_CALCULATOR")],
    [sc!(0x22, "SC_G"), ext_sc!(0x22, "SC_G")],
    [sc!(0x23, "SC_H"), ext_sc!(0x23, "SC_H")],
    [sc!(0x24, "SC_J"), ext_sc!(0x24, "SC_J")],
    [sc!(0x25, "SC_K"), ext_sc!(0x25, "SC_K")],
    [sc!(0x26, "SC_L"), ext_sc!(0x26, "SC_L")],
    [sc!(0x27, "SC_SEMICOLON"), ext_sc!(0x27, "SC_SEMICOLON")],
    [sc!(0x28, "SC_APOSTROPHE"), ext_sc!(0x28, "SC_APOSTROPHE")],
    [sc!(0x29, "SC_BACKTICK"), ext_sc!(0x29, "SC_BACKTICK")],
    [sc!(0x2A, "SC_SHIFT"), ext_sc!(0x2A, "UNASSIGNED")],
    [sc!(0x2B, "SC_BACKSLASH"), ext_sc!(0x2B, "SC_BRIGHTNESS")],
    [sc!(0x2C, "SC_Z"), ext_sc!(0x2C, "SC_Z")],
    [sc!(0x2D, "SC_X"), ext_sc!(0x2D, "SC_X")],
    [sc!(0x2E, "SC_C"), ext_sc!(0x2E, "SC_VOLUME_DOWN")],
    [sc!(0x2F, "SC_V"), ext_sc!(0x2F, "SC_V")],
    [sc!(0x30, "SC_B"), ext_sc!(0x30, "SC_VOLUME_UP")],
    [sc!(0x31, "SC_N"), ext_sc!(0x31, "SC_N")],
    [sc!(0x32, "SC_M"), ext_sc!(0x32, "SC_M")],
    [sc!(0x33, "SC_COMMA"), ext_sc!(0x33, "SC_COMMA")],
    [sc!(0x34, "SC_DOT"), ext_sc!(0x34, "SC_DOT")],
    [sc!(0x35, "SC_SLASH"), ext_sc!(0x35, "SC_NUM_SLASH")],
    [sc!(0x36, "SC_RIGHT_SHIFT"), ext_sc!(0x36, "SC_RIGHT_SHIFT")],
    [sc!(0x37, "SC_NUM_MUL"), ext_sc!(0x37, "SC_PRNT_SCRN")],
    [sc!(0x38, "SC_ALT"), ext_sc!(0x38, "SC_RIGHT_ALT")],
    [sc!(0x39, "SC_SPACE"), ext_sc!(0x39, "SC__")],
    [sc!(0x3A, "SC_CAPS_LOCK"), ext_sc!(0x3A, "UNASSIGNED")],
    [sc!(0x3B, "SC_F1"), ext_sc!(0x3B, "UNASSIGNED")],
    [sc!(0x3C, "SC_F2"), ext_sc!(0x3C, "UNASSIGNED")],
    [sc!(0x3D, "SC_F3"), ext_sc!(0x3D, "UNASSIGNED")],
    [sc!(0x3E, "SC_F4"), ext_sc!(0x3E, "UNASSIGNED")],
    [sc!(0x3F, "SC_F5"), ext_sc!(0x3F, "UNASSIGNED")],
    [sc!(0x40, "SC_F6"), ext_sc!(0x40, "UNASSIGNED")],
    [sc!(0x41, "SC_F7"), ext_sc!(0x41, "UNASSIGNED")],
    [sc!(0x42, "SC_F8"), ext_sc!(0x42, "UNASSIGNED")],
    [sc!(0x43, "SC_F9"), ext_sc!(0x43, "UNASSIGNED")],
    [sc!(0x44, "SC_F10"), ext_sc!(0x44, "UNASSIGNED")],
    [sc!(0x45, "SC_PAUSE"), ext_sc!(0x45, "SC_NUM_LOCK")],
    [sc!(0x46, "SC_SCROLL_LOCK"), ext_sc!(0x46, "SC_BREAK")],
    [sc!(0x47, "SC_NUM_7"), ext_sc!(0x47, "SC_HOME")],
    [sc!(0x48, "SC_NUM_8"), ext_sc!(0x48, "SC_UP")],
    [sc!(0x49, "SC_NUM_9"), ext_sc!(0x49, "SC_PAGE_UP")],
    [sc!(0x4A, "SC_NUM_MINUS"), ext_sc!(0x4A, "SC_MINUS")],
    [sc!(0x4B, "SC_NUM_4"), ext_sc!(0x4B, "SC_LEFT")],
    [sc!(0x4C, "SC_NUM_5"), ext_sc!(0x4C, "UNASSIGNED")],
    [sc!(0x4D, "SC_NUM_6"), ext_sc!(0x4D, "SC_RIGHT")],
    [sc!(0x4E, "SC_NUM_PLUS"), ext_sc!(0x4E, "SC_PLUS")],
    [sc!(0x4F, "SC_NUM_1"), ext_sc!(0x4F, "SC_END")],
    [sc!(0x50, "SC_NUM_2"), ext_sc!(0x50, "SC_DOWN")],
    [sc!(0x51, "SC_NUM_3"), ext_sc!(0x51, "SC_PAGE_DOWN")],
    [sc!(0x52, "SC_NUM_0"), ext_sc!(0x52, "SC_INSERT")],
    [sc!(0x53, "SC_NUM_DEL"), ext_sc!(0x53, "SC_DELETE")],
    [sc!(0x54, "SC_SYS_REQ"), ext_sc!(0x54, "SC_<00>")],
    [sc!(0x55, "UNASSIGNED"), ext_sc!(0x55, "UNASSIGNED")],
    [sc!(0x56, "SC_BACKSLASH"), ext_sc!(0x56, "SC_HELP")],
    [sc!(0x57, "SC_F11"), ext_sc!(0x57, "UNASSIGNED")],
    [sc!(0x58, "SC_F12"), ext_sc!(0x58, "UNASSIGNED")],
    [sc!(0x59, "UNASSIGNED"), ext_sc!(0x59, "UNASSIGNED")],
    [sc!(0x5A, "UNASSIGNED"), ext_sc!(0x5A, "UNASSIGNED")],
    [sc!(0x5B, "UNASSIGNED"), ext_sc!(0x5B, "SC_LEFT_WINDOWS")],
    [sc!(0x5C, "UNASSIGNED"), ext_sc!(0x5C, "SC_RIGHT_WINDOWS")],
    [sc!(0x5D, "UNASSIGNED"), ext_sc!(0x5D, "SC_APPLICATION")],
    [sc!(0x5E, "UNASSIGNED"), ext_sc!(0x5E, "UNASSIGNED")],
    [sc!(0x5F, "UNASSIGNED"), ext_sc!(0x5F, "UNASSIGNED")],
    [sc!(0x60, "UNASSIGNED"), ext_sc!(0x60, "UNASSIGNED")],
    [sc!(0x61, "UNASSIGNED"), ext_sc!(0x61, "UNASSIGNED")],
    [sc!(0x62, "UNASSIGNED"), ext_sc!(0x62, "UNASSIGNED")],
    [sc!(0x63, "UNASSIGNED"), ext_sc!(0x63, "UNASSIGNED")],
    [sc!(0x64, "UNASSIGNED"), ext_sc!(0x64, "UNASSIGNED")],
    [sc!(0x65, "UNASSIGNED"), ext_sc!(0x65, "UNASSIGNED")],
    [sc!(0x66, "UNASSIGNED"), ext_sc!(0x66, "UNASSIGNED")],
    [sc!(0x67, "UNASSIGNED"), ext_sc!(0x67, "UNASSIGNED")],
    [sc!(0x68, "UNASSIGNED"), ext_sc!(0x68, "UNASSIGNED")],
    [sc!(0x69, "UNASSIGNED"), ext_sc!(0x69, "UNASSIGNED")],
    [sc!(0x6A, "UNASSIGNED"), ext_sc!(0x6A, "UNASSIGNED")],
    [sc!(0x6B, "UNASSIGNED"), ext_sc!(0x6B, "UNASSIGNED")],
    [sc!(0x6C, "UNASSIGNED"), ext_sc!(0x6C, "UNASSIGNED")],
    [sc!(0x6D, "UNASSIGNED"), ext_sc!(0x6D, "UNASSIGNED")],
    [sc!(0x6E, "UNASSIGNED"), ext_sc!(0x6E, "UNASSIGNED")],
    [sc!(0x6F, "UNASSIGNED"), ext_sc!(0x6F, "UNASSIGNED")],
    [sc!(0x70, "UNASSIGNED"), ext_sc!(0x70, "UNASSIGNED")],
    [sc!(0x71, "UNASSIGNED"), ext_sc!(0x71, "UNASSIGNED")],
    [sc!(0x72, "UNASSIGNED"), ext_sc!(0x72, "UNASSIGNED")],
    [sc!(0x73, "UNASSIGNED"), ext_sc!(0x73, "UNASSIGNED")],
    [sc!(0x74, "UNASSIGNED"), ext_sc!(0x74, "UNASSIGNED")],
    [sc!(0x75, "UNASSIGNED"), ext_sc!(0x75, "UNASSIGNED")],
    [sc!(0x76, "UNASSIGNED"), ext_sc!(0x76, "UNASSIGNED")],
    [sc!(0x77, "UNASSIGNED"), ext_sc!(0x77, "UNASSIGNED")],
    [sc!(0x78, "UNASSIGNED"), ext_sc!(0x78, "UNASSIGNED")],
    [sc!(0x79, "UNASSIGNED"), ext_sc!(0x79, "UNASSIGNED")],
    [sc!(0x7A, "UNASSIGNED"), ext_sc!(0x7A, "UNASSIGNED")],
    [sc!(0x7B, "UNASSIGNED"), ext_sc!(0x7B, "UNASSIGNED")],
    [sc!(0x7C, "SC_F13"), ext_sc!(0x7C, "SC_	")],
    [sc!(0x7D, "SC_F14"), ext_sc!(0x7D, "UNASSIGNED")],
    [sc!(0x7E, "SC_F15"), ext_sc!(0x7E, "UNASSIGNED")],
    [sc!(0x7F, "SC_F16"), ext_sc!(0x7F, "UNASSIGNED")],
    [sc!(0x80, "SC_F17"), ext_sc!(0x80, "UNASSIGNED")],
    [sc!(0x81, "SC_F18"), ext_sc!(0x81, "UNASSIGNED")],
    [sc!(0x82, "SC_F19"), ext_sc!(0x82, "UNASSIGNED")],
    [sc!(0x83, "SC_F20"), ext_sc!(0x83, "UNASSIGNED")],
    [sc!(0x84, "SC_F21"), ext_sc!(0x84, "UNASSIGNED")],
    [sc!(0x85, "SC_F22"), ext_sc!(0x85, "UNASSIGNED")],
    [sc!(0x86, "SC_F23"), ext_sc!(0x86, "UNASSIGNED")],
    [sc!(0x87, "SC_F24"), ext_sc!(0x87, "UNASSIGNED")],
];
