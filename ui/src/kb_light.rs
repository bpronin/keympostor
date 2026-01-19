use crate::settings::KeyboardLightingSettings;
use crate::layout::Layout;
use libloading::os::windows::{Library, LOAD_WITH_ALTERED_SEARCH_PATH};
use log::debug;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use windows::Win32::Globalization::GetLocaleInfoW;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

static SETTINGS: LazyLock<KeyboardLightingSettings> =
    LazyLock::new(|| KeyboardLightingSettings::load());

#[derive(Default)]
pub struct KeyboardLightingControl {}

impl KeyboardLightingControl {
    pub(crate) fn update_colors(&self, layout: &Option<&Layout>) {
        let layout_name = match layout {
            None => "none",
            Some(l) => &l.name,
        };

        if let Some(layout_settings) = SETTINGS.layouts.get(layout_name) {
            let lang = get_current_keyboard_locale();
            if let Some(land_settings) = &layout_settings.0.get(&lang) {
                debug!("Updating keyboard colors for: {layout_name}, lang: {lang}");
                set_colors(land_settings);
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(into = "Vec<String>", from = "Vec<String>")]
pub struct KeyboardZoneColors {
    pub right: u64,
    pub center: u64,
    pub left: u64,
    pub game: u64,
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

fn get_locale_info(lang_id: u32, lc_type: u32) -> String {
    unsafe {
        let buffer_size = GetLocaleInfoW(lang_id, lc_type, None) as usize;
        let mut buffer = vec![0u16; buffer_size];
        GetLocaleInfoW(lang_id, lc_type, Some(&mut buffer));
        buffer.set_len(buffer_size - 1); /* remove null terminator */
        String::from_utf16_lossy(&buffer)
    }
}

fn get_current_keyboard_locale() -> String {
    unsafe {
        let hkl = GetKeyboardLayout(GetWindowThreadProcessId(GetForegroundWindow(), None));
        let lang_id = (hkl.0 as u32) & 0xFFFF;

        format!(
            "{}_{}",
            get_locale_info(lang_id, 0x59),
            get_locale_info(lang_id, 0x5A)
        )
        .to_lowercase()
    }
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
