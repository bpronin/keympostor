use crate::kb_watch::KeyboardLayoutState;
use crate::layout::KeyTransformLayout;
use crate::util::play_sound;
use log::{debug, error};
use lomen_core::color::LightingColors;
use lomen_core::light_control::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::OnceLock;

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

fn set_layout_keyboard_lighting(
    transform_layout: Option<&KeyTransformLayout>,
    keyboard_state: &KeyboardLayoutState,
) {
    let default_colors = NO_LAYOUT_LIGHTING_COLORS.get_or_init(|| get_colors().ok());

    if let Some(layout) = transform_layout {
        if let Some(layout_settings) = layout.keyboard_lighting.as_ref() {
            let locks = &keyboard_state.locks();
            if let Some(locks_settings) = layout_settings
                .get(locks)
                .or_else(|| layout_settings.get("default"))
            {
                let locale = &keyboard_state.locale();
                if let Some(colors) = locks_settings
                    .get(locale)
                    .or_else(|| locks_settings.get("default"))
                {
                    debug!(
                        "Set keyboard lighting for layout: `{}`, locks: `{}`, locale: `{}``",
                        layout.name, locks, locale
                    );

                    set_colors(&colors.0)
                        .unwrap_or_else(|e| error!("Failed to set keyboard lighting: {e}"));
                }
            }
        }
    } else {
        if let Some(colors) = default_colors {
            debug!("Set default keyboard lighting");

            set_colors(colors)
                .unwrap_or_else(|e| error!("Failed to set default keyboard colors: {e}"));
        }
    }
}

fn play_layout_sound(
    transform_layout: Option<&KeyTransformLayout>,
    keyboard_state: &KeyboardLayoutState,
) {
    if let Some(layout) = transform_layout {
        if let Some(layout_settings) = layout.sound.as_ref() {
            let locks = &keyboard_state.locks();
            if let Some(locks_settings) = layout_settings
                .get(locks)
                .or_else(|| layout_settings.get("default"))
            {
                let locale = &keyboard_state.locale();
                if let Some(sound) = locks_settings
                    .get(locale)
                    .or_else(|| locks_settings.get("default"))
                {
                    debug!(
                        "Playing sound for layout: `{}`, locks: `{}`, locale: `{}``",
                        layout.name, locks, locale
                    );
                    play_sound(sound);
                }
            }
        }
    }
}

pub(crate) fn notify_layout_changed(
    layout: Option<&KeyTransformLayout>,
    keyboard_state: &KeyboardLayoutState,
) {
    play_layout_sound(layout, keyboard_state);
    set_layout_keyboard_lighting(layout, keyboard_state);
}
