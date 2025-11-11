use crate::keyboard::event::KeyEvent;
use crate::keyboard::hook::KEY_HOOK;
use crate::keyboard::rules::KeyTransformRules;

pub mod action;
pub mod consts;
pub mod error;
pub mod event;
pub mod hook;
pub mod key;
pub mod modifiers;
mod parse;
pub mod rules;
mod serialize;
mod transform;
pub mod trigger;

#[derive(Debug, Default)]
pub struct KeyboardHandler {}

impl KeyboardHandler {
    pub fn apply_rules(&self, profile: &KeyTransformRules) {
        KEY_HOOK.with_borrow_mut(|hook| hook.apply_rules(&profile));
    }

    pub fn set_listener(&self, listener: Option<Box<dyn Fn(&KeyEvent)>>) {
        KEY_HOOK.with_borrow_mut(|hook| hook.set_listener(listener));
    }

    pub fn is_enabled(&self) -> bool {
        KEY_HOOK.with_borrow(|hook| hook.is_enabled())
    }

    pub fn set_enabled(&self, enabled: bool) {
        KEY_HOOK.with_borrow_mut(|hook| {
            if enabled {
                hook.install()
            } else {
                hook.uninstall()
            }
        })
    }

    pub fn is_silent(&self) -> bool {
        KEY_HOOK.with_borrow(|hook| hook.is_silent())
    }

    pub fn set_silent(&self, silent: bool) {
        KEY_HOOK.with_borrow_mut(|inner| inner.set_silent(silent));
    }
}
