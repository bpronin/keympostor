use crate::layout::Layout;
use log::{debug, error, warn};
use lomen_core::color::LightingColors;
use lomen_core::light_control::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::OnceLock;
use windows::core::PCWSTR;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::Globalization::GetLocaleInfoW;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, HKL};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

static NO_LAYOUT_LIGHTING_COLORS: OnceLock<Option<LightingColors>> = OnceLock::new();

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(into = "Vec<String>", from = "Vec<String>")]
pub(crate) struct SerdeLightingColors(LightingColors);

impl Display for SerdeLightingColors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Into<Vec<String>> for SerdeLightingColors {
    fn into(self) -> Vec<String> {
        self.0.into()
    }
}

impl From<Vec<String>> for SerdeLightingColors {
    fn from(value: Vec<String>) -> Self {
        Self(LightingColors::from(value))
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
    let default_colors = NO_LAYOUT_LIGHTING_COLORS.get_or_init(|| get_colors().ok());

    if let Some(layout) = transform_layout {
        if let Some(lighting) = layout.keyboard_lighting.as_ref() {
            let locale = get_keyboard_locale(keyboard_layout);
            if let Some(colors) = lighting.get(&locale) {
                debug!(
                    "Set keyboard colors for layout: `{}`, lang: `{}`, colors: {}",
                    layout.name, locale, colors
                );

                set_colors(&colors.0)
                    .unwrap_or_else(|e| error!("Failed to set keyboard colors: {e}"));
            }
        }
    } else {
        if let Some(colors) = default_colors {
            debug!("Set default keyboard colors: {}", colors);

            set_colors(colors)
                .unwrap_or_else(|e| error!("Failed to set default keyboard colors: {e}"));
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
