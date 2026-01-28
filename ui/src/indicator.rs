use crate::layout::Layout;
use crate::r_play_snd;
use crate::res::res_ids::IDR_SWITCH_LAYOUT;
use crate::res::RESOURCES;
use crate::settings::KeyboardLightingSettings;
use log::{debug, error};
use lomen_core::color::ZoneColors;
use lomen_core::light_control::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;
use std::thread;
use windows::Win32::Globalization::GetLocaleInfoW;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, HKL};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub(crate) static KEYBOARD_LIGHTING_SETTINGS: LazyLock<KeyboardLightingSettings> =
    LazyLock::new(|| KeyboardLightingSettings::load_default());

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(into = "Vec<String>", from = "Vec<String>")]
pub(crate) struct KeyboardZoneColors {
    pub(crate) right: u64,
    pub(crate) center: u64,
    pub(crate) left: u64,
    pub(crate) game: u64,
}

impl Display for KeyboardZoneColors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(
            &format!(
                "[#{:0X}, #{:0X}, #{:0X}, #{:0X}]",
                self.right, self.center, self.left, self.game
            ),
            f,
        )
    }
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

fn get_keyboard_locale(keyboard_layout: HKL) -> String {
    let lang_id = (keyboard_layout.0 as u32) & 0xFFFF;
    format!(
        "{}_{}",
        get_locale_info(lang_id, 0x59),
        get_locale_info(lang_id, 0x5A)
    )
    .to_lowercase()
}

fn set_layout_lighting(transform_layout: Option<&Layout>, keyboard_layout: HKL) {
    let name = match transform_layout {
        None => "none",
        Some(l) => l.name.as_str(),
    };
    
    if let Some(layout_settings) = KEYBOARD_LIGHTING_SETTINGS.layouts.get(name) {
        let lang = get_keyboard_locale(keyboard_layout);
        if let Some(colors) = layout_settings.0.get(&lang) {
            debug!("Updating keyboard colors for: {name}, lang: {lang}, colors : {colors}");

            set_colors(&ZoneColors::from([
                colors.right,
                colors.center,
                colors.left,
                colors.game,
            ]))
            .unwrap_or_else(|e| error!("Failed to set keyboard colors: {e}"));
        }
    }
}

fn play_layout_sound(transform_layout: Option<&Layout>, keyboard_layout: HKL) {
    if let Some(l) = transform_layout {
        debug!(
            "Playing layout sound: {:?} {:?}",
            l, keyboard_layout
        );
    }
}

pub(crate) fn get_current_keyboard_layout() -> HKL {
    unsafe { GetKeyboardLayout(GetWindowThreadProcessId(GetForegroundWindow(), None)) }
}

pub(crate) fn on_layout_changed(transform_layout: Option<&Layout>, keyboard_layout: HKL) {
    play_layout_sound(transform_layout, keyboard_layout);
    set_layout_lighting(transform_layout, keyboard_layout);
}
