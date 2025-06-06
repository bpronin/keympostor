use crate::keyboard::key::{Key, ScanCode, VirtualKey};
use crate::keyboard::KeyError;
use fxhash::FxHashMap;

thread_local! {
    pub(crate) static KEY_MAP: KeyMap = KeyMap::new();
}

pub(crate) struct KeyMap {
    key_to_name_map: FxHashMap<Key, &'static str>,
    name_to_key_map: FxHashMap<&'static str, Key>,
}

impl KeyMap {
    fn new() -> Self {
        let mut name_to_key_map = FxHashMap::default();
        let mut key_to_name_map = FxHashMap::default();
        for (name, key) in KEYS {
            if name_to_key_map.insert(name, key).is_some() {
                panic!("Duplicate name: {}", name)
            };
            if key_to_name_map.insert(key, name).is_some() {
                panic!("Duplicate key: {}", key.code_name())
            };
        }
        Self {
            name_to_key_map,
            key_to_name_map,
        }
    }

    pub(crate) fn name_of(&self, key: &Key) -> &'static str {
        self.key_to_name_map
            .get(key)
            .expect(&format!("Unsupported key: {}", key.code_name()))
    }

    pub(crate) fn by_name(&self, name: &str) -> Result<Key, KeyError> {
        self.name_to_key_map
            .get(name)
            .ok_or(KeyError::new(&format!("Illegal key name: `{}`.", name)))
            .copied()
    }
}

macro_rules! new_key {
    ($name:expr, $vk_code:literal, $scan_code:literal, $is_ext_scan_code:literal) => {
        (
            $name,
            Key {
                vk_code: $vk_code,
                scan_code: $scan_code,
                is_ext_scan_code: $is_ext_scan_code,
            },
        )
    };
}

pub const MAX_KEYS: usize = 204;

pub static KEYS: [(&'static str, Key); MAX_KEYS] = [
    new_key!("	", 0x00, 0x0F, true),
    new_key!("", 0x00, 0x01, true),
    new_key!("0", 0x30, 0x0B, false),
    new_key!("1", 0x31, 0x02, false),
    new_key!("2", 0x32, 0x03, false),
    new_key!("3", 0x33, 0x04, false),
    new_key!("4", 0x34, 0x05, false),
    new_key!("5", 0x35, 0x06, false),
    new_key!("6", 0x36, 0x07, false),
    new_key!("7", 0x37, 0x08, false),
    new_key!("8", 0x38, 0x09, false),
    new_key!("9", 0x39, 0x0A, false),
    new_key!("<00>", 0x00, 0x54, true),
    new_key!("A", 0x41, 0x1E, false),
    new_key!("ACCEPT", 0x1E, 0x00, false),
    new_key!("APOSTROPHE", 0xDE, 0x28, false),
    new_key!("APPLICATION", 0x5D, 0x5D, true),
    new_key!("ATTN", 0xF6, 0x00, false),
    new_key!("B", 0x42, 0x30, false),
    new_key!("BACKSLASH", 0xDC, 0x2B, false),
    new_key!("BACKSLASH_2", 0xE2, 0x56, false),
    new_key!("BACKSPACE", 0x08, 0x0E, false),
    new_key!("BACKTICK", 0xC0, 0x29, false),
    new_key!("BREAK", 0x03, 0x46, true),
    new_key!("BRIGHTNESS", 0x00, 0x2B, true),
    new_key!("BROWSER_BACK", 0xA6, 0x6A, true),
    new_key!("BROWSER_FAVORITES", 0xAB, 0x66, true),
    new_key!("BROWSER_FORWARD", 0xA7, 0x69, true),
    new_key!("BROWSER_HOME", 0xAC, 0x00, true),
    new_key!("BROWSER_REFRESH", 0xA8, 0x67, true),
    new_key!("BROWSER_SEARCH", 0xAA, 0x00, true),
    new_key!("BROWSER_STOP", 0xA9, 0x68, true),
    new_key!("C", 0x43, 0x2E, false),
    new_key!("CAPS_LOCK", 0x14, 0x3A, false),
    new_key!("COMMA", 0xBC, 0x33, false),
    new_key!("CONVERT", 0x1C, 0x00, false),
    new_key!("CRSEL", 0xF7, 0x00, false),
    new_key!("CTRL", 0x11, 0x1D, false),
    new_key!("D", 0x44, 0x20, false),
    new_key!("DELETE", 0x2E, 0x53, true),
    new_key!("DOT", 0xBE, 0x34, false),
    new_key!("DOWN", 0x28, 0x50, true),
    new_key!("E", 0x45, 0x12, false),
    new_key!("END", 0x23, 0x4F, true),
    new_key!("ENTER", 0x0D, 0x1C, false),
    new_key!("EQ", 0xBB, 0x0D, false),
    new_key!("EREOF", 0xF9, 0x5D, false),
    new_key!("ESC", 0x1B, 0x01, false),
    new_key!("EXECUTE", 0x2B, 0x00, false),
    new_key!("EXSEL", 0xF8, 0x00, false),
    new_key!("F", 0x46, 0x21, false),
    new_key!("F1", 0x70, 0x3B, false),
    new_key!("F10", 0x79, 0x44, false),
    new_key!("F11", 0x7A, 0x57, false),
    new_key!("F12", 0x7B, 0x58, false),
    new_key!("F13", 0x7C, 0x64, false),
    new_key!("F14", 0x7D, 0x65, false),
    new_key!("F15", 0x7E, 0x66, false),
    new_key!("F16", 0x7F, 0x67, false),
    new_key!("F17", 0x80, 0x68, false),
    new_key!("F18", 0x81, 0x69, false),
    new_key!("F19", 0x82, 0x6A, false),
    new_key!("F2", 0x71, 0x3C, false),
    new_key!("F20", 0x83, 0x6B, false),
    new_key!("F21", 0x84, 0x6C, false),
    new_key!("F22", 0x85, 0x6D, false),
    new_key!("F23", 0x86, 0x6E, false),
    new_key!("F24", 0x87, 0x76, false),
    new_key!("F3", 0x72, 0x3D, false),
    new_key!("F4", 0x73, 0x3E, false),
    new_key!("F5", 0x74, 0x3F, false),
    new_key!("F6", 0x75, 0x40, false),
    new_key!("F7", 0x76, 0x41, false),
    new_key!("F8", 0x77, 0x42, false),
    new_key!("F9", 0x78, 0x43, false),
    new_key!("FINAL", 0x18, 0x00, false),
    new_key!("FN_BROWSER_HOME", 0xAC, 0x32, true),
    new_key!("FN_BROWSER_SEARCH", 0xAA, 0x65, true),
    new_key!("FN_LAUNCH_APP1", 0xB6, 0x6B, true),
    new_key!("FN_LAUNCH_APP2", 0xB7, 0x21, true),
    new_key!("FN_LAUNCH_MAIL", 0xB4, 0x6C, true),
    new_key!("FN_MEDIA_NEXT_TRACK", 0xB0, 0x19, true),
    new_key!("FN_MEDIA_PLAY_PAUSE", 0xB3, 0x22, true),
    new_key!("FN_MEDIA_PREV_TRACK", 0xB1, 0x10, true),
    new_key!("FN_VOLUME_DOWN", 0xAE, 0x2E, true),
    new_key!("FN_VOLUME_MUTE", 0xAD, 0x20, true),
    new_key!("FN_VOLUME_UP", 0xAF, 0x30, true),
    new_key!("G", 0x47, 0x22, false),
    new_key!("H", 0x48, 0x23, false),
    new_key!("HANJA", 0x19, 0x00, false),
    new_key!("HELP", 0x2F, 0x63, false),
    new_key!("HOME", 0x24, 0x47, true),
    new_key!("I", 0x49, 0x17, false),
    new_key!("IME_OFF", 0x1A, 0x00, false),
    new_key!("IME_ON", 0x16, 0x00, false),
    new_key!("INSERT", 0x2D, 0x52, true),
    new_key!("J", 0x4A, 0x24, false),
    new_key!("JUNJA", 0x17, 0x00, false),
    new_key!("K", 0x4B, 0x25, false),
    new_key!("KANA", 0x15, 0x00, false),
    new_key!("L", 0x4C, 0x26, false),
    new_key!("LAUNCH_APP1", 0xB6, 0x00, true),
    new_key!("LAUNCH_APP2", 0xB7, 0x00, true),
    new_key!("LAUNCH_MAIL", 0xB4, 0x00, true),
    new_key!("LAUNCH_MEDIA_SELECT", 0xB5, 0x6D, true),
    new_key!("LEFT", 0x25, 0x4B, true),
    new_key!("LEFT_ALT", 0xA4, 0x38, false),
    new_key!("LEFT_BRACKET", 0xDB, 0x1A, false),
    new_key!("LEFT_BUTTON", 0x01, 0x00, false),
    new_key!("LEFT_CTRL", 0xA2, 0x1D, false),
    new_key!("LEFT_SHIFT", 0xA0, 0x2A, false),
    new_key!("LEFT_WIN", 0x5B, 0x5B, true),
    new_key!("M", 0x4D, 0x32, false),
    new_key!("MEDIA_NEXT_TRACK", 0xB0, 0x00, true),
    new_key!("MEDIA_PLAY_PAUSE", 0xB3, 0x00, true),
    new_key!("MEDIA_PREV_TRACK", 0xB1, 0x00, true),
    new_key!("MEDIA_STOP", 0xB2, 0x24, true),
    new_key!("MENU", 0x12, 0x38, false),
    new_key!("MIDDLE_BUTTON", 0x04, 0x00, false),
    new_key!("MINUS", 0xBD, 0x0C, false),
    new_key!("MODE_CHANGE", 0x1F, 0x00, false),
    new_key!("N", 0x4E, 0x31, false),
    new_key!("NONAME", 0xFC, 0x00, false),
    new_key!("NON_CONVERT", 0x1D, 0x00, false),
    new_key!("NUM_0", 0x60, 0x52, false),
    new_key!("NUM_1", 0x61, 0x4F, false),
    new_key!("NUM_2", 0x62, 0x50, false),
    new_key!("NUM_3", 0x63, 0x51, false),
    new_key!("NUM_4", 0x64, 0x4B, false),
    new_key!("NUM_5", 0x65, 0x4C, false),
    new_key!("NUM_6", 0x66, 0x4D, false),
    new_key!("NUM_7", 0x67, 0x47, false),
    new_key!("NUM_8", 0x68, 0x48, false),
    new_key!("NUM_9", 0x69, 0x49, false),
    new_key!("NUM_CLEAR", 0x0C, 0x4C, false),
    new_key!("NUM_DELETE", 0x2E, 0x53, false),
    new_key!("NUM_DIV", 0x6F, 0x35, true),
    new_key!("NUM_DOT", 0x6E, 0x53, false),
    new_key!("NUM_DOWN", 0x28, 0x50, false),
    new_key!("NUM_END", 0x23, 0x4F, false),
    new_key!("NUM_ENTER", 0x0D, 0x1C, true),
    new_key!("NUM_HOME", 0x24, 0x47, false),
    new_key!("NUM_INSERT", 0x2D, 0x52, false),
    new_key!("NUM_LEFT", 0x25, 0x4B, false),
    new_key!("NUM_LOCK", 0x90, 0x45, true),
    new_key!("NUM_LOCK_2", 0x13, 0x45, true), /* CTRL + NUM_LOCK*/
    new_key!("NUM_MINUS", 0x6D, 0x4A, false),
    new_key!("NUM_MUL", 0x6A, 0x37, false),
    new_key!("NUM_PAGE_DOWN", 0x22, 0x51, false),
    new_key!("NUM_PAGE_UP", 0x21, 0x49, false),
    new_key!("NUM_PLUS", 0x6B, 0x4E, false),
    new_key!("NUM_RIGHT", 0x27, 0x4D, false),
    new_key!("NUM_UP", 0x26, 0x48, false),
    new_key!("O", 0x4F, 0x18, false),
    new_key!("OEM_8", 0xDF, 0x00, false),
    new_key!("OEM_CLEAR", 0xFE, 0x00, false),
    new_key!("P", 0x50, 0x19, false),
    new_key!("PA1", 0xFD, 0x00, false),
    new_key!("PACKET", 0xE7, 0x00, false),
    new_key!("PAGE_DOWN", 0x22, 0x51, true),
    new_key!("PAGE_UP", 0x21, 0x49, true),
    new_key!("PAUSE", 0x13, 0x45, false),
    new_key!("PLAY", 0xFA, 0x00, false),
    new_key!("PLUS", 0x00, 0x4E, true),
    new_key!("PRINT", 0x2A, 0x00, false),
    new_key!("PRINT_SCREEN", 0x2C, 0x37, true),
    new_key!("PROCESS_KEY", 0xE5, 0x00, false),
    new_key!("Q", 0x51, 0x10, false),
    new_key!("R", 0x52, 0x13, false),
    new_key!("RIGHT", 0x27, 0x4D, true),
    new_key!("RIGHT_ALT", 0xA5, 0x38, true),
    new_key!("RIGHT_BRACKET", 0xDD, 0x1B, false),
    new_key!("RIGHT_BUTTON", 0x02, 0x00, false),
    new_key!("RIGHT_CTRL", 0xA3, 0x1D, true),
    new_key!("RIGHT_SHIFT", 0xA1, 0x36, true),
    new_key!("RIGHT_SHIFT_2", 0x00, 0x36, true),
    new_key!("RIGHT_WIN", 0x5C, 0x5C, true),
    new_key!("S", 0x53, 0x1F, false),
    new_key!("SCROLL_LOCK", 0x91, 0x46, false),
    new_key!("SELECT", 0x29, 0x00, false),
    new_key!("SEMICOLON", 0xBA, 0x27, false),
    new_key!("SEPARATOR", 0x6C, 0x00, false),
    new_key!("SHIFT", 0x10, 0x2A, false),
    new_key!("SLASH", 0xBF, 0x35, false),
    new_key!("SLEEP", 0x5F, 0x5F, true),
    new_key!("SPACE", 0x20, 0x39, false),
    new_key!("SYS_REQ", 0x2C, 0x54, false),
    new_key!("T", 0x54, 0x14, false),
    new_key!("TAB", 0x09, 0x0F, false),
    new_key!("U", 0x55, 0x16, false),
    new_key!("UNASSIGNED", 0, 0, false),
    new_key!("UP", 0x26, 0x48, true),
    new_key!("V", 0x56, 0x2F, false),
    new_key!("VOLUME_DOWN", 0xAE, 0x00, true),
    new_key!("VOLUME_MUTE", 0xAD, 0x00, true),
    new_key!("VOLUME_UP", 0xAF, 0x00, true),
    new_key!("W", 0x57, 0x11, false),
    new_key!("X", 0x58, 0x2D, false),
    new_key!("XBUTTON1", 0x05, 0x00, false),
    new_key!("XBUTTON2", 0x06, 0x00, false),
    new_key!("Y", 0x59, 0x15, false),
    new_key!("Z", 0x5A, 0x2C, false),
    new_key!("ZOOM", 0xFB, 0x62, false),
    new_key!("_", 0x00, 0x39, true),
];

macro_rules! new_vk {
    ($code:literal, $name:literal) => {
        VirtualKey {
            value: $code,
            name: $name,
        }
    };
}

pub const MAX_VK_CODE: usize = 256;

pub(crate) static VIRTUAL_KEYS: [VirtualKey; MAX_VK_CODE] = [
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

pub const MAX_SCAN_CODE: usize = 136;

pub(crate) static SCAN_CODES: [[ScanCode; 2]; MAX_SCAN_CODE] = [
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
    use crate::keyboard::key_const::{KEYS, KEY_MAP};

    #[test]
    fn test_key_by_name() {
        assert!(
            KEYS.iter()
                .all(|(name, key)| KEY_MAP.with(|k| k.by_name(name).unwrap()) == *key)
        )
    }

    #[test]
    fn test_key_name() {
        assert!(
            KEYS.iter()
                .all(|(name, key)| KEY_MAP.with(|k| k.name_of(key)) == *name)
        )
    }

    #[test]
    fn test_key_vk() {
        KEYS.iter().for_each(|(_name, key)| {
            key.virtual_key(); /* should not panic */
        })
    }

    #[test]
    fn test_key_sc() {
        KEYS.iter().for_each(|(_name, key)| {
            key.scan_code(); /* should not panic */
        })
    }
}
