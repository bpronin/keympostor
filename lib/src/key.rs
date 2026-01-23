use crate::error::KeyError;
use crate::sc::ScanCode;
use crate::vk::VirtualKey;
use crate::{deserialize_from_string, serialize_to_string};
use phf::phf_map;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Key {
    pub name: &'static str,
    pub vk: VirtualKey,
    pub sc: ScanCode,
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.name, f)
    }
}

impl FromStr for Key {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        key_by_name(s).copied()
    }
}

impl Serialize for Key {
    serialize_to_string!();
}

impl<'de> Deserialize<'de> for Key {
    deserialize_from_string!();
}

pub fn key_by_code(vk_code: u8, sc_code: u8, sc_ext: bool) -> &'static Key {
    CODE_TO_KEY_MAP
        .get(&key_code(vk_code, sc_code, sc_ext))
        .expect(&format!(
            "Unsupported key code: `{:?} {:?}`.",
            vk_code, sc_code
        ))
}

pub fn key_by_name(name: &str) -> Result<&'static Key, KeyError> {
    NAME_TO_KEY_MAP
        .get(name.trim())
        .ok_or(KeyError::new(&format!("Illegal key name: `{name}`.")))
}

#[macro_export]
macro_rules! key {
    ($text:literal) => {
        key_by_name($text).unwrap()
    };
}

macro_rules! new_key {
    ($name:literal, $vk_code:literal, $scan_code:literal, $is_ext_scan_code:literal) => {
        Key {
            name: $name,
            vk: VirtualKey($vk_code),
            sc: ScanCode($scan_code, $is_ext_scan_code),
        }
    };
}

pub static KEY_00: Key = new_key!("<00>", 0x00, 0x54, true);
pub static KEY_0: Key = new_key!("0", 0x30, 0x0B, false);
pub static KEY_1: Key = new_key!("1", 0x31, 0x02, false);
pub static KEY_2: Key = new_key!("2", 0x32, 0x03, false);
pub static KEY_3: Key = new_key!("3", 0x33, 0x04, false);
pub static KEY_4: Key = new_key!("4", 0x34, 0x05, false);
pub static KEY_5: Key = new_key!("5", 0x35, 0x06, false);
pub static KEY_6: Key = new_key!("6", 0x36, 0x07, false);
pub static KEY_7: Key = new_key!("7", 0x37, 0x08, false);
pub static KEY_8: Key = new_key!("8", 0x38, 0x09, false);
pub static KEY_9: Key = new_key!("9", 0x39, 0x0A, false);
pub static KEY_A: Key = new_key!("A", 0x41, 0x1E, false);
pub static KEY_ACCEPT: Key = new_key!("ACCEPT", 0x1E, 0x00, false);
pub static KEY_APOSTROPHE: Key = new_key!("APOSTROPHE", 0xDE, 0x28, false);
pub static KEY_APPLICATION: Key = new_key!("APPLICATION", 0x5D, 0x5D, true);
pub static KEY_ATTN: Key = new_key!("ATTN", 0xF6, 0x00, false);
pub static KEY_B: Key = new_key!("B", 0x42, 0x30, false);
pub static KEY_BACKSLASH: Key = new_key!("BACKSLASH", 0xDC, 0x2B, false);
pub static KEY_BACKSLASH_2: Key = new_key!("BACKSLASH_2", 0xE2, 0x56, false);
pub static KEY_BACKSPACE: Key = new_key!("BACKSPACE", 0x08, 0x0E, false);
pub static KEY_BACKTICK: Key = new_key!("BACKTICK", 0xC0, 0x29, false);
pub static KEY_BREAK: Key = new_key!("BREAK", 0x03, 0x46, true);
pub static KEY_BRIGHTNESS: Key = new_key!("BRIGHTNESS", 0x00, 0x2B, true);
pub static KEY_BROWSER_BACK: Key = new_key!("BROWSER_BACK", 0xA6, 0x6A, true);
pub static KEY_BROWSER_FAVORITES: Key = new_key!("BROWSER_FAVORITES", 0xAB, 0x66, true);
pub static KEY_BROWSER_FORWARD: Key = new_key!("BROWSER_FORWARD", 0xA7, 0x69, true);
pub static KEY_BROWSER_HOME: Key = new_key!("BROWSER_HOME", 0xAC, 0x00, true);
pub static KEY_BROWSER_REFRESH: Key = new_key!("BROWSER_REFRESH", 0xA8, 0x67, true);
pub static KEY_BROWSER_SEARCH: Key = new_key!("BROWSER_SEARCH", 0xAA, 0x00, true);
pub static KEY_BROWSER_STOP: Key = new_key!("BROWSER_STOP", 0xA9, 0x68, true);
pub static KEY_C: Key = new_key!("C", 0x43, 0x2E, false);
pub static KEY_CAPS_LOCK: Key = new_key!("CAPS_LOCK", 0x14, 0x3A, false);
pub static KEY_COMMA: Key = new_key!("COMMA", 0xBC, 0x33, false);
pub static KEY_CONVERT: Key = new_key!("CONVERT", 0x1C, 0x00, false);
pub static KEY_CRSEL: Key = new_key!("CRSEL", 0xF7, 0x00, false);
pub static KEY_CTRL: Key = new_key!("CTRL", 0x11, 0x1D, false);
pub static KEY_D: Key = new_key!("D", 0x44, 0x20, false);
pub static KEY_DELETE: Key = new_key!("DELETE", 0x2E, 0x53, true);
pub static KEY_DOT: Key = new_key!("DOT", 0xBE, 0x34, false);
pub static KEY_DOWN: Key = new_key!("DOWN", 0x28, 0x50, true);
pub static KEY_E: Key = new_key!("E", 0x45, 0x12, false);
pub static KEY_END: Key = new_key!("END", 0x23, 0x4F, true);
pub static KEY_ENTER: Key = new_key!("ENTER", 0x0D, 0x1C, false);
pub static KEY_EQ: Key = new_key!("EQ", 0xBB, 0x0D, false);
pub static KEY_EREOF: Key = new_key!("EREOF", 0xF9, 0x5D, false);
pub static KEY_ESC: Key = new_key!("ESC", 0x1B, 0x01, false);
pub static KEY_EXECUTE: Key = new_key!("EXECUTE", 0x2B, 0x00, false);
pub static KEY_EXSEL: Key = new_key!("EXSEL", 0xF8, 0x00, false);
pub static KEY_F10: Key = new_key!("F10", 0x79, 0x44, false);
pub static KEY_F11: Key = new_key!("F11", 0x7A, 0x57, false);
pub static KEY_F12: Key = new_key!("F12", 0x7B, 0x58, false);
pub static KEY_F13: Key = new_key!("F13", 0x7C, 0x64, false);
pub static KEY_F14: Key = new_key!("F14", 0x7D, 0x65, false);
pub static KEY_F15: Key = new_key!("F15", 0x7E, 0x66, false);
pub static KEY_F16: Key = new_key!("F16", 0x7F, 0x67, false);
pub static KEY_F17: Key = new_key!("F17", 0x80, 0x68, false);
pub static KEY_F18: Key = new_key!("F18", 0x81, 0x69, false);
pub static KEY_F19: Key = new_key!("F19", 0x82, 0x6A, false);
pub static KEY_F1: Key = new_key!("F1", 0x70, 0x3B, false);
pub static KEY_F20: Key = new_key!("F20", 0x83, 0x6B, false);
pub static KEY_F21: Key = new_key!("F21", 0x84, 0x6C, false);
pub static KEY_F22: Key = new_key!("F22", 0x85, 0x6D, false);
pub static KEY_F23: Key = new_key!("F23", 0x86, 0x6E, false);
pub static KEY_F24: Key = new_key!("F24", 0x87, 0x76, false);
pub static KEY_F2: Key = new_key!("F2", 0x71, 0x3C, false);
pub static KEY_F3: Key = new_key!("F3", 0x72, 0x3D, false);
pub static KEY_F4: Key = new_key!("F4", 0x73, 0x3E, false);
pub static KEY_F5: Key = new_key!("F5", 0x74, 0x3F, false);
pub static KEY_F6: Key = new_key!("F6", 0x75, 0x40, false);
pub static KEY_F7: Key = new_key!("F7", 0x76, 0x41, false);
pub static KEY_F8: Key = new_key!("F8", 0x77, 0x42, false);
pub static KEY_F9: Key = new_key!("F9", 0x78, 0x43, false);
pub static KEY_F: Key = new_key!("F", 0x46, 0x21, false);
pub static KEY_FINAL: Key = new_key!("FINAL", 0x18, 0x00, false);
pub static KEY_FN_BROWSER_HOME: Key = new_key!("FN_BROWSER_HOME", 0xAC, 0x32, true);
pub static KEY_FN_BROWSER_SEARCH: Key = new_key!("FN_BROWSER_SEARCH", 0xAA, 0x65, true);
pub static KEY_FN_LAUNCH_APP1: Key = new_key!("FN_LAUNCH_APP1", 0xB6, 0x6B, true);
pub static KEY_FN_LAUNCH_APP2: Key = new_key!("FN_LAUNCH_APP2", 0xB7, 0x21, true);
pub static KEY_FN_LAUNCH_MAIL: Key = new_key!("FN_LAUNCH_MAIL", 0xB4, 0x6C, true);
pub static KEY_FN_MEDIA_NEXT_TRACK: Key = new_key!("FN_MEDIA_NEXT_TRACK", 0xB0, 0x19, true);
pub static KEY_FN_MEDIA_PLAY_PAUSE: Key = new_key!("FN_MEDIA_PLAY_PAUSE", 0xB3, 0x22, true);
pub static KEY_FN_MEDIA_PREV_TRACK: Key = new_key!("FN_MEDIA_PREV_TRACK", 0xB1, 0x10, true);
pub static KEY_FN_VOLUME_DOWN: Key = new_key!("FN_VOLUME_DOWN", 0xAE, 0x2E, true);
pub static KEY_FN_VOLUME_MUTE: Key = new_key!("FN_VOLUME_MUTE", 0xAD, 0x20, true);
pub static KEY_FN_VOLUME_UP: Key = new_key!("FN_VOLUME_UP", 0xAF, 0x30, true);
pub static KEY_G: Key = new_key!("G", 0x47, 0x22, false);
pub static KEY_H: Key = new_key!("H", 0x48, 0x23, false);
pub static KEY_HANJA: Key = new_key!("HANJA", 0x19, 0x00, false);
pub static KEY_HELP: Key = new_key!("HELP", 0x2F, 0x63, false);
pub static KEY_HOME: Key = new_key!("HOME", 0x24, 0x47, true);
pub static KEY_I: Key = new_key!("I", 0x49, 0x17, false);
pub static KEY_IME_OFF: Key = new_key!("IME_OFF", 0x1A, 0x00, false);
pub static KEY_IME_ON: Key = new_key!("IME_ON", 0x16, 0x00, false);
pub static KEY_INSERT: Key = new_key!("INSERT", 0x2D, 0x52, true);
pub static KEY_J: Key = new_key!("J", 0x4A, 0x24, false);
pub static KEY_JUNJA: Key = new_key!("JUNJA", 0x17, 0x00, false);
pub static KEY_K: Key = new_key!("K", 0x4B, 0x25, false);
pub static KEY_KANA: Key = new_key!("KANA", 0x15, 0x00, false);
pub static KEY_L: Key = new_key!("L", 0x4C, 0x26, false);
pub static KEY_LAUNCH_APP1: Key = new_key!("LAUNCH_APP1", 0xB6, 0x00, true);
pub static KEY_LAUNCH_APP2: Key = new_key!("LAUNCH_APP2", 0xB7, 0x00, true);
pub static KEY_LAUNCH_MAIL: Key = new_key!("LAUNCH_MAIL", 0xB4, 0x00, true);
pub static KEY_LAUNCH_MEDIA_SELECT: Key = new_key!("LAUNCH_MEDIA_SELECT", 0xB5, 0x6D, true);
pub static KEY_LEFT: Key = new_key!("LEFT", 0x25, 0x4B, true);
pub static KEY_LEFT_ALT: Key = new_key!("LEFT_ALT", 0xA4, 0x38, false);
pub static KEY_LEFT_BRACKET: Key = new_key!("LEFT_BRACKET", 0xDB, 0x1A, false);
pub static KEY_LEFT_BUTTON: Key = new_key!("LEFT_BUTTON", 0x01, 0x00, false);
pub static KEY_LEFT_CTRL: Key = new_key!("LEFT_CTRL", 0xA2, 0x1D, false);
pub static KEY_LEFT_SHIFT: Key = new_key!("LEFT_SHIFT", 0xA0, 0x2A, false);
pub static KEY_LEFT_WIN: Key = new_key!("LEFT_WIN", 0x5B, 0x5B, true);
pub static KEY_M: Key = new_key!("M", 0x4D, 0x32, false);
pub static KEY_MEDIA_NEXT_TRACK: Key = new_key!("MEDIA_NEXT_TRACK", 0xB0, 0x00, true);
pub static KEY_MEDIA_PLAY_PAUSE: Key = new_key!("MEDIA_PLAY_PAUSE", 0xB3, 0x00, true);
pub static KEY_MEDIA_PREV_TRACK: Key = new_key!("MEDIA_PREV_TRACK", 0xB1, 0x00, true);
pub static KEY_MEDIA_STOP: Key = new_key!("MEDIA_STOP", 0xB2, 0x24, true);
pub static KEY_MENU: Key = new_key!("MENU", 0x12, 0x38, false);
pub static KEY_MIDDLE_BUTTON: Key = new_key!("MIDDLE_BUTTON", 0x04, 0x00, false);
pub static KEY_MINUS: Key = new_key!("MINUS", 0xBD, 0x0C, false);
pub static KEY_MODE_CHANGE: Key = new_key!("MODE_CHANGE", 0x1F, 0x00, false);
pub static KEY_N: Key = new_key!("N", 0x4E, 0x31, false);
pub static KEY_NONAME: Key = new_key!("NONAME", 0xFC, 0x00, false);
pub static KEY_NON_CONVERT: Key = new_key!("NON_CONVERT", 0x1D, 0x00, false);
pub static KEY_NUM_0: Key = new_key!("NUM_0", 0x60, 0x52, false);
pub static KEY_NUM_1: Key = new_key!("NUM_1", 0x61, 0x4F, false);
pub static KEY_NUM_2: Key = new_key!("NUM_2", 0x62, 0x50, false);
pub static KEY_NUM_3: Key = new_key!("NUM_3", 0x63, 0x51, false);
pub static KEY_NUM_4: Key = new_key!("NUM_4", 0x64, 0x4B, false);
pub static KEY_NUM_5: Key = new_key!("NUM_5", 0x65, 0x4C, false);
pub static KEY_NUM_6: Key = new_key!("NUM_6", 0x66, 0x4D, false);
pub static KEY_NUM_7: Key = new_key!("NUM_7", 0x67, 0x47, false);
pub static KEY_NUM_8: Key = new_key!("NUM_8", 0x68, 0x48, false);
pub static KEY_NUM_9: Key = new_key!("NUM_9", 0x69, 0x49, false);
pub static KEY_NUM_CLEAR: Key = new_key!("NUM_CLEAR", 0x0C, 0x4C, false);
pub static KEY_NUM_DELETE: Key = new_key!("NUM_DELETE", 0x2E, 0x53, false);
pub static KEY_NUM_DIV: Key = new_key!("NUM_DIV", 0x6F, 0x35, true);
pub static KEY_NUM_DOT: Key = new_key!("NUM_DOT", 0x6E, 0x53, false);
pub static KEY_NUM_DOWN: Key = new_key!("NUM_DOWN", 0x28, 0x50, false);
pub static KEY_NUM_END: Key = new_key!("NUM_END", 0x23, 0x4F, false);
pub static KEY_NUM_ENTER: Key = new_key!("NUM_ENTER", 0x0D, 0x1C, true);
pub static KEY_NUM_HOME: Key = new_key!("NUM_HOME", 0x24, 0x47, false);
pub static KEY_NUM_INSERT: Key = new_key!("NUM_INSERT", 0x2D, 0x52, false);
pub static KEY_NUM_LEFT: Key = new_key!("NUM_LEFT", 0x25, 0x4B, false);
pub static KEY_NUM_LOCK: Key = new_key!("NUM_LOCK", 0x90, 0x45, true);
pub static KEY_NUM_LOCK_2: Key = new_key!("NUM_LOCK_2", 0x13, 0x45, true); /* CTRL + NUM_LOCK*/
pub static KEY_NUM_MINUS: Key = new_key!("NUM_MINUS", 0x6D, 0x4A, false);
pub static KEY_NUM_MUL: Key = new_key!("NUM_MUL", 0x6A, 0x37, false);
pub static KEY_NUM_PAGE_DOWN: Key = new_key!("NUM_PAGE_DOWN", 0x22, 0x51, false);
pub static KEY_NUM_PAGE_UP: Key = new_key!("NUM_PAGE_UP", 0x21, 0x49, false);
pub static KEY_NUM_PLUS: Key = new_key!("NUM_PLUS", 0x6B, 0x4E, false);
pub static KEY_NUM_RIGHT: Key = new_key!("NUM_RIGHT", 0x27, 0x4D, false);
pub static KEY_NUM_UP: Key = new_key!("NUM_UP", 0x26, 0x48, false);
pub static KEY_O: Key = new_key!("O", 0x4F, 0x18, false);
pub static KEY_OEM_8: Key = new_key!("OEM_8", 0xDF, 0x00, false);
pub static KEY_OEM_CLEAR: Key = new_key!("OEM_CLEAR", 0xFE, 0x00, false);
pub static KEY_P: Key = new_key!("P", 0x50, 0x19, false);
pub static KEY_PA1: Key = new_key!("PA1", 0xFD, 0x00, false);
pub static KEY_PACKET: Key = new_key!("PACKET", 0xE7, 0x00, false);
pub static KEY_PAGE_DOWN: Key = new_key!("PAGE_DOWN", 0x22, 0x51, true);
pub static KEY_PAGE_UP: Key = new_key!("PAGE_UP", 0x21, 0x49, true);
pub static KEY_PAUSE: Key = new_key!("PAUSE", 0x13, 0x45, false);
pub static KEY_PLAY: Key = new_key!("PLAY", 0xFA, 0x00, false);
pub static KEY_PLUS: Key = new_key!("PLUS", 0x00, 0x4E, true);
pub static KEY_PRINT: Key = new_key!("PRINT", 0x2A, 0x00, false);
pub static KEY_PRINT_SCREEN: Key = new_key!("PRINT_SCREEN", 0x2C, 0x37, true);
pub static KEY_PROCESS_KEY: Key = new_key!("PROCESS_KEY", 0xE5, 0x00, false);
pub static KEY_Q: Key = new_key!("Q", 0x51, 0x10, false);
pub static KEY_R: Key = new_key!("R", 0x52, 0x13, false);
pub static KEY_RIGHT: Key = new_key!("RIGHT", 0x27, 0x4D, true);
pub static KEY_RIGHT_ALT: Key = new_key!("RIGHT_ALT", 0xA5, 0x38, true);
pub static KEY_RIGHT_BRACKET: Key = new_key!("RIGHT_BRACKET", 0xDD, 0x1B, false);
pub static KEY_RIGHT_BUTTON: Key = new_key!("RIGHT_BUTTON", 0x02, 0x00, false);
pub static KEY_RIGHT_CTRL: Key = new_key!("RIGHT_CTRL", 0xA3, 0x1D, true);
pub static KEY_RIGHT_SHIFT: Key = new_key!("RIGHT_SHIFT", 0xA1, 0x36, true);
pub static KEY_RIGHT_SHIFT_2: Key = new_key!("RIGHT_SHIFT_2", 0x00, 0x36, true);
pub static KEY_RIGHT_WIN: Key = new_key!("RIGHT_WIN", 0x5C, 0x5C, true);
pub static KEY_S: Key = new_key!("S", 0x53, 0x1F, false);
pub static KEY_SCROLL_LOCK: Key = new_key!("SCROLL_LOCK", 0x91, 0x46, false);
pub static KEY_SELECT: Key = new_key!("SELECT", 0x29, 0x00, false);
pub static KEY_SEMICOLON: Key = new_key!("SEMICOLON", 0xBA, 0x27, false);
pub static KEY_SEPARATOR: Key = new_key!("SEPARATOR", 0x6C, 0x00, false);
pub static KEY_SHIFT: Key = new_key!("SHIFT", 0x10, 0x2A, false);
pub static KEY_SLASH: Key = new_key!("SLASH", 0xBF, 0x35, false);
pub static KEY_SLEEP: Key = new_key!("SLEEP", 0x5F, 0x5F, true);
pub static KEY_SPACE: Key = new_key!("SPACE", 0x20, 0x39, false);
pub static KEY_SYS_REQ: Key = new_key!("SYS_REQ", 0x2C, 0x54, false);
pub static KEY_T: Key = new_key!("T", 0x54, 0x14, false);
pub static KEY_TAB: Key = new_key!("TAB", 0x09, 0x0F, false);
pub static KEY_U: Key = new_key!("U", 0x55, 0x16, false);
pub static KEY_UNASSIGNED: Key = new_key!("UNASSIGNED", 0, 0, false);
pub static KEY_UP: Key = new_key!("UP", 0x26, 0x48, true);
pub static KEY_V: Key = new_key!("V", 0x56, 0x2F, false);
pub static KEY_VOLUME_DOWN: Key = new_key!("VOLUME_DOWN", 0xAE, 0x00, true);
pub static KEY_VOLUME_MUTE: Key = new_key!("VOLUME_MUTE", 0xAD, 0x00, true);
pub static KEY_VOLUME_UP: Key = new_key!("VOLUME_UP", 0xAF, 0x00, true);
pub static KEY_W: Key = new_key!("W", 0x57, 0x11, false);
pub static KEY_WHEEL_X: Key = new_key!("WHEEL_X", 0xF1, 0x00, true);
pub static KEY_WHEEL_Y: Key = new_key!("WHEEL_Y", 0xF3, 0x00, true);
pub static KEY_X: Key = new_key!("X", 0x58, 0x2D, false);
pub static KEY_XBUTTON1: Key = new_key!("XBUTTON1", 0x05, 0x00, false);
pub static KEY_XBUTTON2: Key = new_key!("XBUTTON2", 0x06, 0x00, false);
pub static KEY_Y: Key = new_key!("Y", 0x59, 0x15, false);
pub static KEY_Z: Key = new_key!("Z", 0x5A, 0x2C, false);
pub static KEY_ZOOM: Key = new_key!("ZOOM", 0xFB, 0x62, false);
pub static KEY__: Key = new_key!("_", 0x00, 0x39, true);
pub static KEY__ESC: Key = new_key!("<ESC>", 0x00, 0x01, true);
pub static KEY__TAB: Key = new_key!("<TAB>", 0x00, 0x0F, true);

fn key_code(vk_code: u8, sc_code: u8, sc_ext: bool) -> u32 {
    (vk_code as u32) << 16 | (sc_code as u32) << 8 | (sc_ext as u32)
}

pub(crate) static CODE_TO_KEY_MAP: phf::Map<u32, Key> = phf_map! {
    0x000000 => KEY_UNASSIGNED,
    0x000101 => KEY__ESC,
    0x000F01 => KEY__TAB,
    0x002B01 => KEY_BRIGHTNESS,
    0x003601 => KEY_RIGHT_SHIFT_2,
    0x003901 => KEY__,
    0x004E01 => KEY_PLUS,
    0x005401 => KEY_00,
    0x010000 => KEY_LEFT_BUTTON,
    0x020000 => KEY_RIGHT_BUTTON,
    0x034601 => KEY_BREAK,
    0x040000 => KEY_MIDDLE_BUTTON,
    0x050000 => KEY_XBUTTON1,
    0x060000 => KEY_XBUTTON2,
    0x080E00 => KEY_BACKSPACE,
    0x090F00 => KEY_TAB,
    0x0C4C00 => KEY_NUM_CLEAR,
    0x0D1C00 => KEY_ENTER,
    0x0D1C01 => KEY_NUM_ENTER,
    0x102A00 => KEY_SHIFT,
    0x111D00 => KEY_CTRL,
    0x123800 => KEY_MENU,
    0x134500 => KEY_PAUSE,
    0x134501 => KEY_NUM_LOCK_2,
    0x143A00 => KEY_CAPS_LOCK,
    0x150000 => KEY_KANA,
    0x160000 => KEY_IME_ON,
    0x170000 => KEY_JUNJA,
    0x180000 => KEY_FINAL,
    0x190000 => KEY_HANJA,
    0x1A0000 => KEY_IME_OFF,
    0x1B0100 => KEY_ESC,
    0x1C0000 => KEY_CONVERT,
    0x1D0000 => KEY_NON_CONVERT,
    0x1E0000 => KEY_ACCEPT,
    0x1F0000 => KEY_MODE_CHANGE,
    0x203900 => KEY_SPACE,
    0x214900 => KEY_NUM_PAGE_UP,
    0x214901 => KEY_PAGE_UP,
    0x225100 => KEY_NUM_PAGE_DOWN,
    0x225101 => KEY_PAGE_DOWN,
    0x234F00 => KEY_NUM_END,
    0x234F01 => KEY_END,
    0x244700 => KEY_NUM_HOME,
    0x244701 => KEY_HOME,
    0x254B00 => KEY_NUM_LEFT,
    0x254B01 => KEY_LEFT,
    0x264800 => KEY_NUM_UP,
    0x264801 => KEY_UP,
    0x274D00 => KEY_NUM_RIGHT,
    0x274D01 => KEY_RIGHT,
    0x285000 => KEY_NUM_DOWN,
    0x285001 => KEY_DOWN,
    0x290000 => KEY_SELECT,
    0x2A0000 => KEY_PRINT,
    0x2B0000 => KEY_EXECUTE,
    0x2C3701 => KEY_PRINT_SCREEN,
    0x2C5400 => KEY_SYS_REQ,
    0x2D5200 => KEY_NUM_INSERT,
    0x2D5201 => KEY_INSERT,
    0x2E5300 => KEY_NUM_DELETE,
    0x2E5301 => KEY_DELETE,
    0x2F6300 => KEY_HELP,
    0x300B00 => KEY_0,
    0x310200 => KEY_1,
    0x320300 => KEY_2,
    0x330400 => KEY_3,
    0x340500 => KEY_4,
    0x350600 => KEY_5,
    0x360700 => KEY_6,
    0x370800 => KEY_7,
    0x380900 => KEY_8,
    0x390A00 => KEY_9,
    0x411E00 => KEY_A,
    0x423000 => KEY_B,
    0x432E00 => KEY_C,
    0x442000 => KEY_D,
    0x451200 => KEY_E,
    0x462100 => KEY_F,
    0x472200 => KEY_G,
    0x482300 => KEY_H,
    0x491700 => KEY_I,
    0x4A2400 => KEY_J,
    0x4B2500 => KEY_K,
    0x4C2600 => KEY_L,
    0x4D3200 => KEY_M,
    0x4E3100 => KEY_N,
    0x4F1800 => KEY_O,
    0x501900 => KEY_P,
    0x511000 => KEY_Q,
    0x521300 => KEY_R,
    0x531F00 => KEY_S,
    0x541400 => KEY_T,
    0x551600 => KEY_U,
    0x562F00 => KEY_V,
    0x571100 => KEY_W,
    0x582D00 => KEY_X,
    0x591500 => KEY_Y,
    0x5A2C00 => KEY_Z,
    0x5B5B01 => KEY_LEFT_WIN,
    0x5C5C01 => KEY_RIGHT_WIN,
    0x5D5D01 => KEY_APPLICATION,
    0x5F5F01 => KEY_SLEEP,
    0x605200 => KEY_NUM_0,
    0x614F00 => KEY_NUM_1,
    0x625000 => KEY_NUM_2,
    0x635100 => KEY_NUM_3,
    0x644B00 => KEY_NUM_4,
    0x654C00 => KEY_NUM_5,
    0x664D00 => KEY_NUM_6,
    0x674700 => KEY_NUM_7,
    0x684800 => KEY_NUM_8,
    0x694900 => KEY_NUM_9,
    0x6A3700 => KEY_NUM_MUL,
    0x6B4E00 => KEY_NUM_PLUS,
    0x6C0000 => KEY_SEPARATOR,
    0x6D4A00 => KEY_NUM_MINUS,
    0x6E5300 => KEY_NUM_DOT,
    0x6F3501 => KEY_NUM_DIV,
    0x703B00 => KEY_F1,
    0x713C00 => KEY_F2,
    0x723D00 => KEY_F3,
    0x733E00 => KEY_F4,
    0x743F00 => KEY_F5,
    0x754000 => KEY_F6,
    0x764100 => KEY_F7,
    0x774200 => KEY_F8,
    0x784300 => KEY_F9,
    0x794400 => KEY_F10,
    0x7A5700 => KEY_F11,
    0x7B5800 => KEY_F12,
    0x7C6400 => KEY_F13,
    0x7D6500 => KEY_F14,
    0x7E6600 => KEY_F15,
    0x7F6700 => KEY_F16,
    0x806800 => KEY_F17,
    0x816900 => KEY_F18,
    0x826A00 => KEY_F19,
    0x836B00 => KEY_F20,
    0x846C00 => KEY_F21,
    0x856D00 => KEY_F22,
    0x866E00 => KEY_F23,
    0x877600 => KEY_F24,
    0x904501 => KEY_NUM_LOCK,
    0x914600 => KEY_SCROLL_LOCK,
    0xA02A00 => KEY_LEFT_SHIFT,
    0xA13601 => KEY_RIGHT_SHIFT,
    0xA21D00 => KEY_LEFT_CTRL,
    0xA31D01 => KEY_RIGHT_CTRL,
    0xA43800 => KEY_LEFT_ALT,
    0xA53801 => KEY_RIGHT_ALT,
    0xA66A01 => KEY_BROWSER_BACK,
    0xA76901 => KEY_BROWSER_FORWARD,
    0xA86701 => KEY_BROWSER_REFRESH,
    0xA96801 => KEY_BROWSER_STOP,
    0xAA0001 => KEY_BROWSER_SEARCH,
    0xAA6501 => KEY_FN_BROWSER_SEARCH,
    0xAB6601 => KEY_BROWSER_FAVORITES,
    0xAC0001 => KEY_BROWSER_HOME,
    0xAC3201 => KEY_FN_BROWSER_HOME,
    0xAD0001 => KEY_VOLUME_MUTE,
    0xAD2001 => KEY_FN_VOLUME_MUTE,
    0xAE0001 => KEY_VOLUME_DOWN,
    0xAE2E01 => KEY_FN_VOLUME_DOWN,
    0xAF0001 => KEY_VOLUME_UP,
    0xAF3001 => KEY_FN_VOLUME_UP,
    0xB00001 => KEY_MEDIA_NEXT_TRACK,
    0xB01901 => KEY_FN_MEDIA_NEXT_TRACK,
    0xB10001 => KEY_MEDIA_PREV_TRACK,
    0xB11001 => KEY_FN_MEDIA_PREV_TRACK,
    0xB22401 => KEY_MEDIA_STOP,
    0xB30001 => KEY_MEDIA_PLAY_PAUSE,
    0xB32201 => KEY_FN_MEDIA_PLAY_PAUSE,
    0xB40001 => KEY_LAUNCH_MAIL,
    0xB46C01 => KEY_FN_LAUNCH_MAIL,
    0xB56D01 => KEY_LAUNCH_MEDIA_SELECT,
    0xB60001 => KEY_LAUNCH_APP1,
    0xB66B01 => KEY_FN_LAUNCH_APP1,
    0xB70001 => KEY_LAUNCH_APP2,
    0xB72101 => KEY_FN_LAUNCH_APP2,
    0xBA2700 => KEY_SEMICOLON,
    0xBB0D00 => KEY_EQ,
    0xBC3300 => KEY_COMMA,
    0xBD0C00 => KEY_MINUS,
    0xBE3400 => KEY_DOT,
    0xBF3500 => KEY_SLASH,
    0xC02900 => KEY_BACKTICK,
    0xDB1A00 => KEY_LEFT_BRACKET,
    0xDC2B00 => KEY_BACKSLASH,
    0xDD1B00 => KEY_RIGHT_BRACKET,
    0xDE2800 => KEY_APOSTROPHE,
    0xDF0000 => KEY_OEM_8,
    0xE25600 => KEY_BACKSLASH_2,
    0xE50000 => KEY_PROCESS_KEY,
    0xE70000 => KEY_PACKET,
    0xF10001 => KEY_WHEEL_X,
    0xF30001 => KEY_WHEEL_Y,
    0xF60000 => KEY_ATTN,
    0xF70000 => KEY_CRSEL,
    0xF80000 => KEY_EXSEL,
    0xF95D00 => KEY_EREOF,
    0xFA0000 => KEY_PLAY,
    0xFB6200 => KEY_ZOOM,
    0xFC0000 => KEY_NONAME,
    0xFD0000 => KEY_PA1,
    0xFE0000 => KEY_OEM_CLEAR,
};

pub(crate) static NAME_TO_KEY_MAP: phf::Map<&'static str, Key> = phf_map! {
    "" => KEY_UNASSIGNED,
    "0" => KEY_0,
    "<00>" => KEY_00,
    "1" => KEY_1,
    "2" => KEY_2,
    "3" => KEY_3,
    "4" => KEY_4,
    "5" => KEY_5,
    "6" => KEY_6,
    "7" => KEY_7,
    "8" => KEY_8,
    "9" => KEY_9,
    "A" => KEY_A,
    "ACCEPT" => KEY_ACCEPT,
    "APOSTROPHE" => KEY_APOSTROPHE,
    "APPLICATION" => KEY_APPLICATION,
    "ATTN" => KEY_ATTN,
    "B" => KEY_B,
    "BACKSLASH" => KEY_BACKSLASH,
    "BACKSLASH_2" => KEY_BACKSLASH_2,
    "BACKSPACE" => KEY_BACKSPACE,
    "BACKTICK" => KEY_BACKTICK,
    "BREAK" => KEY_BREAK,
    "BRIGHTNESS" => KEY_BRIGHTNESS,
    "BROWSER_BACK" => KEY_BROWSER_BACK,
    "BROWSER_FAVORITES" => KEY_BROWSER_FAVORITES,
    "BROWSER_FORWARD" => KEY_BROWSER_FORWARD,
    "BROWSER_HOME" => KEY_BROWSER_HOME,
    "BROWSER_REFRESH" => KEY_BROWSER_REFRESH,
    "BROWSER_SEARCH" => KEY_BROWSER_SEARCH,
    "BROWSER_STOP" => KEY_BROWSER_STOP,
    "C" => KEY_C,
    "CAPS_LOCK" => KEY_CAPS_LOCK,
    "COMMA" => KEY_COMMA,
    "CONVERT" => KEY_CONVERT,
    "CRSEL" => KEY_CRSEL,
    "CTRL" => KEY_CTRL,
    "D" => KEY_D,
    "DELETE" => KEY_DELETE,
    "DOT" => KEY_DOT,
    "DOWN" => KEY_DOWN,
    "E" => KEY_E,
    "END" => KEY_END,
    "ENTER" => KEY_ENTER,
    "EQ" => KEY_EQ,
    "EREOF" => KEY_EREOF,
    "ESC" => KEY_ESC,
    "EXECUTE" => KEY_EXECUTE,
    "EXSEL" => KEY_EXSEL,
    "F" => KEY_F,
    "F1" => KEY_F1,
    "F10" => KEY_F10,
    "F11" => KEY_F11,
    "F12" => KEY_F12,
    "F13" => KEY_F13,
    "F14" => KEY_F14,
    "F15" => KEY_F15,
    "F16" => KEY_F16,
    "F17" => KEY_F17,
    "F18" => KEY_F18,
    "F19" => KEY_F19,
    "F2" => KEY_F2,
    "F20" => KEY_F20,
    "F21" => KEY_F21,
    "F22" => KEY_F22,
    "F23" => KEY_F23,
    "F24" => KEY_F24,
    "F3" => KEY_F3,
    "F4" => KEY_F4,
    "F5" => KEY_F5,
    "F6" => KEY_F6,
    "F7" => KEY_F7,
    "F8" => KEY_F8,
    "F9" => KEY_F9,
    "FINAL" => KEY_FINAL,
    "FN_BROWSER_HOME" => KEY_FN_BROWSER_HOME,
    "FN_BROWSER_SEARCH" => KEY_FN_BROWSER_SEARCH,
    "FN_LAUNCH_APP1" => KEY_FN_LAUNCH_APP1,
    "FN_LAUNCH_APP2" => KEY_FN_LAUNCH_APP2,
    "FN_LAUNCH_MAIL" => KEY_FN_LAUNCH_MAIL,
    "FN_MEDIA_NEXT_TRACK" => KEY_FN_MEDIA_NEXT_TRACK,
    "FN_MEDIA_PLAY_PAUSE" => KEY_FN_MEDIA_PLAY_PAUSE,
    "FN_MEDIA_PREV_TRACK" => KEY_FN_MEDIA_PREV_TRACK,
    "FN_VOLUME_DOWN" => KEY_FN_VOLUME_DOWN,
    "FN_VOLUME_MUTE" => KEY_FN_VOLUME_MUTE,
    "FN_VOLUME_UP" => KEY_FN_VOLUME_UP,
    "G" => KEY_G,
    "H" => KEY_H,
    "HANJA" => KEY_HANJA,
    "HELP" => KEY_HELP,
    "HOME" => KEY_HOME,
    "I" => KEY_I,
    "IME_OFF" => KEY_IME_OFF,
    "IME_ON" => KEY_IME_ON,
    "INSERT" => KEY_INSERT,
    "J" => KEY_J,
    "JUNJA" => KEY_JUNJA,
    "K" => KEY_K,
    "KANA" => KEY_KANA,
    "L" => KEY_L,
    "LAUNCH_APP1" => KEY_LAUNCH_APP1,
    "LAUNCH_APP2" => KEY_LAUNCH_APP2,
    "LAUNCH_MAIL" => KEY_LAUNCH_MAIL,
    "LAUNCH_MEDIA_SELECT" => KEY_LAUNCH_MEDIA_SELECT,
    "LEFT" => KEY_LEFT,
    "LEFT_ALT" => KEY_LEFT_ALT,
    "LEFT_BRACKET" => KEY_LEFT_BRACKET,
    "LEFT_BUTTON" => KEY_LEFT_BUTTON,
    "LEFT_CTRL" => KEY_LEFT_CTRL,
    "LEFT_SHIFT" => KEY_LEFT_SHIFT,
    "LEFT_WIN" => KEY_LEFT_WIN,
    "M" => KEY_M,
    "MEDIA_NEXT_TRACK" => KEY_MEDIA_NEXT_TRACK,
    "MEDIA_PLAY_PAUSE" => KEY_MEDIA_PLAY_PAUSE,
    "MEDIA_PREV_TRACK" => KEY_MEDIA_PREV_TRACK,
    "MEDIA_STOP" => KEY_MEDIA_STOP,
    "MENU" => KEY_MENU,
    "MIDDLE_BUTTON" => KEY_MIDDLE_BUTTON,
    "MINUS" => KEY_MINUS,
    "MODE_CHANGE" => KEY_MODE_CHANGE,
    "N" => KEY_N,
    "NONAME" => KEY_NONAME,
    "NON_CONVERT" => KEY_NON_CONVERT,
    "NUM_0" => KEY_NUM_0,
    "NUM_1" => KEY_NUM_1,
    "NUM_2" => KEY_NUM_2,
    "NUM_3" => KEY_NUM_3,
    "NUM_4" => KEY_NUM_4,
    "NUM_5" => KEY_NUM_5,
    "NUM_6" => KEY_NUM_6,
    "NUM_7" => KEY_NUM_7,
    "NUM_8" => KEY_NUM_8,
    "NUM_9" => KEY_NUM_9,
    "NUM_CLEAR" => KEY_NUM_CLEAR,
    "NUM_DELETE" => KEY_NUM_DELETE,
    "NUM_DIV" => KEY_NUM_DIV,
    "NUM_DOT" => KEY_NUM_DOT,
    "NUM_DOWN" => KEY_NUM_DOWN,
    "NUM_END" => KEY_NUM_END,
    "NUM_ENTER" => KEY_NUM_ENTER,
    "NUM_HOME" => KEY_NUM_HOME,
    "NUM_INSERT" => KEY_NUM_INSERT,
    "NUM_LEFT" => KEY_NUM_LEFT,
    "NUM_LOCK" => KEY_NUM_LOCK,
    "NUM_LOCK_2" => KEY_NUM_LOCK_2,
    "NUM_MINUS" => KEY_NUM_MINUS,
    "NUM_MUL" => KEY_NUM_MUL,
    "NUM_PAGE_DOWN" => KEY_NUM_PAGE_DOWN,
    "NUM_PAGE_UP" => KEY_NUM_PAGE_UP,
    "NUM_PLUS" => KEY_NUM_PLUS,
    "NUM_RIGHT" => KEY_NUM_RIGHT,
    "NUM_UP" => KEY_NUM_UP,
    "O" => KEY_O,
    "OEM_8" => KEY_OEM_8,
    "OEM_CLEAR" => KEY_OEM_CLEAR,
    "P" => KEY_P,
    "PA1" => KEY_PA1,
    "PACKET" => KEY_PACKET,
    "PAGE_DOWN" => KEY_PAGE_DOWN,
    "PAGE_UP" => KEY_PAGE_UP,
    "PAUSE" => KEY_PAUSE,
    "PLAY" => KEY_PLAY,
    "PLUS" => KEY_PLUS,
    "PRINT" => KEY_PRINT,
    "PRINT_SCREEN" => KEY_PRINT_SCREEN,
    "PROCESS_KEY" => KEY_PROCESS_KEY,
    "Q" => KEY_Q,
    "R" => KEY_R,
    "RIGHT" => KEY_RIGHT,
    "RIGHT_ALT" => KEY_RIGHT_ALT,
    "RIGHT_BRACKET" => KEY_RIGHT_BRACKET,
    "RIGHT_BUTTON" => KEY_RIGHT_BUTTON,
    "RIGHT_CTRL" => KEY_RIGHT_CTRL,
    "RIGHT_SHIFT" => KEY_RIGHT_SHIFT,
    "RIGHT_SHIFT_2" => KEY_RIGHT_SHIFT_2,
    "RIGHT_WIN" => KEY_RIGHT_WIN,
    "S" => KEY_S,
    "SCROLL_LOCK" => KEY_SCROLL_LOCK,
    "SELECT" => KEY_SELECT,
    "SEMICOLON" => KEY_SEMICOLON,
    "SEPARATOR" => KEY_SEPARATOR,
    "SHIFT" => KEY_SHIFT,
    "SLASH" => KEY_SLASH,
    "SLEEP" => KEY_SLEEP,
    "SPACE" => KEY_SPACE,
    "SYS_REQ" => KEY_SYS_REQ,
    "T" => KEY_T,
    "TAB" => KEY_TAB,
    "U" => KEY_U,
    "UNASSIGNED" => KEY_UNASSIGNED,
    "UP" => KEY_UP,
    "V" => KEY_V,
    "VOLUME_DOWN" => KEY_VOLUME_DOWN,
    "VOLUME_MUTE" => KEY_VOLUME_MUTE,
    "VOLUME_UP" => KEY_VOLUME_UP,
    "W" => KEY_W,
    "WHEEL_X" => KEY_WHEEL_X,
    "WHEEL_Y" => KEY_WHEEL_Y,
    "X" => KEY_X,
    "XBUTTON1" => KEY_XBUTTON1,
    "XBUTTON2" => KEY_XBUTTON2,
    "Y" => KEY_Y,
    "Z" => KEY_Z,
    "ZOOM" => KEY_ZOOM,
    "_" => KEY__,
    "<ESC>" => KEY__ESC,
    "<TAB>" => KEY__TAB	,
};

#[cfg(test)]
mod tests {
    use crate::key::{key_by_name, key_code, Key, CODE_TO_KEY_MAP, NAME_TO_KEY_MAP};
    use crate::sc::ScanCode;
    use crate::utils::test::SerdeWrapper;
    use crate::vk::VirtualKey;
    use std::str::FromStr;

    #[test]
    fn test_key_code() {
        assert_eq!(key_code(0x30, 0x0B, false), 0x300B00);
    }

    #[test]
    fn test_maps() {
        CODE_TO_KEY_MAP.entries().for_each(|(code, key)| {
            assert!(NAME_TO_KEY_MAP.get(key.name).is_some());
            assert_eq!(*code, key_code(key.vk.0, key.sc.0, key.sc.1));
        });
        NAME_TO_KEY_MAP.entries().for_each(|(name, key)| {
            let code = key_code(key.vk.0, key.sc.0, key.sc.1);
            assert!(CODE_TO_KEY_MAP.get(&code).is_some());
            assert_eq!(*name, key.name);
        });
    }

    #[test]
    fn test_key_by_name() {
        assert!(
            NAME_TO_KEY_MAP
                .entries()
                .all(|(name, key)| key_by_name(name).unwrap() == key)
        )
    }

    #[test]
    fn test_key_name() {
        assert!(
            NAME_TO_KEY_MAP
                .entries()
                .all(|(name, key)| key.name == *name)
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
                    vk: VirtualKey(0x0D),
                    sc: ScanCode(0x1C, false),
                }
            )
        );

        assert_eq!(
            "NUM_ENTER",
            format!(
                "{}",
                Key {
                    name: "NUM_ENTER",
                    vk: VirtualKey(0x0D),
                    sc: ScanCode(0x1C, true),
                }
            )
        );
    }

    #[test]
    fn test_key_from_str() {
        assert_eq!(
            Key {
                name: "ENTER",
                vk: VirtualKey(0x0D),
                sc: ScanCode(0x1C, false),
            },
            Key::from_str("ENTER").unwrap()
        );

        assert_eq!(
            Key {
                name: "NUM_ENTER",
                vk: VirtualKey(0x0D),
                sc: ScanCode(0x1C, true),
            },
            Key::from_str("NUM_ENTER").unwrap()
        );

        assert_eq!(
            Key {
                name: "F3",
                vk: VirtualKey(0x72),
                sc: ScanCode(0x3D, false),
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
        let actual: SerdeWrapper<Key> = toml::from_str(&text).unwrap();

        assert_eq!(source.value, &actual.value);

        let source = SerdeWrapper::new(key!("NUM_ENTER"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual: SerdeWrapper<Key> = toml::from_str(&text).unwrap();

        assert_eq!(source.value, &actual.value);
    }

    // #[test]
    // fn generate() {
    //     for (_n, k) in KEYS {
    //         // println!("\"{}\" => KEY_{},", k.name, k);
    //         println!(
    //             "(0x{:02X},0x{:02X},{}) => KEY_{},",
    //             k.vk_code, k.scan_code.0, k.scan_code.1, k
    //         );
    //     }
    // }
}
