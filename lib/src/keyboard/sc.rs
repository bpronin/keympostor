use crate::keyboard::key::Key;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

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

    // pub(crate) fn hex_code(&self) -> String {
    //     format!("SC_0x{:04X}", self.ext_value())
    // }
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

macro_rules! new_sc {
    ($code:literal, $name:literal, $ext_name:literal) => {
        [
            ScanCode {
                value: $code,
                is_extended: false,
                name: $name,
            },
            ScanCode {
                value: $code,
                is_extended: true,
                name: $ext_name,
            },
        ]
    };
}

const MAX_SCAN_CODE: usize = 136;

static SCAN_CODES: [[ScanCode; 2]; MAX_SCAN_CODE] = [
    new_sc!(0x00, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x01, "SC_ESC", "SC_"),
    new_sc!(0x02, "SC_1", "SC_1"),
    new_sc!(0x03, "SC_2", "SC_2"),
    new_sc!(0x04, "SC_3", "SC_3"),
    new_sc!(0x05, "SC_4", "SC_4"),
    new_sc!(0x06, "SC_5", "SC_5"),
    new_sc!(0x07, "SC_6", "SC_6"),
    new_sc!(0x08, "SC_7", "SC_7"),
    new_sc!(0x09, "SC_8", "SC_8"),
    new_sc!(0x0A, "SC_9", "SC_9"),
    new_sc!(0x0B, "SC_0", "SC_0"),
    new_sc!(0x0C, "SC_MINUS", "SC_MINUS"),
    new_sc!(0x0D, "SC_EQ", "SC_EQ"),
    new_sc!(0x0E, "SC_BACKSPACE", "SC"),
    new_sc!(0x0F, "SC_TAB", "SC_	"),
    new_sc!(0x10, "SC_Q", "SC_Q"),
    new_sc!(0x11, "SC_W", "SC_W"),
    new_sc!(0x12, "SC_E", "SC_E"),
    new_sc!(0x13, "SC_R", "SC_R"),
    new_sc!(0x14, "SC_T", "SC_T"),
    new_sc!(0x15, "SC_Y", "SC_Y"),
    new_sc!(0x16, "SC_U", "SC_U"),
    new_sc!(0x17, "SC_I", "SC_I"),
    new_sc!(0x18, "SC_O", "SC_O"),
    new_sc!(0x19, "SC_P", "SC_P"),
    new_sc!(0x1A, "SC_L_BRACKET", "SC_L_BRACKET"),
    new_sc!(0x1B, "SC_R_BRACKET", "SC_R_BRACKET"),
    new_sc!(0x1C, "SC_ENTER", "SC_NUM_ENTER"),
    new_sc!(0x1D, "SC_CTRL", "SC_RIGHT_CTRL"),
    new_sc!(0x1E, "SC_A", "SC_A"),
    new_sc!(0x1F, "SC_S", "SC_S"),
    new_sc!(0x20, "SC_D", "SC_VOL_MUTE"),
    new_sc!(0x21, "SC_F", "SC_CALCULATOR"),
    new_sc!(0x22, "SC_G", "SC_G"),
    new_sc!(0x23, "SC_H", "SC_H"),
    new_sc!(0x24, "SC_J", "SC_J"),
    new_sc!(0x25, "SC_K", "SC_K"),
    new_sc!(0x26, "SC_L", "SC_L"),
    new_sc!(0x27, "SC_SEMICOLON", "SC_SEMICOLON"),
    new_sc!(0x28, "SC_APOSTROPHE", "SC_APOSTROPHE"),
    new_sc!(0x29, "SC_BACKTICK", "SC_BACKTICK"),
    new_sc!(0x2A, "SC_SHIFT", "UNASSIGNED"),
    new_sc!(0x2B, "SC_BACKSLASH", "SC_BRIGHTNESS"),
    new_sc!(0x2C, "SC_Z", "SC_Z"),
    new_sc!(0x2D, "SC_X", "SC_X"),
    new_sc!(0x2E, "SC_C", "SC_VOLUME_DOWN"),
    new_sc!(0x2F, "SC_V", "SC_V"),
    new_sc!(0x30, "SC_B", "SC_VOLUME_UP"),
    new_sc!(0x31, "SC_N", "SC_N"),
    new_sc!(0x32, "SC_M", "SC_M"),
    new_sc!(0x33, "SC_COMMA", "SC_COMMA"),
    new_sc!(0x34, "SC_DOT", "SC_DOT"),
    new_sc!(0x35, "SC_SLASH", "SC_NUM_SLASH"),
    new_sc!(0x36, "SC_RIGHT_SHIFT", "SC_RIGHT_SHIFT"),
    new_sc!(0x37, "SC_NUM_MUL", "SC_PRNT_SCRN"),
    new_sc!(0x38, "SC_ALT", "SC_RIGHT_ALT"),
    new_sc!(0x39, "SC_SPACE", "SC__"),
    new_sc!(0x3A, "SC_CAPS_LOCK", "UNASSIGNED"),
    new_sc!(0x3B, "SC_F1", "UNASSIGNED"),
    new_sc!(0x3C, "SC_F2", "UNASSIGNED"),
    new_sc!(0x3D, "SC_F3", "UNASSIGNED"),
    new_sc!(0x3E, "SC_F4", "UNASSIGNED"),
    new_sc!(0x3F, "SC_F5", "UNASSIGNED"),
    new_sc!(0x40, "SC_F6", "UNASSIGNED"),
    new_sc!(0x41, "SC_F7", "UNASSIGNED"),
    new_sc!(0x42, "SC_F8", "UNASSIGNED"),
    new_sc!(0x43, "SC_F9", "UNASSIGNED"),
    new_sc!(0x44, "SC_F10", "UNASSIGNED"),
    new_sc!(0x45, "SC_PAUSE", "SC_NUM_LOCK"),
    new_sc!(0x46, "SC_SCROLL_LOCK", "SC_BREAK"),
    new_sc!(0x47, "SC_NUM_7", "SC_HOME"),
    new_sc!(0x48, "SC_NUM_8", "SC_UP"),
    new_sc!(0x49, "SC_NUM_9", "SC_PAGE_UP"),
    new_sc!(0x4A, "SC_NUM_MINUS", "SC_MINUS"),
    new_sc!(0x4B, "SC_NUM_4", "SC_LEFT"),
    new_sc!(0x4C, "SC_NUM_5", "UNASSIGNED"),
    new_sc!(0x4D, "SC_NUM_6", "SC_RIGHT"),
    new_sc!(0x4E, "SC_NUM_PLUS", "SC_PLUS"),
    new_sc!(0x4F, "SC_NUM_1", "SC_END"),
    new_sc!(0x50, "SC_NUM_2", "SC_DOWN"),
    new_sc!(0x51, "SC_NUM_3", "SC_PAGE_DOWN"),
    new_sc!(0x52, "SC_NUM_0", "SC_INSERT"),
    new_sc!(0x53, "SC_NUM_DEL", "SC_DELETE"),
    new_sc!(0x54, "SC_SYS_REQ", "SC_<00>"),
    new_sc!(0x55, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x56, "SC_BACKSLASH", "SC_HELP"),
    new_sc!(0x57, "SC_F11", "UNASSIGNED"),
    new_sc!(0x58, "SC_F12", "UNASSIGNED"),
    new_sc!(0x59, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x5A, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x5B, "UNASSIGNED", "SC_LEFT_WINDOWS"),
    new_sc!(0x5C, "UNASSIGNED", "SC_RIGHT_WINDOWS"),
    new_sc!(0x5D, "UNASSIGNED", "SC_APPLICATION"),
    new_sc!(0x5E, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x5F, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x60, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x61, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x62, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x63, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x64, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x65, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x66, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x67, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x68, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x69, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x6A, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x6B, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x6C, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x6D, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x6E, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x6F, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x70, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x71, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x72, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x73, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x74, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x75, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x76, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x77, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x78, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x79, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x7A, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x7B, "UNASSIGNED", "UNASSIGNED"),
    new_sc!(0x7C, "SC_F13", "SC_	"),
    new_sc!(0x7D, "SC_F14", "UNASSIGNED"),
    new_sc!(0x7E, "SC_F15", "UNASSIGNED"),
    new_sc!(0x7F, "SC_F16", "UNASSIGNED"),
    new_sc!(0x80, "SC_F17", "UNASSIGNED"),
    new_sc!(0x81, "SC_F18", "UNASSIGNED"),
    new_sc!(0x82, "SC_F19", "UNASSIGNED"),
    new_sc!(0x83, "SC_F20", "UNASSIGNED"),
    new_sc!(0x84, "SC_F21", "UNASSIGNED"),
    new_sc!(0x85, "SC_F22", "UNASSIGNED"),
    new_sc!(0x86, "SC_F23", "UNASSIGNED"),
    new_sc!(0x87, "SC_F24", "UNASSIGNED"),
];

#[cfg(test)]
mod tests {
    use crate::keyboard::key::KEYS;
    use crate::keyboard::sc::ScanCode;
    use crate::sc_key;
    use std::str::FromStr;

    #[test]
    fn test_map() {
        KEYS.iter().for_each(|(_name, key)| {
            let _ = ScanCode::from(key); /* should not panic */
        })
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
