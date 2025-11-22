use crate::keyboard::hook::{HOOK, install_mouse_hook, uninstall_mouse_hook};
use crate::keyboard::rules::KeyTransformRules;
use crate::keyboard::transform::KeyTransformMap;
use hook::{install_key_hook, uninstall_key_hook};
use log::debug;
use std::cell::RefCell;
use windows::Win32::Foundation::HWND;

pub mod action;
pub mod error;
pub mod event;
pub mod hook;
mod input;
pub mod key;
pub mod modifiers;
pub mod rules;
pub mod sc;
mod transform;
pub mod transition;
pub mod trigger;
pub mod vk;

#[derive(Debug, Default)]
pub struct KeyboardHook {
    is_enabled: RefCell<bool>,
}

impl KeyboardHook {
    pub fn init(&self, owner: Option<HWND>) {
        unsafe {
            HOOK.owner = owner;
        }
    }

    pub fn apply_rules(&self, rules: &KeyTransformRules) {
        unsafe {
            HOOK.transform_map = Some(KeyTransformMap::new(rules));
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled.borrow().to_owned()
    }

    pub fn set_enabled(&self, enabled: bool) {
        if enabled {
            install_key_hook();
            install_mouse_hook();
        } else {
            uninstall_key_hook();
            uninstall_mouse_hook();
        }
        self.is_enabled.replace(enabled);
    }

    pub fn is_notify_enabled(&self) -> bool {
        unsafe { HOOK.is_notify_enabled }
    }

    pub fn set_notify_enabled(&self, enabled: bool) {
        unsafe { HOOK.is_notify_enabled = enabled }

        if enabled {
            debug!("Hooks notifications enabled");
        } else {
            debug!("Hooks notifications disabled");
        }
    }
}
