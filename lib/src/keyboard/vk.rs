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
    // pub(crate) fn hex_code(&self) -> String {
    //     format!("VC_0x{:02X}", self.value)
    // }
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

macro_rules! new_vk {
    ($code:literal, $name:literal) => {
        VirtualKey {
            value: $code,
            name: $name,
        }
    };
}

pub const MAX_VK_CODE: usize = 256;

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
    new_vk!(0xF0, "UNASSIGNED"), /* used as a custom MOUSE_X key */
    new_vk!(0xF1, "UNASSIGNED"), /* used as a custom MOUSE_Y key */
    new_vk!(0xF2, "UNASSIGNED"), /* used as a custom WHEEL key */
    new_vk!(0xF3, "UNASSIGNED"), /* used as a custom WHEEL_TILT key */
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

#[cfg(test)]
mod tests {
    use crate::keyboard::key::NAME_TO_KEY_MAP;
    use crate::keyboard::vk::VirtualKey;
    use crate::vk_key;
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
    fn test_map() {
        NAME_TO_KEY_MAP.entries().for_each(|(_name, key)| {
            let _ = VirtualKey::from(key); /* should not panic */
        })
    }
}
