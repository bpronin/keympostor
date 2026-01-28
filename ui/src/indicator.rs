use crate::layout::Layout;
use crate::settings::KeyboardLightingSettings;
use log::{debug, error, warn};
use lomen_core::color::ZoneColors;
use lomen_core::light_control::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;
use windows::core::PCWSTR;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::Globalization::GetLocaleInfoW;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};
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
        write!(
            f,
            "[#{:06X}, #{:06X}, #{:06X}, #{:06X}]",
            self.right, self.center, self.left, self.game
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

fn play_sound(filename: &str) {
    unsafe {
        let w_filename: Vec<u16> = filename.encode_utf16().chain(std::iter::once(0)).collect();
        let result = PlaySoundW(
            PCWSTR(w_filename.as_ptr()),
            None,
            SND_FILENAME | SND_NODEFAULT | SND_ASYNC,
        );

        if !result.as_bool() {
            warn!(
                "Unable to play sound: `{}` : {:?}",
                filename,
                GetLastError()
            );
        }
    }
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
        if let Some(sounds) = &l.sound {
            let locale = get_keyboard_locale(keyboard_layout);
            if let Some(sound) = sounds.get(&locale).or_else(|| sounds.get("default")) {
                play_sound(sound);

                debug!("Playing layout sound: {sound} for locale: {locale}");
            }
        };
    }
}

pub(crate) fn get_current_keyboard_layout() -> HKL {
    unsafe { GetKeyboardLayout(GetWindowThreadProcessId(GetForegroundWindow(), None)) }
}

pub(crate) fn on_layout_changed(transform_layout: Option<&Layout>, keyboard_layout: HKL) {
    play_layout_sound(transform_layout, keyboard_layout);
    set_layout_lighting(transform_layout, keyboard_layout);
}
