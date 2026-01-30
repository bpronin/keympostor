use crate::kb_watch;
use crate::layout::Layout;
use crate::util::{get_keyboard_locale, play_sound};
use log::{debug, error, warn};
use lomen_core::color::LightingColors;
use lomen_core::light_control::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::OnceLock;
use windows::core::PCWSTR;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};
use windows::Win32::UI::Input::KeyboardAndMouse::HKL;

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

pub(crate) fn notify_layout_changed(transform_layout: Option<&Layout>, keyboard_layout: HKL) {
    play_layout_sound(transform_layout, keyboard_layout);
    set_layout_lighting(transform_layout, keyboard_layout);
}
