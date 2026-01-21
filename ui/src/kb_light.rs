use crate::settings::KeyboardLightingSettings;
use libloading::os::windows::{Library, LOAD_WITH_ALTERED_SEARCH_PATH};
use log::debug;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use windows::Win32::Globalization::GetLocaleInfoW;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, HKL};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub(crate) static KEYBOARD_LIGHTING_SETTINGS: LazyLock<KeyboardLightingSettings> =
    LazyLock::new(|| KeyboardLightingSettings::load_default());

#[repr(C)]
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(into = "Vec<String>", from = "Vec<String>")]
pub(crate) struct KeyboardZoneColors {
    pub(crate) right: u64,
    pub(crate) center: u64,
    pub(crate) left: u64,
    pub(crate) game: u64,
}

impl Into<Vec<String>> for KeyboardZoneColors {
    fn into(self) -> Vec<String> {
        let format = |v: u64| format!("#{:06X}", v);

        vec![
            format(self.right),
            format(self.center),
            format(self.left),
            format(self.game),
        ]
    }
}

impl From<Vec<String>> for KeyboardZoneColors {
    fn from(value: Vec<String>) -> Self {
        let parse = |s: &str| {
            u64::from_str_radix(s.trim_start_matches("#"), 16)
                .map_err(|e| format!("Hex parse error: {}", e))
        };

        Self {
            right: parse(&value[0]).unwrap(),
            center: parse(&value[1]).unwrap(),
            left: parse(&value[2]).unwrap(),
            game: parse(&value[3]).unwrap(),
        }
    }
}

pub(crate) fn get_current_keyboard_layout() -> HKL {
    unsafe { GetKeyboardLayout(GetWindowThreadProcessId(GetForegroundWindow(), None)) }
}

pub(crate) fn update_keyboard_lighting(layout_name: Option<&str>, keyboard_layout: HKL) {
    update_keyboard_lighting2(layout_name.unwrap_or("none"), keyboard_layout);
}

fn update_keyboard_lighting2(layout_name: &str, keyboard_layout: HKL) {
    if let Some(layout_settings) = KEYBOARD_LIGHTING_SETTINGS.layouts.get(layout_name) {
        let lang = get_keyboard_locale(keyboard_layout);
        if let Some(land_settings) = &layout_settings.0.get(&lang) {
            debug!("Updating keyboard colors for: {layout_name}, lang: {lang}");
            set_colors(land_settings);
        }
    }
}

fn get_locale_info(lang_id: u32, lc_type: u32) -> String {
    unsafe {
        let buffer_size = GetLocaleInfoW(lang_id, lc_type, None) as usize;
        let mut buffer = vec![0u16; buffer_size];
        GetLocaleInfoW(lang_id, lc_type, Some(&mut buffer));
        buffer.set_len(buffer_size - 1); /* remove null terminator */
        String::from_utf16_lossy(&buffer)
    }
}

fn get_keyboard_locale(keyboard_layout: HKL) -> String {
    let lang_id = (keyboard_layout.0 as u32) & 0xFFFF;
    format!(
        "{}_{}",
        get_locale_info(lang_id, 0x59),
        get_locale_info(lang_id, 0x5A)
    )
    .to_lowercase()
}

static DLL: LazyLock<Option<Library>> = LazyLock::new(|| unsafe {
    Library::load_with_flags("lomen.dll", LOAD_WITH_ALTERED_SEARCH_PATH)
        .inspect_err(|e| debug!("Failed to load lomen.dll: {e}. Keyboard lighting disabled."))
        .ok()
});

type FnSetColors = extern "stdcall" fn(*const KeyboardZoneColors);

fn set_colors(colors: &KeyboardZoneColors) {
    if let Some(lib) = DLL.as_ref() {
        unsafe {
            let fun = lib.get::<FnSetColors>(b"set_colors\0").unwrap();
            fun(colors);
        }
    }
}
