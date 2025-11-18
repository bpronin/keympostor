use hook::KEY_HOOK;
use windows::Win32::Foundation::HWND;
use log::debug;
use crate::keyboard::rules::KeyTransformRules;
use crate::keyboard::transform::KeyTransformMap;

pub mod action;
pub mod consts;
pub mod error;
pub mod event;
pub mod hook;
pub mod key;
pub mod modifiers;
pub mod rules;
mod transform;
pub mod trigger;

#[derive(Debug, Default)]
pub struct KeyboardHook;

impl KeyboardHook {
    pub fn init(&self, owner: Option<HWND>) {
        unsafe {
            KEY_HOOK.owner = owner;
        }
    }

    pub fn apply_rules(&self, rules: &KeyTransformRules) {
        unsafe {
            KEY_HOOK.transform_map = Some(KeyTransformMap::new(rules));
        }
    }

    pub fn is_enabled(&self) -> bool {
        match unsafe { KEY_HOOK.handle } {
            Some(_) => true,
            None => false,
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        if enabled {
            hook::install_key_hook()
        } else {
            hook::uninstall_key_hook()
        }
    }

    pub fn is_notify_enabled(&self) -> bool {
        unsafe { KEY_HOOK.is_notify_enabled }
    }

    pub fn set_notify_enabled(&self, enabled: bool) {
        unsafe { KEY_HOOK.is_notify_enabled = enabled }

        if enabled {
            debug!("Keyboard hook notifications enabled");
        } else {
            debug!("Keyboard hook notifications disabled");
        }
    }
}