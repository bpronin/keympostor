pub mod action;
pub mod consts;
pub mod error;
pub mod event;
pub mod hook;
pub mod key;
pub mod modifiers;
mod parse;
pub mod rules;
mod serialize;
mod transform;
pub mod trigger;

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
