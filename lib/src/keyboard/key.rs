use crate::keyboard::error::KeyError;
use crate::{deserialize_from_string, serialize_to_string};
use fxhash::FxHashMap;
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
        KEY_MAP.with(|map| map.find_by_code(&(vk_code, scan_code))).unwrap()
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
        KEY_MAP.with(|map| map.find_by_name(s.trim()))
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
        Key::from_str($text).unwrap()
    };
}

thread_local! {
    static KEY_MAP: KeyMap = KeyMap::new();
}

struct KeyMap {
    code_to_key_map: FxHashMap<(u8, (u8, bool)), Key>,
    name_to_key_map: FxHashMap<&'static str, Key>,
}

impl KeyMap {
    fn new() -> Self {
        let mut name_to_key_map = FxHashMap::default();
        let mut code_to_key_map = FxHashMap::default();
        for (name, key) in KEYS {
            if name_to_key_map.insert(name, key).is_some() {
                panic!("Duplicate name: {}", name)
            };
            if code_to_key_map
                .insert((key.vk_code, key.scan_code), key)
                .is_some()
            {
                panic!("Duplicate key: {}", key)
            };
        }
        Self {
            name_to_key_map,
            code_to_key_map,
        }
    }

    pub(crate) fn find_by_code(&self, code: &(u8, (u8, bool))) -> Result<Key, KeyError> {
        self.code_to_key_map
            .get(code)
            .ok_or(KeyError::new(&format!("Illegal key code: `{:?}`.", code)))
            .copied()
    }

    pub(crate) fn find_by_name(&self, name: &str) -> Result<Key, KeyError> {
        self.name_to_key_map
            .get(name)
            .ok_or(KeyError::new(&format!("Illegal key name: `{}`.", name)))
            .copied()
    }
}

macro_rules! new_key {
    ($name:literal, $vk_code:literal, $scan_code:literal, $is_ext_scan_code:literal) => {
        Key {
            name: $name,
            vk_code: $vk_code,
            scan_code: ($scan_code, $is_ext_scan_code),
        }
    };
}

pub(crate) static KEY_00: Key = new_key!("<00>", 0x00, 0x54, true);
pub(crate) static KEY_0: Key = new_key!("0", 0x30, 0x0B, false);
pub(crate) static KEY_1: Key = new_key!("1", 0x31, 0x02, false);
pub(crate) static KEY_2: Key = new_key!("2", 0x32, 0x03, false);
pub(crate) static KEY_3: Key = new_key!("3", 0x33, 0x04, false);
pub(crate) static KEY_4: Key = new_key!("4", 0x34, 0x05, false);
pub(crate) static KEY_5: Key = new_key!("5", 0x35, 0x06, false);
pub(crate) static KEY_6: Key = new_key!("6", 0x36, 0x07, false);
pub(crate) static KEY_7: Key = new_key!("7", 0x37, 0x08, false);
pub(crate) static KEY_8: Key = new_key!("8", 0x38, 0x09, false);
pub(crate) static KEY_9: Key = new_key!("9", 0x39, 0x0A, false);
pub(crate) static KEY_A: Key = new_key!("A", 0x41, 0x1E, false);
pub(crate) static KEY_ACCEPT: Key = new_key!("ACCEPT", 0x1E, 0x00, false);
pub(crate) static KEY_APOSTROPHE: Key = new_key!("APOSTROPHE", 0xDE, 0x28, false);
pub(crate) static KEY_APPLICATION: Key = new_key!("APPLICATION", 0x5D, 0x5D, true);
pub(crate) static KEY_ATTN: Key = new_key!("ATTN", 0xF6, 0x00, false);
pub(crate) static KEY_B: Key = new_key!("B", 0x42, 0x30, false);
pub(crate) static KEY_BACKSLASH: Key = new_key!("BACKSLASH", 0xDC, 0x2B, false);
pub(crate) static KEY_BACKSLASH_2: Key = new_key!("BACKSLASH_2", 0xE2, 0x56, false);
pub(crate) static KEY_BACKSPACE: Key = new_key!("BACKSPACE", 0x08, 0x0E, false);
pub(crate) static KEY_BACKTICK: Key = new_key!("BACKTICK", 0xC0, 0x29, false);
pub(crate) static KEY_BREAK: Key = new_key!("BREAK", 0x03, 0x46, true);
pub(crate) static KEY_BRIGHTNESS: Key = new_key!("BRIGHTNESS", 0x00, 0x2B, true);
pub(crate) static KEY_BROWSER_BACK: Key = new_key!("BROWSER_BACK", 0xA6, 0x6A, true);
pub(crate) static KEY_BROWSER_FAVORITES: Key = new_key!("BROWSER_FAVORITES", 0xAB, 0x66, true);
pub(crate) static KEY_BROWSER_FORWARD: Key = new_key!("BROWSER_FORWARD", 0xA7, 0x69, true);
pub(crate) static KEY_BROWSER_HOME: Key = new_key!("BROWSER_HOME", 0xAC, 0x00, true);
pub(crate) static KEY_BROWSER_REFRESH: Key = new_key!("BROWSER_REFRESH", 0xA8, 0x67, true);
pub(crate) static KEY_BROWSER_SEARCH: Key = new_key!("BROWSER_SEARCH", 0xAA, 0x00, true);
pub(crate) static KEY_BROWSER_STOP: Key = new_key!("BROWSER_STOP", 0xA9, 0x68, true);
pub(crate) static KEY_C: Key = new_key!("C", 0x43, 0x2E, false);
pub(crate) static KEY_CAPS_LOCK: Key = new_key!("CAPS_LOCK", 0x14, 0x3A, false);
pub(crate) static KEY_COMMA: Key = new_key!("COMMA", 0xBC, 0x33, false);
pub(crate) static KEY_CONVERT: Key = new_key!("CONVERT", 0x1C, 0x00, false);
pub(crate) static KEY_CRSEL: Key = new_key!("CRSEL", 0xF7, 0x00, false);
pub(crate) static KEY_CTRL: Key = new_key!("CTRL", 0x11, 0x1D, false);
pub(crate) static KEY_D: Key = new_key!("D", 0x44, 0x20, false);
pub(crate) static KEY_DELETE: Key = new_key!("DELETE", 0x2E, 0x53, true);
pub(crate) static KEY_DOT: Key = new_key!("DOT", 0xBE, 0x34, false);
pub(crate) static KEY_DOWN: Key = new_key!("DOWN", 0x28, 0x50, true);
pub(crate) static KEY_E: Key = new_key!("E", 0x45, 0x12, false);
pub(crate) static KEY_END: Key = new_key!("END", 0x23, 0x4F, true);
pub(crate) static KEY_ENTER: Key = new_key!("ENTER", 0x0D, 0x1C, false);
pub(crate) static KEY_EQ: Key = new_key!("EQ", 0xBB, 0x0D, false);
pub(crate) static KEY_EREOF: Key = new_key!("EREOF", 0xF9, 0x5D, false);
pub(crate) static KEY_ESC: Key = new_key!("ESC", 0x1B, 0x01, false);
pub(crate) static KEY_EXECUTE: Key = new_key!("EXECUTE", 0x2B, 0x00, false);
pub(crate) static KEY_EXSEL: Key = new_key!("EXSEL", 0xF8, 0x00, false);
pub(crate) static KEY_F10: Key = new_key!("F10", 0x79, 0x44, false);
pub(crate) static KEY_F11: Key = new_key!("F11", 0x7A, 0x57, false);
pub(crate) static KEY_F12: Key = new_key!("F12", 0x7B, 0x58, false);
pub(crate) static KEY_F13: Key = new_key!("F13", 0x7C, 0x64, false);
pub(crate) static KEY_F14: Key = new_key!("F14", 0x7D, 0x65, false);
pub(crate) static KEY_F15: Key = new_key!("F15", 0x7E, 0x66, false);
pub(crate) static KEY_F16: Key = new_key!("F16", 0x7F, 0x67, false);
pub(crate) static KEY_F17: Key = new_key!("F17", 0x80, 0x68, false);
pub(crate) static KEY_F18: Key = new_key!("F18", 0x81, 0x69, false);
pub(crate) static KEY_F19: Key = new_key!("F19", 0x82, 0x6A, false);
pub(crate) static KEY_F1: Key = new_key!("F1", 0x70, 0x3B, false);
pub(crate) static KEY_F20: Key = new_key!("F20", 0x83, 0x6B, false);
pub(crate) static KEY_F21: Key = new_key!("F21", 0x84, 0x6C, false);
pub(crate) static KEY_F22: Key = new_key!("F22", 0x85, 0x6D, false);
pub(crate) static KEY_F23: Key = new_key!("F23", 0x86, 0x6E, false);
pub(crate) static KEY_F24: Key = new_key!("F24", 0x87, 0x76, false);
pub(crate) static KEY_F2: Key = new_key!("F2", 0x71, 0x3C, false);
pub(crate) static KEY_F3: Key = new_key!("F3", 0x72, 0x3D, false);
pub(crate) static KEY_F4: Key = new_key!("F4", 0x73, 0x3E, false);
pub(crate) static KEY_F5: Key = new_key!("F5", 0x74, 0x3F, false);
pub(crate) static KEY_F6: Key = new_key!("F6", 0x75, 0x40, false);
pub(crate) static KEY_F7: Key = new_key!("F7", 0x76, 0x41, false);
pub(crate) static KEY_F8: Key = new_key!("F8", 0x77, 0x42, false);
pub(crate) static KEY_F9: Key = new_key!("F9", 0x78, 0x43, false);
pub(crate) static KEY_F: Key = new_key!("F", 0x46, 0x21, false);
pub(crate) static KEY_FINAL: Key = new_key!("FINAL", 0x18, 0x00, false);
pub(crate) static KEY_FN_BROWSER_HOME: Key = new_key!("FN_BROWSER_HOME", 0xAC, 0x32, true);
pub(crate) static KEY_FN_BROWSER_SEARCH: Key = new_key!("FN_BROWSER_SEARCH", 0xAA, 0x65, true);
pub(crate) static KEY_FN_LAUNCH_APP1: Key = new_key!("FN_LAUNCH_APP1", 0xB6, 0x6B, true);
pub(crate) static KEY_FN_LAUNCH_APP2: Key = new_key!("FN_LAUNCH_APP2", 0xB7, 0x21, true);
pub(crate) static KEY_FN_LAUNCH_MAIL: Key = new_key!("FN_LAUNCH_MAIL", 0xB4, 0x6C, true);
pub(crate) static KEY_FN_MEDIA_NEXT_TRACK: Key = new_key!("FN_MEDIA_NEXT_TRACK", 0xB0, 0x19, true);
pub(crate) static KEY_FN_MEDIA_PLAY_PAUSE: Key = new_key!("FN_MEDIA_PLAY_PAUSE", 0xB3, 0x22, true);
pub(crate) static KEY_FN_MEDIA_PREV_TRACK: Key = new_key!("FN_MEDIA_PREV_TRACK", 0xB1, 0x10, true);
pub(crate) static KEY_FN_VOLUME_DOWN: Key = new_key!("FN_VOLUME_DOWN", 0xAE, 0x2E, true);
pub(crate) static KEY_FN_VOLUME_MUTE: Key = new_key!("FN_VOLUME_MUTE", 0xAD, 0x20, true);
pub(crate) static KEY_FN_VOLUME_UP: Key = new_key!("FN_VOLUME_UP", 0xAF, 0x30, true);
pub(crate) static KEY_G: Key = new_key!("G", 0x47, 0x22, false);
pub(crate) static KEY_H: Key = new_key!("H", 0x48, 0x23, false);
pub(crate) static KEY_HANJA: Key = new_key!("HANJA", 0x19, 0x00, false);
pub(crate) static KEY_HELP: Key = new_key!("HELP", 0x2F, 0x63, false);
pub(crate) static KEY_HOME: Key = new_key!("HOME", 0x24, 0x47, true);
pub(crate) static KEY_I: Key = new_key!("I", 0x49, 0x17, false);
pub(crate) static KEY_IME_OFF: Key = new_key!("IME_OFF", 0x1A, 0x00, false);
pub(crate) static KEY_IME_ON: Key = new_key!("IME_ON", 0x16, 0x00, false);
pub(crate) static KEY_INSERT: Key = new_key!("INSERT", 0x2D, 0x52, true);
pub(crate) static KEY_J: Key = new_key!("J", 0x4A, 0x24, false);
pub(crate) static KEY_JUNJA: Key = new_key!("JUNJA", 0x17, 0x00, false);
pub(crate) static KEY_K: Key = new_key!("K", 0x4B, 0x25, false);
pub(crate) static KEY_KANA: Key = new_key!("KANA", 0x15, 0x00, false);
pub(crate) static KEY_L: Key = new_key!("L", 0x4C, 0x26, false);
pub(crate) static KEY_LAUNCH_APP1: Key = new_key!("LAUNCH_APP1", 0xB6, 0x00, true);
pub(crate) static KEY_LAUNCH_APP2: Key = new_key!("LAUNCH_APP2", 0xB7, 0x00, true);
pub(crate) static KEY_LAUNCH_MAIL: Key = new_key!("LAUNCH_MAIL", 0xB4, 0x00, true);
pub(crate) static KEY_LAUNCH_MEDIA_SELECT: Key = new_key!("LAUNCH_MEDIA_SELECT", 0xB5, 0x6D, true);
pub(crate) static KEY_LEFT: Key = new_key!("LEFT", 0x25, 0x4B, true);
pub(crate) static KEY_LEFT_ALT: Key = new_key!("LEFT_ALT", 0xA4, 0x38, false);
pub(crate) static KEY_LEFT_BRACKET: Key = new_key!("LEFT_BRACKET", 0xDB, 0x1A, false);
pub(crate) static KEY_LEFT_BUTTON: Key = new_key!("LEFT_BUTTON", 0x01, 0x00, false);
pub(crate) static KEY_LEFT_CTRL: Key = new_key!("LEFT_CTRL", 0xA2, 0x1D, false);
pub(crate) static KEY_LEFT_SHIFT: Key = new_key!("LEFT_SHIFT", 0xA0, 0x2A, false);
pub(crate) static KEY_LEFT_WIN: Key = new_key!("LEFT_WIN", 0x5B, 0x5B, true);
pub(crate) static KEY_M: Key = new_key!("M", 0x4D, 0x32, false);
pub(crate) static KEY_MEDIA_NEXT_TRACK: Key = new_key!("MEDIA_NEXT_TRACK", 0xB0, 0x00, true);
pub(crate) static KEY_MEDIA_PLAY_PAUSE: Key = new_key!("MEDIA_PLAY_PAUSE", 0xB3, 0x00, true);
pub(crate) static KEY_MEDIA_PREV_TRACK: Key = new_key!("MEDIA_PREV_TRACK", 0xB1, 0x00, true);
pub(crate) static KEY_MEDIA_STOP: Key = new_key!("MEDIA_STOP", 0xB2, 0x24, true);
pub(crate) static KEY_MENU: Key = new_key!("MENU", 0x12, 0x38, false);
pub(crate) static KEY_MIDDLE_BUTTON: Key = new_key!("MIDDLE_BUTTON", 0x04, 0x00, false);
pub(crate) static KEY_MINUS: Key = new_key!("MINUS", 0xBD, 0x0C, false);
pub(crate) static KEY_MODE_CHANGE: Key = new_key!("MODE_CHANGE", 0x1F, 0x00, false);
pub(crate) static KEY_MOUSE: Key = new_key!("MOUSE", 0xF0, 0x00, true);
pub(crate) static KEY_N: Key = new_key!("N", 0x4E, 0x31, false);
pub(crate) static KEY_NONAME: Key = new_key!("NONAME", 0xFC, 0x00, false);
pub(crate) static KEY_NON_CONVERT: Key = new_key!("NON_CONVERT", 0x1D, 0x00, false);
pub(crate) static KEY_NUM_0: Key = new_key!("NUM_0", 0x60, 0x52, false);
pub(crate) static KEY_NUM_1: Key = new_key!("NUM_1", 0x61, 0x4F, false);
pub(crate) static KEY_NUM_2: Key = new_key!("NUM_2", 0x62, 0x50, false);
pub(crate) static KEY_NUM_3: Key = new_key!("NUM_3", 0x63, 0x51, false);
pub(crate) static KEY_NUM_4: Key = new_key!("NUM_4", 0x64, 0x4B, false);
pub(crate) static KEY_NUM_5: Key = new_key!("NUM_5", 0x65, 0x4C, false);
pub(crate) static KEY_NUM_6: Key = new_key!("NUM_6", 0x66, 0x4D, false);
pub(crate) static KEY_NUM_7: Key = new_key!("NUM_7", 0x67, 0x47, false);
pub(crate) static KEY_NUM_8: Key = new_key!("NUM_8", 0x68, 0x48, false);
pub(crate) static KEY_NUM_9: Key = new_key!("NUM_9", 0x69, 0x49, false);
pub(crate) static KEY_NUM_CLEAR: Key = new_key!("NUM_CLEAR", 0x0C, 0x4C, false);
pub(crate) static KEY_NUM_DELETE: Key = new_key!("NUM_DELETE", 0x2E, 0x53, false);
pub(crate) static KEY_NUM_DIV: Key = new_key!("NUM_DIV", 0x6F, 0x35, true);
pub(crate) static KEY_NUM_DOT: Key = new_key!("NUM_DOT", 0x6E, 0x53, false);
pub(crate) static KEY_NUM_DOWN: Key = new_key!("NUM_DOWN", 0x28, 0x50, false);
pub(crate) static KEY_NUM_END: Key = new_key!("NUM_END", 0x23, 0x4F, false);
pub(crate) static KEY_NUM_ENTER: Key = new_key!("NUM_ENTER", 0x0D, 0x1C, true);
pub(crate) static KEY_NUM_HOME: Key = new_key!("NUM_HOME", 0x24, 0x47, false);
pub(crate) static KEY_NUM_INSERT: Key = new_key!("NUM_INSERT", 0x2D, 0x52, false);
pub(crate) static KEY_NUM_LEFT: Key = new_key!("NUM_LEFT", 0x25, 0x4B, false);
pub(crate) static KEY_NUM_LOCK: Key = new_key!("NUM_LOCK", 0x90, 0x45, true);
pub(crate) static KEY_NUM_LOCK_2: Key = new_key!("NUM_LOCK_2", 0x13, 0x45, true); /* CTRL + NUM_LOCK*/
pub(crate) static KEY_NUM_MINUS: Key = new_key!("NUM_MINUS", 0x6D, 0x4A, false);
pub(crate) static KEY_NUM_MUL: Key = new_key!("NUM_MUL", 0x6A, 0x37, false);
pub(crate) static KEY_NUM_PAGE_DOWN: Key = new_key!("NUM_PAGE_DOWN", 0x22, 0x51, false);
pub(crate) static KEY_NUM_PAGE_UP: Key = new_key!("NUM_PAGE_UP", 0x21, 0x49, false);
pub(crate) static KEY_NUM_PLUS: Key = new_key!("NUM_PLUS", 0x6B, 0x4E, false);
pub(crate) static KEY_NUM_RIGHT: Key = new_key!("NUM_RIGHT", 0x27, 0x4D, false);
pub(crate) static KEY_NUM_UP: Key = new_key!("NUM_UP", 0x26, 0x48, false);
pub(crate) static KEY_O: Key = new_key!("O", 0x4F, 0x18, false);
pub(crate) static KEY_OEM_8: Key = new_key!("OEM_8", 0xDF, 0x00, false);
pub(crate) static KEY_OEM_CLEAR: Key = new_key!("OEM_CLEAR", 0xFE, 0x00, false);
pub(crate) static KEY_P: Key = new_key!("P", 0x50, 0x19, false);
pub(crate) static KEY_PA1: Key = new_key!("PA1", 0xFD, 0x00, false);
pub(crate) static KEY_PACKET: Key = new_key!("PACKET", 0xE7, 0x00, false);
pub(crate) static KEY_PAGE_DOWN: Key = new_key!("PAGE_DOWN", 0x22, 0x51, true);
pub(crate) static KEY_PAGE_UP: Key = new_key!("PAGE_UP", 0x21, 0x49, true);
pub(crate) static KEY_PAUSE: Key = new_key!("PAUSE", 0x13, 0x45, false);
pub(crate) static KEY_PLAY: Key = new_key!("PLAY", 0xFA, 0x00, false);
pub(crate) static KEY_PLUS: Key = new_key!("PLUS", 0x00, 0x4E, true);
pub(crate) static KEY_PRINT: Key = new_key!("PRINT", 0x2A, 0x00, false);
pub(crate) static KEY_PRINT_SCREEN: Key = new_key!("PRINT_SCREEN", 0x2C, 0x37, true);
pub(crate) static KEY_PROCESS_KEY: Key = new_key!("PROCESS_KEY", 0xE5, 0x00, false);
pub(crate) static KEY_Q: Key = new_key!("Q", 0x51, 0x10, false);
pub(crate) static KEY_R: Key = new_key!("R", 0x52, 0x13, false);
pub(crate) static KEY_RIGHT: Key = new_key!("RIGHT", 0x27, 0x4D, true);
pub(crate) static KEY_RIGHT_ALT: Key = new_key!("RIGHT_ALT", 0xA5, 0x38, true);
pub(crate) static KEY_RIGHT_BRACKET: Key = new_key!("RIGHT_BRACKET", 0xDD, 0x1B, false);
pub(crate) static KEY_RIGHT_BUTTON: Key = new_key!("RIGHT_BUTTON", 0x02, 0x00, false);
pub(crate) static KEY_RIGHT_CTRL: Key = new_key!("RIGHT_CTRL", 0xA3, 0x1D, true);
pub(crate) static KEY_RIGHT_SHIFT: Key = new_key!("RIGHT_SHIFT", 0xA1, 0x36, true);
pub(crate) static KEY_RIGHT_SHIFT_2: Key = new_key!("RIGHT_SHIFT_2", 0x00, 0x36, true);
pub(crate) static KEY_RIGHT_WIN: Key = new_key!("RIGHT_WIN", 0x5C, 0x5C, true);
pub(crate) static KEY_S: Key = new_key!("S", 0x53, 0x1F, false);
pub(crate) static KEY_SCROLL_LOCK: Key = new_key!("SCROLL_LOCK", 0x91, 0x46, false);
pub(crate) static KEY_SELECT: Key = new_key!("SELECT", 0x29, 0x00, false);
pub(crate) static KEY_SEMICOLON: Key = new_key!("SEMICOLON", 0xBA, 0x27, false);
pub(crate) static KEY_SEPARATOR: Key = new_key!("SEPARATOR", 0x6C, 0x00, false);
pub(crate) static KEY_SHIFT: Key = new_key!("SHIFT", 0x10, 0x2A, false);
pub(crate) static KEY_SLASH: Key = new_key!("SLASH", 0xBF, 0x35, false);
pub(crate) static KEY_SLEEP: Key = new_key!("SLEEP", 0x5F, 0x5F, true);
pub(crate) static KEY_SPACE: Key = new_key!("SPACE", 0x20, 0x39, false);
pub(crate) static KEY_SYS_REQ: Key = new_key!("SYS_REQ", 0x2C, 0x54, false);
pub(crate) static KEY_T: Key = new_key!("T", 0x54, 0x14, false);
pub(crate) static KEY_TAB: Key = new_key!("TAB", 0x09, 0x0F, false);
pub(crate) static KEY_U: Key = new_key!("U", 0x55, 0x16, false);
pub(crate) static KEY_UNASSIGNED: Key = new_key!("UNASSIGNED", 0, 0, false);
pub(crate) static KEY_UP: Key = new_key!("UP", 0x26, 0x48, true);
pub(crate) static KEY_V: Key = new_key!("V", 0x56, 0x2F, false);
pub(crate) static KEY_VOLUME_DOWN: Key = new_key!("VOLUME_DOWN", 0xAE, 0x00, true);
pub(crate) static KEY_VOLUME_MUTE: Key = new_key!("VOLUME_MUTE", 0xAD, 0x00, true);
pub(crate) static KEY_VOLUME_UP: Key = new_key!("VOLUME_UP", 0xAF, 0x00, true);
pub(crate) static KEY_W: Key = new_key!("W", 0x57, 0x11, false);
pub(crate) static KEY_WHEEL: Key = new_key!("WHEEL", 0xF1, 0x00, true);
pub(crate) static KEY_X: Key = new_key!("X", 0x58, 0x2D, false);
pub(crate) static KEY_XBUTTON1: Key = new_key!("XBUTTON1", 0x05, 0x00, false);
pub(crate) static KEY_XBUTTON2: Key = new_key!("XBUTTON2", 0x06, 0x00, false);
pub(crate) static KEY_Y: Key = new_key!("Y", 0x59, 0x15, false);
pub(crate) static KEY_Z: Key = new_key!("Z", 0x5A, 0x2C, false);
pub(crate) static KEY_ZOOM: Key = new_key!("ZOOM", 0xFB, 0x62, false);
pub(crate) static KEY__: Key = new_key!("_", 0x00, 0x39, true);
pub(crate) static KEY__ESC: Key = new_key!("", 0x00, 0x01, true);
pub(crate) static KEY___: Key = new_key!("	", 0x00, 0x0F, true);

macro_rules! key_entry {
    ($key:expr) => {
        ($key.name, $key)
    };
}

pub const MAX_KEYS: usize = 206;

pub static KEYS: [(&'static str, Key); MAX_KEYS] = [
    key_entry!(KEY_0),
    key_entry!(KEY_00),
    key_entry!(KEY_1),
    key_entry!(KEY_2),
    key_entry!(KEY_3),
    key_entry!(KEY_4),
    key_entry!(KEY_5),
    key_entry!(KEY_6),
    key_entry!(KEY_7),
    key_entry!(KEY_8),
    key_entry!(KEY_9),
    key_entry!(KEY_A),
    key_entry!(KEY_ACCEPT),
    key_entry!(KEY_APOSTROPHE),
    key_entry!(KEY_APPLICATION),
    key_entry!(KEY_ATTN),
    key_entry!(KEY_B),
    key_entry!(KEY_BACKSLASH),
    key_entry!(KEY_BACKSLASH_2),
    key_entry!(KEY_BACKSPACE),
    key_entry!(KEY_BACKTICK),
    key_entry!(KEY_BREAK),
    key_entry!(KEY_BRIGHTNESS),
    key_entry!(KEY_BROWSER_BACK),
    key_entry!(KEY_BROWSER_FAVORITES),
    key_entry!(KEY_BROWSER_FORWARD),
    key_entry!(KEY_BROWSER_HOME),
    key_entry!(KEY_BROWSER_REFRESH),
    key_entry!(KEY_BROWSER_SEARCH),
    key_entry!(KEY_BROWSER_STOP),
    key_entry!(KEY_C),
    key_entry!(KEY_CAPS_LOCK),
    key_entry!(KEY_COMMA),
    key_entry!(KEY_CONVERT),
    key_entry!(KEY_CRSEL),
    key_entry!(KEY_CTRL),
    key_entry!(KEY_D),
    key_entry!(KEY_DELETE),
    key_entry!(KEY_DOT),
    key_entry!(KEY_DOWN),
    key_entry!(KEY_E),
    key_entry!(KEY_END),
    key_entry!(KEY_ENTER),
    key_entry!(KEY_EQ),
    key_entry!(KEY_EREOF),
    key_entry!(KEY_ESC),
    key_entry!(KEY_EXECUTE),
    key_entry!(KEY_EXSEL),
    key_entry!(KEY_F),
    key_entry!(KEY_F1),
    key_entry!(KEY_F10),
    key_entry!(KEY_F11),
    key_entry!(KEY_F12),
    key_entry!(KEY_F13),
    key_entry!(KEY_F14),
    key_entry!(KEY_F15),
    key_entry!(KEY_F16),
    key_entry!(KEY_F17),
    key_entry!(KEY_F18),
    key_entry!(KEY_F19),
    key_entry!(KEY_F2),
    key_entry!(KEY_F20),
    key_entry!(KEY_F21),
    key_entry!(KEY_F22),
    key_entry!(KEY_F23),
    key_entry!(KEY_F24),
    key_entry!(KEY_F3),
    key_entry!(KEY_F4),
    key_entry!(KEY_F5),
    key_entry!(KEY_F6),
    key_entry!(KEY_F7),
    key_entry!(KEY_F8),
    key_entry!(KEY_F9),
    key_entry!(KEY_FINAL),
    key_entry!(KEY_FN_BROWSER_HOME),
    key_entry!(KEY_FN_BROWSER_SEARCH),
    key_entry!(KEY_FN_LAUNCH_APP1),
    key_entry!(KEY_FN_LAUNCH_APP2),
    key_entry!(KEY_FN_LAUNCH_MAIL),
    key_entry!(KEY_FN_MEDIA_NEXT_TRACK),
    key_entry!(KEY_FN_MEDIA_PLAY_PAUSE),
    key_entry!(KEY_FN_MEDIA_PREV_TRACK),
    key_entry!(KEY_FN_VOLUME_DOWN),
    key_entry!(KEY_FN_VOLUME_MUTE),
    key_entry!(KEY_FN_VOLUME_UP),
    key_entry!(KEY_G),
    key_entry!(KEY_H),
    key_entry!(KEY_HANJA),
    key_entry!(KEY_HELP),
    key_entry!(KEY_HOME),
    key_entry!(KEY_I),
    key_entry!(KEY_IME_OFF),
    key_entry!(KEY_IME_ON),
    key_entry!(KEY_INSERT),
    key_entry!(KEY_J),
    key_entry!(KEY_JUNJA),
    key_entry!(KEY_K),
    key_entry!(KEY_KANA),
    key_entry!(KEY_L),
    key_entry!(KEY_LAUNCH_APP1),
    key_entry!(KEY_LAUNCH_APP2),
    key_entry!(KEY_LAUNCH_MAIL),
    key_entry!(KEY_LAUNCH_MEDIA_SELECT),
    key_entry!(KEY_LEFT),
    key_entry!(KEY_LEFT_ALT),
    key_entry!(KEY_LEFT_BRACKET),
    key_entry!(KEY_LEFT_BUTTON),
    key_entry!(KEY_LEFT_CTRL),
    key_entry!(KEY_LEFT_SHIFT),
    key_entry!(KEY_LEFT_WIN),
    key_entry!(KEY_M),
    key_entry!(KEY_MEDIA_NEXT_TRACK),
    key_entry!(KEY_MEDIA_PLAY_PAUSE),
    key_entry!(KEY_MEDIA_PREV_TRACK),
    key_entry!(KEY_MEDIA_STOP),
    key_entry!(KEY_MENU),
    key_entry!(KEY_MIDDLE_BUTTON),
    key_entry!(KEY_MINUS),
    key_entry!(KEY_MODE_CHANGE),
    key_entry!(KEY_MOUSE),
    key_entry!(KEY_N),
    key_entry!(KEY_NONAME),
    key_entry!(KEY_NON_CONVERT),
    key_entry!(KEY_NUM_0),
    key_entry!(KEY_NUM_1),
    key_entry!(KEY_NUM_2),
    key_entry!(KEY_NUM_3),
    key_entry!(KEY_NUM_4),
    key_entry!(KEY_NUM_5),
    key_entry!(KEY_NUM_6),
    key_entry!(KEY_NUM_7),
    key_entry!(KEY_NUM_8),
    key_entry!(KEY_NUM_9),
    key_entry!(KEY_NUM_CLEAR),
    key_entry!(KEY_NUM_DELETE),
    key_entry!(KEY_NUM_DIV),
    key_entry!(KEY_NUM_DOT),
    key_entry!(KEY_NUM_DOWN),
    key_entry!(KEY_NUM_END),
    key_entry!(KEY_NUM_ENTER),
    key_entry!(KEY_NUM_HOME),
    key_entry!(KEY_NUM_INSERT),
    key_entry!(KEY_NUM_LEFT),
    key_entry!(KEY_NUM_LOCK),
    key_entry!(KEY_NUM_LOCK_2),
    key_entry!(KEY_NUM_MINUS),
    key_entry!(KEY_NUM_MUL),
    key_entry!(KEY_NUM_PAGE_DOWN),
    key_entry!(KEY_NUM_PAGE_UP),
    key_entry!(KEY_NUM_PLUS),
    key_entry!(KEY_NUM_RIGHT),
    key_entry!(KEY_NUM_UP),
    key_entry!(KEY_O),
    key_entry!(KEY_OEM_8),
    key_entry!(KEY_OEM_CLEAR),
    key_entry!(KEY_P),
    key_entry!(KEY_PA1),
    key_entry!(KEY_PACKET),
    key_entry!(KEY_PAGE_DOWN),
    key_entry!(KEY_PAGE_UP),
    key_entry!(KEY_PAUSE),
    key_entry!(KEY_PLAY),
    key_entry!(KEY_PLUS),
    key_entry!(KEY_PRINT),
    key_entry!(KEY_PRINT_SCREEN),
    key_entry!(KEY_PROCESS_KEY),
    key_entry!(KEY_Q),
    key_entry!(KEY_R),
    key_entry!(KEY_RIGHT),
    key_entry!(KEY_RIGHT_ALT),
    key_entry!(KEY_RIGHT_BRACKET),
    key_entry!(KEY_RIGHT_BUTTON),
    key_entry!(KEY_RIGHT_CTRL),
    key_entry!(KEY_RIGHT_SHIFT),
    key_entry!(KEY_RIGHT_SHIFT_2),
    key_entry!(KEY_RIGHT_WIN),
    key_entry!(KEY_S),
    key_entry!(KEY_SCROLL_LOCK),
    key_entry!(KEY_SELECT),
    key_entry!(KEY_SEMICOLON),
    key_entry!(KEY_SEPARATOR),
    key_entry!(KEY_SHIFT),
    key_entry!(KEY_SLASH),
    key_entry!(KEY_SLEEP),
    key_entry!(KEY_SPACE),
    key_entry!(KEY_SYS_REQ),
    key_entry!(KEY_T),
    key_entry!(KEY_TAB),
    key_entry!(KEY_U),
    key_entry!(KEY_UNASSIGNED),
    key_entry!(KEY_UP),
    key_entry!(KEY_V),
    key_entry!(KEY_VOLUME_DOWN),
    key_entry!(KEY_VOLUME_MUTE),
    key_entry!(KEY_VOLUME_UP),
    key_entry!(KEY_W),
    key_entry!(KEY_WHEEL),
    key_entry!(KEY_X),
    key_entry!(KEY_XBUTTON1),
    key_entry!(KEY_XBUTTON2),
    key_entry!(KEY_Y),
    key_entry!(KEY_Z),
    key_entry!(KEY_ZOOM),
    key_entry!(KEY__),
    key_entry!(KEY__ESC),
    key_entry!(KEY___),
];

#[cfg(test)]
mod tests {
    use crate::keyboard::key::{Key, KEYS, KEY_MAP};
    use crate::utils::test::SerdeWrapper;
    use std::str::FromStr;

    #[test]
    fn test_key_by_name() {
        assert!(
            KEYS.iter()
                .all(|(name, key)| KEY_MAP.with(|k| k.find_by_name(name).unwrap()) == *key)
        )
    }

    #[test]
    fn test_key_name() {
        assert!(
            KEYS.iter()
                .all(|(name, key)| KEY_MAP.with(|_| key.name == *name))
        )
    }

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
