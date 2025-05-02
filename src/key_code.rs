use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, LLKHF_EXTENDED};

pub(crate) const MAX_KEY_ID: usize = 0x100;

pub(crate) trait KeyCode {
    fn name(&self) -> &'static str;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct VirtualKey {
    name: &'static str,
    pub(crate) value: u8,
}

impl VirtualKey {
    pub(crate) fn by_code(code: u8) -> Result<&'static VirtualKey, String> {
        let position = VIRTUAL_KEYS.iter().position(|probe| probe.value == code);

        if let Some(ix) = position {
            Ok(&VIRTUAL_KEYS[ix])
        } else {
            Err(format!("Unsupported virtual key code: 0x{:02X}.", code))
        }
    }

    pub(crate) fn by_name(name: &str) -> Result<&'static VirtualKey, String> {
        let position = VIRTUAL_KEYS.iter().position(|probe| probe.name == name);

        if let Some(ix) = position {
            Ok(&VIRTUAL_KEYS[ix])
        } else {
            Err(format!("Unsupported virtual key name: `{}`", name))
        }
    }
}

impl KeyCode for VirtualKey {
    fn name(&self) -> &'static str {
        self.name
    }
}

impl Display for VirtualKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:02X}", &self.value, )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ScanCode {
    name: &'static str,
    pub(crate) value: u8,
    pub(crate) is_extended: bool,
}

impl ScanCode {
    pub(crate) fn by_code(code: u8, extended: bool) -> Result<&'static ScanCode, String> {
        let position = SCANCODES
            .iter()
            .position(|probe| probe.is_extended == extended && probe.value == code);

        if let Some(ix) = position {
            Ok(&SCANCODES[ix])
        } else {
            Err(format!(
                "Unsupported scan code: {} extended: {}",
                code, extended
            ))
        }
    }

    pub(crate) fn by_name(name: &str) -> Result<&'static ScanCode, String> {
        let position = SCANCODES.iter().position(|probe| probe.name == name);

        if let Some(ix) = position {
            Ok(&SCANCODES[ix])
        } else {
            Err(format!("Unsupported scan code name: `{}`.", name))
        }
    }

    pub(crate) fn by_ext_code(ext_code: u16) -> Result<&'static ScanCode, String> {
        Self::by_code(ext_code as u8, ext_code & 0xE000 != 0)
    }

    pub(crate) fn by_symbol(symbol: char) -> Result<&'static ScanCode, String> {
        let ext_code = unsafe { OemKeyScan(symbol as u16) } as u16;
        ScanCode::by_ext_code(ext_code)
    }

    pub(crate) fn ext_value(&self) -> u16 {
        if self.is_extended {
            self.value as u16 | 0xE0 << 8
        } else {
            self.value as u16
        }
    }
}

impl KeyCode for ScanCode {
    fn name(&self) -> &'static str {
        self.name
    }
}

impl Display for ScanCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04X}", &self.ext_value(), )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Key {
    pub(crate) virtual_key: Option<&'static VirtualKey>,
    pub(crate) scancode: Option<&'static ScanCode>,
}

impl Key {
    pub(crate) fn from_scancode(scancode: &'static ScanCode) -> Self {
        Self {
            scancode: Some(scancode),
            virtual_key: None,
        }
    }

    pub(crate) fn from_virtual_key(virtual_key: &'static VirtualKey) -> Self {
        Self {
            scancode: None,
            virtual_key: Some(virtual_key),
        }
    }

    pub(crate) fn from_kb(kb: &KBDLLHOOKSTRUCT) -> Self {
        let scancode = ScanCode::by_code(kb.scanCode as u8, kb.flags.contains(LLKHF_EXTENDED))
            .unwrap_or(&INVALID_SCANCODE);

        let virtual_key = VirtualKey::by_code(kb.vkCode as u8).unwrap_or(&INVALID_VIRTUAL_KEY);

        Self {
            scancode: Some(scancode),
            virtual_key: Some(virtual_key),
        }
    }

    fn parse(text: Option<&str>) -> Result<Self, String> {
        if let Some(s) = text {
            if s.starts_with("SC_0x") {
                Self::parse_scancode_from_code(s)
            } else if s.starts_with("VK_0x") {
                Self::parse_virtual_key_from_code(s)
            } else if s.starts_with("SC_") {
                Self::parse_scancode_from_name(s)
            } else if s.starts_with("VK_") {
                Self::parse_virtual_key_from_name(s)
            } else {
                Self::parse_scancode_from_symbol(s).or_else(|_| {
                    let vk = format!("VK_{}", s);
                    Self::parse(Some(vk.as_str())).or_else(|_| {
                        let sc = format!("SC_{}", s);
                        Self::parse(Some(sc.as_str()))
                    })
                })
            }
        } else {
            Err("Missing key identifier.".to_string())
        }
    }

    #[inline]
    fn parse_virtual_key_from_name(s: &str) -> Result<Self, String> {
        Ok(Self::from_virtual_key(VirtualKey::by_name(s)?))
    }

    #[inline]
    fn parse_virtual_key_from_code(s: &str) -> Result<Self, String> {
        let code = u8::from_str_radix(s.strip_prefix("VK_0x").ok_or("No `VK_` prefix")?, 16)
            .map_err(|_| format!("Failed to parse virtual key code: {}.", s))?;

        Ok(Self::from_virtual_key(VirtualKey::by_code(code)?))
    }

    #[inline]
    fn parse_scancode_from_name(s: &str) -> Result<Self, String> {
        Ok(Self::from_scancode(ScanCode::by_name(s)?))
    }

    #[inline]
    fn parse_scancode_from_code(s: &str) -> Result<Self, String> {
        let code = u16::from_str_radix(s.strip_prefix("SC_0x").ok_or("No `SC_` prefix")?, 16)
            .map_err(|_| format!("Failed to parse scancode: {}.", s))?;

        Ok(Self::from_scancode(ScanCode::by_ext_code(code)?))
    }

    #[inline]
    fn parse_scancode_from_symbol(s: &str) -> Result<Self, String> {
        if s.len() == 1 {
            let symbol = s.chars().next().unwrap();
            Ok(Self::from_scancode(ScanCode::by_symbol(symbol)?))
        } else {
            Err(format!("Symbol is to long: {}", s.len()))
        }
    }
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let text = if let Some(virtual_key) = self.virtual_key {
            virtual_key.name
        } else if let Some(scancode) = self.scancode {
            scancode.name
        } else {
            panic!("Action key cannot be empty.");
        };

        Ok(text.serialize(serializer)?)
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        let result = Self::parse(Some(text.as_str()))
            .map_err(|e| Error::custom(format!("Unable to parse key identifier.\n{}", e)))?;
        Ok(result)
    }
}

static INVALID_VIRTUAL_KEY: VirtualKey = VirtualKey {
    name: "N/A",
    value: 0x00,
};
static VIRTUAL_KEYS: [VirtualKey; 174] = [
    VirtualKey {
        name: "VK_LBUTTON",
        value: 0x01,
    },
    VirtualKey {
        name: "VK_RBUTTON",
        value: 0x02,
    },
    VirtualKey {
        name: "VK_CANCEL",
        value: 0x03,
    },
    VirtualKey {
        name: "VK_MBUTTON",
        value: 0x04,
    },
    VirtualKey {
        name: "VK_XBUTTON1",
        value: 0x05,
    },
    VirtualKey {
        name: "VK_XBUTTON2",
        value: 0x06,
    },
    VirtualKey {
        name: "VK_BACK",
        value: 0x08,
    },
    VirtualKey {
        name: "VK_TAB",
        value: 0x09,
    },
    VirtualKey {
        name: "VK_CLEAR",
        value: 0x0C,
    },
    VirtualKey {
        name: "VK_RETURN",
        value: 0x0D,
    },
    VirtualKey {
        name: "VK_SHIFT",
        value: 0x10,
    },
    VirtualKey {
        name: "VK_CONTROL",
        value: 0x11,
    },
    VirtualKey {
        name: "VK_MENU",
        value: 0x12,
    },
    VirtualKey {
        name: "VK_PAUSE",
        value: 0x13,
    },
    VirtualKey {
        name: "VK_CAPITAL",
        value: 0x14,
    },
    VirtualKey {
        name: "VK_KANA",
        value: 0x15,
    },
    VirtualKey {
        name: "VK_HANGUL",
        value: 0x15,
    },
    VirtualKey {
        name: "VK_IME_ON",
        value: 0x16,
    },
    VirtualKey {
        name: "VK_JUNJA",
        value: 0x17,
    },
    VirtualKey {
        name: "VK_FINAL",
        value: 0x18,
    },
    VirtualKey {
        name: "VK_HANJA",
        value: 0x19,
    },
    VirtualKey {
        name: "VK_KANJI",
        value: 0x19,
    },
    VirtualKey {
        name: "VK_IME_OFF",
        value: 0x1A,
    },
    VirtualKey {
        name: "VK_ESCAPE",
        value: 0x1B,
    },
    VirtualKey {
        name: "VK_CONVERT",
        value: 0x1C,
    },
    VirtualKey {
        name: "VK_NONCONVERT",
        value: 0x1D,
    },
    VirtualKey {
        name: "VK_ACCEPT",
        value: 0x1E,
    },
    VirtualKey {
        name: "VK_MODECHANGE",
        value: 0x1F,
    },
    VirtualKey {
        name: "VK_SPACE",
        value: 0x20,
    },
    VirtualKey {
        name: "VK_PRIOR",
        value: 0x21,
    },
    VirtualKey {
        name: "VK_NEXT",
        value: 0x22,
    },
    VirtualKey {
        name: "VK_END",
        value: 0x23,
    },
    VirtualKey {
        name: "VK_HOME",
        value: 0x24,
    },
    VirtualKey {
        name: "VK_LEFT",
        value: 0x25,
    },
    VirtualKey {
        name: "VK_UP",
        value: 0x26,
    },
    VirtualKey {
        name: "VK_RIGHT",
        value: 0x27,
    },
    VirtualKey {
        name: "VK_DOWN",
        value: 0x28,
    },
    VirtualKey {
        name: "VK_SELECT",
        value: 0x29,
    },
    VirtualKey {
        name: "VK_PRINT",
        value: 0x2A,
    },
    VirtualKey {
        name: "VK_EXECUTE",
        value: 0x2B,
    },
    VirtualKey {
        name: "VK_SNAPSHOT",
        value: 0x2C,
    },
    VirtualKey {
        name: "VK_INSERT",
        value: 0x2D,
    },
    VirtualKey {
        name: "VK_DELETE",
        value: 0x2E,
    },
    VirtualKey {
        name: "VK_HELP",
        value: 0x2F,
    },
    VirtualKey {
        name: "VK_0",
        value: 0x30,
    },
    VirtualKey {
        name: "VK_1",
        value: 0x31,
    },
    VirtualKey {
        name: "VK_2",
        value: 0x32,
    },
    VirtualKey {
        name: "VK_3",
        value: 0x33,
    },
    VirtualKey {
        name: "VK_4",
        value: 0x34,
    },
    VirtualKey {
        name: "VK_5",
        value: 0x35,
    },
    VirtualKey {
        name: "VK_6",
        value: 0x36,
    },
    VirtualKey {
        name: "VK_7",
        value: 0x37,
    },
    VirtualKey {
        name: "VK_8",
        value: 0x38,
    },
    VirtualKey {
        name: "VK_9",
        value: 0x39,
    },
    VirtualKey {
        name: "VK_A",
        value: 0x41,
    },
    VirtualKey {
        name: "VK_B",
        value: 0x42,
    },
    VirtualKey {
        name: "VK_C",
        value: 0x43,
    },
    VirtualKey {
        name: "VK_D",
        value: 0x44,
    },
    VirtualKey {
        name: "VK_E",
        value: 0x45,
    },
    VirtualKey {
        name: "VK_F",
        value: 0x46,
    },
    VirtualKey {
        name: "VK_G",
        value: 0x47,
    },
    VirtualKey {
        name: "VK_H",
        value: 0x48,
    },
    VirtualKey {
        name: "VK_I",
        value: 0x49,
    },
    VirtualKey {
        name: "VK_J",
        value: 0x4A,
    },
    VirtualKey {
        name: "VK_K",
        value: 0x4B,
    },
    VirtualKey {
        name: "VK_L",
        value: 0x4C,
    },
    VirtualKey {
        name: "VK_M",
        value: 0x4D,
    },
    VirtualKey {
        name: "VK_N",
        value: 0x4E,
    },
    VirtualKey {
        name: "VK_O",
        value: 0x4F,
    },
    VirtualKey {
        name: "VK_P",
        value: 0x50,
    },
    VirtualKey {
        name: "VK_Q",
        value: 0x51,
    },
    VirtualKey {
        name: "VK_R",
        value: 0x52,
    },
    VirtualKey {
        name: "VK_S",
        value: 0x53,
    },
    VirtualKey {
        name: "VK_T",
        value: 0x54,
    },
    VirtualKey {
        name: "VK_U",
        value: 0x55,
    },
    VirtualKey {
        name: "VK_V",
        value: 0x56,
    },
    VirtualKey {
        name: "VK_W",
        value: 0x57,
    },
    VirtualKey {
        name: "VK_X",
        value: 0x58,
    },
    VirtualKey {
        name: "VK_Y",
        value: 0x59,
    },
    VirtualKey {
        name: "VK_Z",
        value: 0x5A,
    },
    VirtualKey {
        name: "VK_LWIN",
        value: 0x5B,
    },
    VirtualKey {
        name: "VK_RWIN",
        value: 0x5C,
    },
    VirtualKey {
        name: "VK_APPS",
        value: 0x5D,
    },
    VirtualKey {
        name: "VK_SLEEP",
        value: 0x5F,
    },
    VirtualKey {
        name: "VK_NUMPAD0",
        value: 0x60,
    },
    VirtualKey {
        name: "VK_NUMPAD1",
        value: 0x61,
    },
    VirtualKey {
        name: "VK_NUMPAD2",
        value: 0x62,
    },
    VirtualKey {
        name: "VK_NUMPAD3",
        value: 0x63,
    },
    VirtualKey {
        name: "VK_NUMPAD4",
        value: 0x64,
    },
    VirtualKey {
        name: "VK_NUMPAD5",
        value: 0x65,
    },
    VirtualKey {
        name: "VK_NUMPAD6",
        value: 0x66,
    },
    VirtualKey {
        name: "VK_NUMPAD7",
        value: 0x67,
    },
    VirtualKey {
        name: "VK_NUMPAD8",
        value: 0x68,
    },
    VirtualKey {
        name: "VK_NUMPAD9",
        value: 0x69,
    },
    VirtualKey {
        name: "VK_MULTIPLY",
        value: 0x6A,
    },
    VirtualKey {
        name: "VK_ADD",
        value: 0x6B,
    },
    VirtualKey {
        name: "VK_SEPARATOR",
        value: 0x6C,
    },
    VirtualKey {
        name: "VK_SUBTRACT",
        value: 0x6D,
    },
    VirtualKey {
        name: "VK_DECIMAL",
        value: 0x6E,
    },
    VirtualKey {
        name: "VK_DIVIDE",
        value: 0x6F,
    },
    VirtualKey {
        name: "VK_F1",
        value: 0x70,
    },
    VirtualKey {
        name: "VK_F2",
        value: 0x71,
    },
    VirtualKey {
        name: "VK_F3",
        value: 0x72,
    },
    VirtualKey {
        name: "VK_F4",
        value: 0x73,
    },
    VirtualKey {
        name: "VK_F5",
        value: 0x74,
    },
    VirtualKey {
        name: "VK_F6",
        value: 0x75,
    },
    VirtualKey {
        name: "VK_F7",
        value: 0x76,
    },
    VirtualKey {
        name: "VK_F8",
        value: 0x77,
    },
    VirtualKey {
        name: "VK_F9",
        value: 0x78,
    },
    VirtualKey {
        name: "VK_F10",
        value: 0x79,
    },
    VirtualKey {
        name: "VK_F11",
        value: 0x7A,
    },
    VirtualKey {
        name: "VK_F12",
        value: 0x7B,
    },
    VirtualKey {
        name: "VK_F13",
        value: 0x7C,
    },
    VirtualKey {
        name: "VK_F14",
        value: 0x7D,
    },
    VirtualKey {
        name: "VK_F15",
        value: 0x7E,
    },
    VirtualKey {
        name: "VK_F16",
        value: 0x7F,
    },
    VirtualKey {
        name: "VK_F17",
        value: 0x80,
    },
    VirtualKey {
        name: "VK_F18",
        value: 0x81,
    },
    VirtualKey {
        name: "VK_F19",
        value: 0x82,
    },
    VirtualKey {
        name: "VK_F20",
        value: 0x83,
    },
    VirtualKey {
        name: "VK_F21",
        value: 0x84,
    },
    VirtualKey {
        name: "VK_F22",
        value: 0x85,
    },
    VirtualKey {
        name: "VK_F23",
        value: 0x86,
    },
    VirtualKey {
        name: "VK_F24",
        value: 0x87,
    },
    VirtualKey {
        name: "VK_NUMLOCK",
        value: 0x90,
    },
    VirtualKey {
        name: "VK_SCROLL",
        value: 0x91,
    },
    VirtualKey {
        name: "VK_LSHIFT",
        value: 0xA0,
    },
    VirtualKey {
        name: "VK_RSHIFT",
        value: 0xA1,
    },
    VirtualKey {
        name: "VK_LCONTROL",
        value: 0xA2,
    },
    VirtualKey {
        name: "VK_RCONTROL",
        value: 0xA3,
    },
    VirtualKey {
        name: "VK_LMENU",
        value: 0xA4,
    },
    VirtualKey {
        name: "VK_RMENU",
        value: 0xA5,
    },
    VirtualKey {
        name: "VK_BROWSER_BACK",
        value: 0xA6,
    },
    VirtualKey {
        name: "VK_BROWSER_FORWARD",
        value: 0xA7,
    },
    VirtualKey {
        name: "VK_BROWSER_REFRESH",
        value: 0xA8,
    },
    VirtualKey {
        name: "VK_BROWSER_STOP",
        value: 0xA9,
    },
    VirtualKey {
        name: "VK_BROWSER_SEARCH",
        value: 0xAA,
    },
    VirtualKey {
        name: "VK_BROWSER_FAVORITES",
        value: 0xAB,
    },
    VirtualKey {
        name: "VK_BROWSER_HOME",
        value: 0xAC,
    },
    VirtualKey {
        name: "VK_VOLUME_MUTE",
        value: 0xAD,
    },
    VirtualKey {
        name: "VK_VOLUME_DOWN",
        value: 0xAE,
    },
    VirtualKey {
        name: "VK_VOLUME_UP",
        value: 0xAF,
    },
    VirtualKey {
        name: "VK_MEDIA_NEXT_TRACK",
        value: 0xB0,
    },
    VirtualKey {
        name: "VK_MEDIA_PREV_TRACK",
        value: 0xB1,
    },
    VirtualKey {
        name: "VK_MEDIA_STOP",
        value: 0xB2,
    },
    VirtualKey {
        name: "VK_MEDIA_PLAY_PAUSE",
        value: 0xB3,
    },
    VirtualKey {
        name: "VK_LAUNCH_MAIL",
        value: 0xB4,
    },
    VirtualKey {
        name: "VK_LAUNCH_MEDIA_SELECT",
        value: 0xB5,
    },
    VirtualKey {
        name: "VK_LAUNCH_APP1",
        value: 0xB6,
    },
    VirtualKey {
        name: "VK_LAUNCH_APP2",
        value: 0xB7,
    },
    VirtualKey {
        name: "VK_OEM_1",
        value: 0xBA,
    },
    VirtualKey {
        name: "VK_OEM_PLUS",
        value: 0xBB,
    },
    VirtualKey {
        name: "VK_OEM_COMMA",
        value: 0xBC,
    },
    VirtualKey {
        name: "VK_OEM_MINUS",
        value: 0xBD,
    },
    VirtualKey {
        name: "VK_OEM_PERIOD",
        value: 0xBE,
    },
    VirtualKey {
        name: "VK_OEM_2",
        value: 0xBF,
    },
    VirtualKey {
        name: "VK_OEM_3",
        value: 0xC0,
    },
    VirtualKey {
        name: "VK_OEM_4",
        value: 0xDB,
    },
    VirtualKey {
        name: "VK_OEM_5",
        value: 0xDC,
    },
    VirtualKey {
        name: "VK_OEM_6",
        value: 0xDD,
    },
    VirtualKey {
        name: "VK_OEM_7",
        value: 0xDE,
    },
    VirtualKey {
        name: "VK_OEM_8",
        value: 0xDF,
    },
    VirtualKey {
        name: "VK_OEM_102",
        value: 0xE2,
    },
    VirtualKey {
        name: "VK_PROCESSKEY",
        value: 0xE5,
    },
    VirtualKey {
        name: "VK_PACKET",
        value: 0xE7,
    },
    VirtualKey {
        name: "VK_ATTN",
        value: 0xF6,
    },
    VirtualKey {
        name: "VK_CRSEL",
        value: 0xF7,
    },
    VirtualKey {
        name: "VK_EXSEL",
        value: 0xF8,
    },
    VirtualKey {
        name: "VK_EREOF",
        value: 0xF9,
    },
    VirtualKey {
        name: "VK_PLAY",
        value: 0xFA,
    },
    VirtualKey {
        name: "VK_ZOOM",
        value: 0xFB,
    },
    VirtualKey {
        name: "VK_NONAME",
        value: 0xFC,
    },
    VirtualKey {
        name: "VK_PA1",
        value: 0xFD,
    },
    VirtualKey {
        name: "VK_OEM_CLEAR",
        value: 0xFE,
    },
];

static INVALID_SCANCODE: ScanCode = ScanCode {
    name: "N/A",
    value: 0x00,
    is_extended: false,
};
static SCANCODES: [ScanCode; 126] = [
    ScanCode {
        name: "SC_ESC",
        value: 0x01,
        is_extended: false,
    },
    ScanCode {
        name: "SC_1",
        value: 0x02,
        is_extended: false,
    },
    ScanCode {
        name: "SC_2",
        value: 0x03,
        is_extended: false,
    },
    ScanCode {
        name: "SC_3",
        value: 0x04,
        is_extended: false,
    },
    ScanCode {
        name: "SC_4",
        value: 0x05,
        is_extended: false,
    },
    ScanCode {
        name: "SC_5",
        value: 0x06,
        is_extended: false,
    },
    ScanCode {
        name: "SC_6",
        value: 0x07,
        is_extended: false,
    },
    ScanCode {
        name: "SC_7",
        value: 0x08,
        is_extended: false,
    },
    ScanCode {
        name: "SC_8",
        value: 0x09,
        is_extended: false,
    },
    ScanCode {
        name: "SC_9",
        value: 0x0A,
        is_extended: false,
    },
    ScanCode {
        name: "SC_0",
        value: 0x0B,
        is_extended: false,
    },
    ScanCode {
        name: "SC_MINUS",
        value: 0x0C,
        is_extended: false,
    },
    ScanCode {
        name: "SC_EQUALITY",
        value: 0x0D,
        is_extended: false,
    },
    ScanCode {
        name: "SC_BACKSPACE",
        value: 0x0E,
        is_extended: false,
    },
    ScanCode {
        name: "SC_TAB",
        value: 0x0F,
        is_extended: false,
    },
    ScanCode {
        name: "SC_Q",
        value: 0x10,
        is_extended: false,
    },
    ScanCode {
        name: "SC_W",
        value: 0x11,
        is_extended: false,
    },
    ScanCode {
        name: "SC_E",
        value: 0x12,
        is_extended: false,
    },
    ScanCode {
        name: "SC_R",
        value: 0x13,
        is_extended: false,
    },
    ScanCode {
        name: "SC_T",
        value: 0x14,
        is_extended: false,
    },
    ScanCode {
        name: "SC_Y",
        value: 0x15,
        is_extended: false,
    },
    ScanCode {
        name: "SC_U",
        value: 0x16,
        is_extended: false,
    },
    ScanCode {
        name: "SC_I",
        value: 0x17,
        is_extended: false,
    },
    ScanCode {
        name: "SC_O",
        value: 0x18,
        is_extended: false,
    },
    ScanCode {
        name: "SC_P",
        value: 0x19,
        is_extended: false,
    },
    ScanCode {
        name: "SC_L_BRACKET",
        value: 0x1A,
        is_extended: false,
    },
    ScanCode {
        name: "SC_R_BRACKET",
        value: 0x1B,
        is_extended: false,
    },
    ScanCode {
        name: "SC_ENTER",
        value: 0x1C,
        is_extended: false,
    },
    ScanCode {
        name: "SC_CTRL",
        value: 0x1D,
        is_extended: false,
    },
    ScanCode {
        name: "SC_A",
        value: 0x1E,
        is_extended: false,
    },
    ScanCode {
        name: "SC_S",
        value: 0x1F,
        is_extended: false,
    },
    ScanCode {
        name: "SC_D",
        value: 0x20,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F",
        value: 0x21,
        is_extended: false,
    },
    ScanCode {
        name: "SC_G",
        value: 0x22,
        is_extended: false,
    },
    ScanCode {
        name: "SC_H",
        value: 0x23,
        is_extended: false,
    },
    ScanCode {
        name: "SC_J",
        value: 0x24,
        is_extended: false,
    },
    ScanCode {
        name: "SC_K",
        value: 0x25,
        is_extended: false,
    },
    ScanCode {
        name: "SC_L",
        value: 0x26,
        is_extended: false,
    },
    ScanCode {
        name: "SC_SEMICOLON",
        value: 0x27,
        is_extended: false,
    },
    ScanCode {
        name: "SC_APOSTROPHE",
        value: 0x28,
        is_extended: false,
    },
    ScanCode {
        name: "SC_BACKTICK",
        value: 0x29,
        is_extended: false,
    },
    ScanCode {
        name: "SC_SHIFT",
        value: 0x2A,
        is_extended: false,
    },
    ScanCode {
        name: "SC_BACKSLASH",
        value: 0x2B,
        is_extended: false,
    },
    ScanCode {
        name: "SC_Z",
        value: 0x2C,
        is_extended: false,
    },
    ScanCode {
        name: "SC_X",
        value: 0x2D,
        is_extended: false,
    },
    ScanCode {
        name: "SC_C",
        value: 0x2E,
        is_extended: false,
    },
    ScanCode {
        name: "SC_V",
        value: 0x2F,
        is_extended: false,
    },
    ScanCode {
        name: "SC_B",
        value: 0x30,
        is_extended: false,
    },
    ScanCode {
        name: "SC_N",
        value: 0x31,
        is_extended: false,
    },
    ScanCode {
        name: "SC_M",
        value: 0x32,
        is_extended: false,
    },
    ScanCode {
        name: "SC_COMMA",
        value: 0x33,
        is_extended: false,
    },
    ScanCode {
        name: "SC_DOT",
        value: 0x34,
        is_extended: false,
    },
    ScanCode {
        name: "SC_SLASH",
        value: 0x35,
        is_extended: false,
    },
    ScanCode {
        name: "SC_RIGHT_SHIFT",
        value: 0x36,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_MUL",
        value: 0x37,
        is_extended: false,
    },
    ScanCode {
        name: "SC_ALT",
        value: 0x38,
        is_extended: false,
    },
    ScanCode {
        name: "SC_SPACE",
        value: 0x39,
        is_extended: false,
    },
    ScanCode {
        name: "SC_CAPS_LOCK",
        value: 0x3A,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F1",
        value: 0x3B,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F2",
        value: 0x3C,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F3",
        value: 0x3D,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F4",
        value: 0x3E,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F5",
        value: 0x3F,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F6",
        value: 0x40,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F7",
        value: 0x41,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F8",
        value: 0x42,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F9",
        value: 0x43,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F10",
        value: 0x44,
        is_extended: false,
    },
    ScanCode {
        name: "SC_PAUSE",
        value: 0x45,
        is_extended: false,
    },
    ScanCode {
        name: "SC_SCROLL_LOCK",
        value: 0x46,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_7",
        value: 0x47,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_8",
        value: 0x48,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_9",
        value: 0x49,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_MINUS",
        value: 0x4A,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_4",
        value: 0x4B,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_5",
        value: 0x4C,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_6",
        value: 0x4D,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_PLUS",
        value: 0x4E,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_1",
        value: 0x4F,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_2",
        value: 0x50,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_3",
        value: 0x51,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_0",
        value: 0x52,
        is_extended: false,
    },
    ScanCode {
        name: "SC_NUM_DEL",
        value: 0x53,
        is_extended: false,
    },
    ScanCode {
        name: "SC_SYS_REQ",
        value: 0x54,
        is_extended: false,
    },
    ScanCode {
        name: "SC_EUROPE_2",
        value: 0x56,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F11",
        value: 0x57,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F12",
        value: 0x58,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F13",
        value: 0x7C,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F14",
        value: 0x7D,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F15",
        value: 0x7E,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F16",
        value: 0x7F,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F17",
        value: 0x80,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F18",
        value: 0x81,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F19",
        value: 0x82,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F20",
        value: 0x83,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F21",
        value: 0x84,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F22",
        value: 0x85,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F23",
        value: 0x86,
        is_extended: false,
    },
    ScanCode {
        name: "SC_F24",
        value: 0x87,
        is_extended: false,
    },
    /*
        Extended
    */
    ScanCode {
        name: "SC_NUM_ENTER",
        value: 0x1C,
        is_extended: true,
    },
    ScanCode {
        name: "SC_RIGHT_CTRL",
        value: 0x1D,
        is_extended: true,
    },
    ScanCode {
        name: "SC_VOL_MUTE",
        value: 0x20,
        is_extended: true,
    },
    ScanCode {
        name: "SC_CALCULATOR",
        value: 0x21,
        is_extended: true,
    },
    ScanCode {
        name: "SC_BRIGHTNESS",
        value: 0x2B,
        is_extended: true,
    },
    ScanCode {
        name: "SC_VOL_DOWN",
        value: 0x2E,
        is_extended: true,
    },
    ScanCode {
        name: "SC_VOL_UP",
        value: 0x30,
        is_extended: true,
    },
    ScanCode {
        name: "SC_NUM_DIV",
        value: 0x35,
        is_extended: true,
    },
    ScanCode {
        name: "SC_RIGHT_SHIFT",
        value: 0x36,
        is_extended: true,
    },
    ScanCode {
        name: "SC_PRNT_SCRN",
        value: 0x37,
        is_extended: true,
    },
    ScanCode {
        name: "SC_RIGHT_ALT",
        value: 0x38,
        is_extended: true,
    },
    ScanCode {
        name: "SC_NUM_LOCK",
        value: 0x45,
        is_extended: true,
    },
    ScanCode {
        name: "SC_BREAK",
        value: 0x46,
        is_extended: true,
    },
    ScanCode {
        name: "SC_HOME",
        value: 0x47,
        is_extended: true,
    },
    ScanCode {
        name: "SC_UP",
        value: 0x48,
        is_extended: true,
    },
    ScanCode {
        name: "SC_PAGE_UP",
        value: 0x49,
        is_extended: true,
    },
    ScanCode {
        name: "SC_LEFT",
        value: 0x4B,
        is_extended: true,
    },
    ScanCode {
        name: "SC_RIGHT",
        value: 0x4D,
        is_extended: true,
    },
    ScanCode {
        name: "SC_END",
        value: 0x4F,
        is_extended: true,
    },
    ScanCode {
        name: "SC_DOWN",
        value: 0x50,
        is_extended: true,
    },
    ScanCode {
        name: "SC_PAGE_DOWN",
        value: 0x51,
        is_extended: true,
    },
    ScanCode {
        name: "SC_INSERT",
        value: 0x52,
        is_extended: true,
    },
    ScanCode {
        name: "SC_DELETE",
        value: 0x53,
        is_extended: true,
    },
    ScanCode {
        name: "SC_HELP",
        value: 0x56,
        is_extended: true,
    },
    ScanCode {
        name: "SC_LEFT_WINDOWS",
        value: 0x5B,
        is_extended: true,
    },
    ScanCode {
        name: "SC_RIGHT_WINDOWS",
        value: 0x5C,
        is_extended: true,
    },
    ScanCode {
        name: "SC_APPLICATION",
        value: 0x5D,
        is_extended: true,
    },
];

#[cfg(test)]
mod tests {
    use crate::key_code::*;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyNameTextW;
    //
    //     // #[test]
    //     // fn test_convert_scancode_to_ext_scancode() {
    //     //     let val = convert_scancode_to_ext_scancode(0x1C, false);
    //     //     assert_eq!(val, 0x1C);
    //     //
    //     //     let val = convert_scancode_to_ext_scancode(0x1C, true);
    //     //     assert_eq!(val, 0xE01C);
    //     // }
    //
    //     // #[test]
    //     // fn test_convert_ext_scancode_to_scancode() {
    //     //     let val = convert_ext_scancode_to_scancode(0x1C);
    //     //     assert_eq!(val.0, 0x1C);
    //     //     assert_eq!(val.1, false);
    //     //
    //     //     let val = convert_ext_scancode_to_scancode(0xE01C);
    //     //     assert_eq!(val.0, 0x1C);
    //     //     assert_eq!(val.1, true);
    //     // }
    //
    //     #[test]
    //     fn scancode_by_symbol() {
    //         assert_eq!(ScanCode::by_name("SC_A"), ScanCode::by_symbol('A'));
    //         assert_eq!(ScanCode::by_name("SC_7"), ScanCode::by_symbol('&'));
    //         assert_eq!(ScanCode::by_name("SC_BACKSLASH"), ScanCode::by_symbol('\\'));
    //         assert_eq!(ScanCode::by_name("SC_R_BRACKET"), ScanCode::by_symbol(']'));
    //     }
    //
    //     #[test]
    //     fn parse_key_actions_by_name() {
    //         let actual = KeyActionSequence::parse(&[
    //             "VK_A UP".to_string(),
    //             "VK_B DOWN".to_string(),
    //             "SC_A UP".to_string(),
    //             "SC_B DOWN".to_string(),
    //         ])
    //             .unwrap();
    //
    //         let expected = KeyActionSequence::from(vec![
    //             KeyAction {
    //                 key_id: AsVirtualKey(VirtualKey::by_name("VK_A").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsVirtualKey(VirtualKey::by_name("VK_B").unwrap()),
    //                 transition: Down,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_A").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_B").unwrap()),
    //                 transition: Down,
    //             },
    //         ]);
    //
    //         assert_eq!(actual, expected)
    //     }
    //
    //     #[test]
    //     fn parse_key_actions_vk_by_code() {
    //         let actual = KeyActionSequence::parse(&[
    //             "VK_0x1C UP".to_string(),
    //             "VK_0x30 DOWN".to_string(),
    //             "SC_0xE01C UP".to_string(),
    //             "SC_0xE030 DOWN".to_string(),
    //         ])
    //             .unwrap();
    //
    //         let expected = KeyActionSequence::from(vec![
    //             KeyAction {
    //                 key_id: AsVirtualKey(VirtualKey::by_name("VK_CONVERT").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsVirtualKey(VirtualKey::by_name("VK_0").unwrap()),
    //                 transition: Down,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_NUM_ENTER").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_VOL_UP").unwrap()),
    //                 transition: Down,
    //             },
    //         ]);
    //
    //         assert_eq!(actual, expected)
    //     }
    //
    //     #[test]
    //     fn parse_key_actions_symbols() {
    //         let actual = KeyActionSequence::parse(&[
    //             "] UP".to_string(),
    //             "= UP".to_string(),
    //             "+ UP".to_string(),
    //             "\\ DOWN".to_string(),
    //         ])
    //             .unwrap();
    //
    //         let expected = KeyActionSequence::from(vec![
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_R_BRACKET").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_EQUALITY").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_EQUALITY").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_BACKSLASH").unwrap()),
    //                 transition: Down,
    //             },
    //         ]);
    //
    //         assert_eq!(actual, expected)
    //     }
    //
    //     #[test]
    //     fn parse_key_actions_no_key_id_type() {
    //         let actual = KeyActionSequence::parse(&[
    //             "A UP".to_string(),
    //             "ENTER UP".to_string(),
    //             "RETURN UP".to_string(),
    //             "0x0D UP".to_string(),
    //             "0xE01C UP".to_string(),
    //         ])
    //             .unwrap();
    //
    //         let expected = KeyActionSequence::from(vec![
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_A").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_ENTER").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsVirtualKey(VirtualKey::by_name("VK_RETURN").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsVirtualKey(VirtualKey::by_name("VK_RETURN").unwrap()),
    //                 transition: Up,
    //             },
    //             KeyAction {
    //                 key_id: AsScanCode(ScanCode::by_name("SC_NUM_ENTER").unwrap()),
    //                 transition: Up,
    //             },
    //         ]);
    //
    //         assert_eq!(actual, expected)
    //     }
    //
    //     #[test]
    //     #[should_panic]
    //     fn parse_key_actions_no_transition() {
    //         let text = ["VK_A".to_string()];
    //         let sequence = KeyActionSequence::parse(&text).unwrap();
    //         dbg!(sequence);
    //         todo!();
    //     }
    //

    fn find_key_name(scancode: u8, extended: bool) -> Option<String> {
        let mut l_param: i32 = (scancode as i32) << 16;
        if extended {
            l_param |= 1 << 24;
        }

        let mut buffer = [0u16; 64];
        let result = unsafe { GetKeyNameTextW(l_param, &mut buffer) };
        if result > 0 {
            let text = OsString::from_wide(&buffer[..result as usize])
                .to_string_lossy()
                .into_owned();
            Some(text)
        } else {
            None
        }
    }

    fn fmt_scancode_name(key_name: &str) -> String {
        "SC_".to_string()
            + key_name
            .to_uppercase()
            .replace(' ', "_")
            .replace('`', "BACKTICK")
            .replace('\'', "APOSTROPHE")
            .replace('/', "SLASH")
            .replace('\\', "BACKSLASH")
            .replace('+', "PLUS")
            .replace('-', "MINUS")
            .replace('*', "MUL")
            .replace('=', "EQ")
            .replace('[', "L_BRACKET")
            .replace(']', "R_BRACKET")
            .replace(';', "SEMICOLON")
            .replace(',', "COMMA")
            .replace('.', "DOT")
            .as_str()
    }

    #[test]
    #[ignore]
    fn generate_scancode_table() {
        for extended in [false, true] {
            for value in 0..0xFF {
                if let Some(name) = find_key_name(value, extended) {
                    println!(
                        "ScanCode{{ name:\"{}\", value:0x{:02X}, extended:{} }},",
                        fmt_scancode_name(&name),
                        value,
                        extended
                    );
                }
            }
        }
    }

    #[test]
    fn test_scancode_table() {
        for extended in [false, true] {
            for value in 0..0xFF {
                if ScanCode::by_code(value, extended).is_err() {
                    println!("Unsupported scancode: 0x{:X} extended: {}", value, extended);
                }
            }
        }
    }
}