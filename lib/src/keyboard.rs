use std::cell::RefCell;
use windows::Win32::Foundation::HWND;
use log::debug;
use hook::{install_key_hook, uninstall_key_hook};
use crate::keyboard::hook::{install_mouse_hook, uninstall_mouse_hook, HOOK};
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
pub mod code;

#[derive(Debug, Default)]
pub struct KeyboardHook{
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