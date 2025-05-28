pub mod key;
pub mod key_action;
pub mod key_const;
pub mod key_event;
pub mod key_hook;
pub mod key_modifiers;
pub mod key_trigger;
mod parse;
mod serialize;
mod transform_map;
pub mod transform_rules;

/*
#[cfg(test)]
mod tests {
use log::LevelFilter;
use simple_logger::SimpleLogger;


  *** Key codes generation stuff ***

#[test]
#[ignore]
fn generate_keys() {
    let mut id = 0;
    for vk_code in 0..MAX_VK_CODE {
        let vk_key = VirtualKey::from_code(vk_code as u8).unwrap();
        let vk_name = vk_key.name;
        let name = if let Some(name) = vk_name.strip_prefix("VK_") {
            name
        } else {
            vk_name
        };

        let vk_opt = format!("Some(&VIRTUAL_KEYS[{}])", vk_code);

        let sc_opt = if let Ok(sc_key) = vk_key.to_scan_code() {
            format!(
                "Some(&SCAN_CODES[{}][{}])",
                sc_key.value, sc_key.is_extended as u8
            )
        } else {
            "None".to_string()
        };

        println!("new_key!({}, \"{}\", {}, {}),", id, name, vk_opt, sc_opt);

        id = id + 1;
    }

    for ext_sc in [false, true] {
        for sc_code in 0..MAX_SCAN_CODE {
            let sc_key = ScanCode::from_code(sc_code as u8, ext_sc).unwrap();
            let sc_name = sc_key.name;
            let name = if let Some(name) = sc_name.strip_prefix("SC_") {
                name
            } else {
                sc_name
            };

            if sc_key.to_virtual_key().is_ok() {
                continue;
            }

            let sc_opt = format!(
                "Some(&SCAN_CODES[{}][{}])",
                sc_key.value, sc_key.is_extended as u8
            );

            println!("new_key!({}, \"{}\", None, {}),", id, name, sc_opt);

            id = id + 1;
        }
    }
}

#[test]
#[ignore]
fn generate_keys_names() {
    let mut map = BTreeMap::<&str, (&VirtualKey, &ScanCode)>::new();
    for vk_code in 0..MAX_VK_CODE {
        let vk = VirtualKey::from_code(vk_code as u8).unwrap();

        let sc = vk
            .to_scan_code()
            .unwrap_or(ScanCode::from_code(0, false).unwrap());

        let name = if let Some(name) = vk.name.strip_prefix("VK_") {
            name
        } else if let Some(name) = sc.name.strip_prefix("SC_") {
            name
        } else {
            continue;
        };

        if !map.contains_key(name) {
            map.insert(name, (vk, sc));
        } else {
            eprintln!("Duplicate key: {}", name);
        }
    }

    for ext_sc in [false, true] {
        for sc_code in 0..MAX_SCAN_CODE {
            let sc = ScanCode::from_code(sc_code as u8, ext_sc).unwrap();

            let vk = sc
                .to_virtual_key()
                .unwrap_or(VirtualKey::from_code(0).unwrap());

            let name = if let Some(name) = sc.name.strip_prefix("SC_") {
                name
            } else if let Some(name) = vk.name.strip_prefix("VK_") {
                name
            } else {
                continue;
            };

            if !map.contains_key(name) {
                map.insert(name, (vk, sc));
            } else {
                eprintln!("Duplicate key: {}", name);
            }
        }
    }

    map.iter().for_each(|(name, v)| {
        println!(
            "new_key!(\"{}\", 0x{:02X}, 0x{:02X}, {}),",
            name, v.0.value, v.1.value, v.1.is_extended
        );
    })
}

use crate::keys::{ScanCode, VirtualKey, MAX_SCAN_CODE, MAX_VK_CODE};

pub(crate) static SCANCODE_NAMES: [[&str; 2]; MAX_SCAN_CODE] = [
    ["UNASSIGNED", "UNASSIGNED"],
    ["SC_ESC", "SC_"],
    ["SC_1", "SC_1"],
    ["SC_2", "SC_2"],
    ["SC_3", "SC_3"],
    ["SC_4", "SC_4"],
    ["SC_5", "SC_5"],
    ["SC_6", "SC_6"],
    ["SC_7", "SC_7"],
    ["SC_8", "SC_8"],
    ["SC_9", "SC_9"],
    ["SC_0", "SC_0"],
    ["SC_MINUS", "SC_MINUS"],
    ["SC_EQ", "SC_EQ"],
    ["SC_BACKSPACE", "SC"],
    ["SC_TAB", "SC_	"],
    ["SC_Q", "SC_Q"],
    ["SC_W", "SC_W"],
    ["SC_E", "SC_E"],
    ["SC_R", "SC_R"],
    ["SC_T", "SC_T"],
    ["SC_Y", "SC_Y"],
    ["SC_U", "SC_U"],
    ["SC_I", "SC_I"],
    ["SC_O", "SC_O"],
    ["SC_P", "SC_P"],
    ["SC_L_BRACKET", "SC_L_BRACKET"],
    ["SC_R_BRACKET", "SC_R_BRACKET"],
    ["SC_ENTER", "SC_NUM_ENTER"],
    ["SC_CTRL", "SC_RIGHT_CTRL"],
    ["SC_A", "SC_A"],
    ["SC_S", "SC_S"],
    ["SC_D", "SC_D"],
    ["SC_F", "SC_F"],
    ["SC_G", "SC_G"],
    ["SC_H", "SC_H"],
    ["SC_J", "SC_J"],
    ["SC_K", "SC_K"],
    ["SC_L", "SC_L"],
    ["SC_SEMICOLON", "SC_SEMICOLON"],
    ["SC_APOSTROPHE", "SC_APOSTROPHE"],
    ["SC_BACKTICK", "SC_BACKTICK"],
    ["SC_SHIFT", "UNASSIGNED"],
    ["SC_BACKSLASH", "SC_BACKSLASH"],
    ["SC_Z", "SC_Z"],
    ["SC_X", "SC_X"],
    ["SC_C", "SC_C"],
    ["SC_V", "SC_V"],
    ["SC_B", "SC_B"],
    ["SC_N", "SC_N"],
    ["SC_M", "SC_M"],
    ["SC_COMMA", "SC_COMMA"],
    ["SC_DOT", "SC_DOT"],
    ["SC_SLASH", "SC_NUM_SLASH"],
    ["SC_RIGHT_SHIFT", "UNASSIGNED"],
    ["SC_NUM_MUL", "SC_PRNT_SCRN"],
    ["SC_ALT", "SC_RIGHT_ALT"],
    ["SC_SPACE", "SC__"],
    ["SC_CAPS_LOCK", "UNASSIGNED"],
    ["SC_F1", "UNASSIGNED"],
    ["SC_F2", "UNASSIGNED"],
    ["SC_F3", "UNASSIGNED"],
    ["SC_F4", "UNASSIGNED"],
    ["SC_F5", "UNASSIGNED"],
    ["SC_F6", "UNASSIGNED"],
    ["SC_F7", "UNASSIGNED"],
    ["SC_F8", "UNASSIGNED"],
    ["SC_F9", "UNASSIGNED"],
    ["SC_F10", "UNASSIGNED"],
    ["SC_PAUSE", "SC_NUM_LOCK"],
    ["SC_SCROLL_LOCK", "SC_BREAK"],
    ["SC_NUM_7", "SC_HOME"],
    ["SC_NUM_8", "SC_UP"],
    ["SC_NUM_9", "SC_PAGE_UP"],
    ["SC_NUM_MINUS", "SC_MINUS"],
    ["SC_NUM_4", "SC_LEFT"],
    ["SC_NUM_5", "UNASSIGNED"],
    ["SC_NUM_6", "SC_RIGHT"],
    ["SC_NUM_PLUS", "SC_PLUS"],
    ["SC_NUM_1", "SC_END"],
    ["SC_NUM_2", "SC_DOWN"],
    ["SC_NUM_3", "SC_PAGE_DOWN"],
    ["SC_NUM_0", "SC_INSERT"],
    ["SC_NUM_DEL", "SC_DELETE"],
    ["SC_SYS_REQ", "SC_<00>"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["SC_BACKSLASH", "SC_HELP"],
    ["SC_F11", "UNASSIGNED"],
    ["SC_F12", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "SC_LEFT_WINDOWS"],
    ["UNASSIGNED", "SC_RIGHT_WINDOWS"],
    ["UNASSIGNED", "SC_APPLICATION"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["UNASSIGNED", "UNASSIGNED"],
    ["SC_F13", "SC_	"],
    ["SC_F14", "UNASSIGNED"],
    ["SC_F15", "UNASSIGNED"],
    ["SC_F16", "UNASSIGNED"],
    ["SC_F17", "UNASSIGNED"],
    ["SC_F18", "UNASSIGNED"],
    ["SC_F19", "UNASSIGNED"],
    ["SC_F20", "UNASSIGNED"],
    ["SC_F21", "UNASSIGNED"],
    ["SC_F22", "UNASSIGNED"],
    ["SC_F23", "UNASSIGNED"],
    ["SC_F24", "UNASSIGNED"],
];

pub(crate) static VIRTUAL_KEY_NAMES: [&str; MAX_VK_CODE] = [
    "UNASSIGNED",
    "VK_LBUTTON",
    "VK_RBUTTON",
    "VK_CANCEL",
    "VK_MBUTTON",
    "VK_XBUTTON1",
    "VK_XBUTTON2",
    "UNASSIGNED",
    "VK_BACK",
    "VK_TAB",
    "UNASSIGNED",
    "UNASSIGNED",
    "VK_CLEAR",
    "VK_RETURN",
    "UNASSIGNED",
    "UNASSIGNED",
    "VK_SHIFT",
    "VK_CONTROL",
    "VK_MENU",
    "VK_PAUSE",
    "VK_CAPITAL",
    "VK_KANA",
    "VK_IME_ON",
    "VK_JUNJA",
    "VK_FINAL",
    "VK_HANJA",
    "VK_IME_OFF",
    "VK_ESCAPE",
    "VK_CONVERT",
    "VK_NONCONVERT",
    "VK_ACCEPT",
    "VK_MODECHANGE",
    "VK_SPACE",
    "VK_PRIOR",
    "VK_NEXT",
    "VK_END",
    "VK_HOME",
    "VK_LEFT",
    "VK_UP",
    "VK_RIGHT",
    "VK_DOWN",
    "VK_SELECT",
    "VK_PRINT",
    "VK_EXECUTE",
    "VK_SNAPSHOT",
    "VK_INSERT",
    "VK_DELETE",
    "VK_HELP",
    "VK_0",
    "VK_1",
    "VK_2",
    "VK_3",
    "VK_4",
    "VK_5",
    "VK_6",
    "VK_7",
    "VK_8",
    "VK_9",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "VK_A",
    "VK_B",
    "VK_C",
    "VK_D",
    "VK_E",
    "VK_F",
    "VK_G",
    "VK_H",
    "VK_I",
    "VK_J",
    "VK_K",
    "VK_L",
    "VK_M",
    "VK_N",
    "VK_O",
    "VK_P",
    "VK_Q",
    "VK_R",
    "VK_S",
    "VK_T",
    "VK_U",
    "VK_V",
    "VK_W",
    "VK_X",
    "VK_Y",
    "VK_Z",
    "VK_LWIN",
    "VK_RWIN",
    "VK_APPS",
    "UNASSIGNED",
    "VK_SLEEP",
    "VK_NUMPAD0",
    "VK_NUMPAD1",
    "VK_NUMPAD2",
    "VK_NUMPAD3",
    "VK_NUMPAD4",
    "VK_NUMPAD5",
    "VK_NUMPAD6",
    "VK_NUMPAD7",
    "VK_NUMPAD8",
    "VK_NUMPAD9",
    "VK_MULTIPLY",
    "VK_ADD",
    "VK_SEPARATOR",
    "VK_SUBTRACT",
    "VK_DECIMAL",
    "VK_DIVIDE",
    "VK_F1",
    "VK_F2",
    "VK_F3",
    "VK_F4",
    "VK_F5",
    "VK_F6",
    "VK_F7",
    "VK_F8",
    "VK_F9",
    "VK_F10",
    "VK_F11",
    "VK_F12",
    "VK_F13",
    "VK_F14",
    "VK_F15",
    "VK_F16",
    "VK_F17",
    "VK_F18",
    "VK_F19",
    "VK_F20",
    "VK_F21",
    "VK_F22",
    "VK_F23",
    "VK_F24",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "VK_NUMLOCK",
    "VK_SCROLL",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "VK_LSHIFT",
    "VK_RSHIFT",
    "VK_LCONTROL",
    "VK_RCONTROL",
    "VK_LMENU",
    "VK_RMENU",
    "VK_BROWSER_BACK",
    "VK_BROWSER_FORWARD",
    "VK_BROWSER_REFRESH",
    "VK_BROWSER_STOP",
    "VK_BROWSER_SEARCH",
    "VK_BROWSER_FAVORITES",
    "VK_BROWSER_HOME",
    "VK_VOLUME_MUTE",
    "VK_VOLUME_DOWN",
    "VK_VOLUME_UP",
    "VK_MEDIA_NEXT_TRACK",
    "VK_MEDIA_PREV_TRACK",
    "VK_MEDIA_STOP",
    "VK_MEDIA_PLAY_PAUSE",
    "VK_LAUNCH_MAIL",
    "VK_LAUNCH_MEDIA_SELECT",
    "VK_LAUNCH_APP1",
    "VK_LAUNCH_APP2",
    "UNASSIGNED",
    "UNASSIGNED",
    "VK_OEM_1",
    "VK_OEM_PLUS",
    "VK_OEM_COMMA",
    "VK_OEM_MINUS",
    "VK_OEM_PERIOD",
    "VK_OEM_2",
    "VK_OEM_3",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "VK_OEM_4",
    "VK_OEM_5",
    "VK_OEM_6",
    "VK_OEM_7",
    "VK_OEM_8",
    "UNASSIGNED",
    "UNASSIGNED",
    "VK_OEM_102",
    "UNASSIGNED",
    "UNASSIGNED",
    "VK_PROCESSKEY",
    "UNASSIGNED",
    "VK_PACKET",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "UNASSIGNED",
    "VK_ATTN",
    "VK_CRSEL",
    "VK_EXSEL",
    "VK_EREOF",
    "VK_PLAY",
    "VK_ZOOM",
    "VK_NONAME",
    "VK_PA1",
    "VK_OEM_CLEAR",
    "VK__none_",
];

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
        + &key_name
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
}

fn find_scancode_name(scancode: u8, extended: bool) -> String {
    if let Some(key_name) = find_key_name(scancode, extended) {
        fmt_scancode_name(&key_name)
    } else {
        "UNASSIGNED".to_string()
    }
}

fn find_vk_code(scancode: u8, extended: bool) -> VirtualKey {
    let ext_code = if extended {
        scancode as u32 | 0xE0 << 8
    } else {
        scancode as u32
    };
    let vk_code = unsafe { MapVirtualKeyW(ext_code, MAPVK_VSC_TO_VK_EX) };
    if vk_code > 0 {
        VirtualKey {
            value: vk_code as u8,
            name: VIRTUAL_KEY_NAMES[vk_code as usize],
        }
    } else {
        VirtualKey {
            value: 0,
            name: "UNASSIGNED",
        }
    }
}

fn find_scancode(vk_code: u32) -> ScanCode {
    let ext_code = unsafe { MapVirtualKeyW(vk_code, MAPVK_VK_TO_VSC_EX) };
    if ext_code > 0 {
        let code = ext_code as u8;
        let extended = ext_code & 0xE000 != 0;
        ScanCode {
            value: code,
            is_extended: extended,
            name: &find_scancode_name(code, extended),
        }
    } else {
        ScanCode {
            value: 0,
            is_extended: false,
            name: "UNASSIGNED",
        }
    }
}

#[test]
#[ignore]
fn generate_sc_names() {
    for scancode in 0..0xFF {
        println!(
            "[\"{}\", \"{}\"],",
            find_scancode_name(scancode, false),
            find_scancode_name(scancode, true)
        );
    }
}

#[test]
#[ignore]
fn generate_vk_array() {
    for vk_code in 0..MAX_VK_CODE {
        let vk = VirtualKey {
            value: vk_code as u8,
            name: VIRTUAL_KEY_NAMES[vk_code],
        };
        println!("virtual_key!(0x{:02X}, \"{}\"),", vk.value, vk.name, );
    }
}

#[test]
#[ignore]
fn generate_sc_array() {
    for scancode in 0..MAX_SCAN_CODE {
        let sc = ScanCode {
            value: scancode as u8,
            is_extended: false,
            name: SCANCODE_NAMES[scancode][false as usize],
        };

        let ext_sc = ScanCode {
            value: scancode as u8,
            is_extended: true,
            name: SCANCODE_NAMES[scancode][true as usize],
        };

        println!(
            "[scancode!(0x{:02X}, \"{}\"), ext_scancode!(0x{:02X}, \"{}\")],",
            sc.value, sc.name, ext_sc.value, ext_sc.name,
        );
    }
}

*/
