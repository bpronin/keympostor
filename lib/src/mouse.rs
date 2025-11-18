use log::debug;
use windows::Win32::Foundation::HWND;
use crate::mouse::hook::{install_mouse_hook, uninstall_mouse_hook, MOUSE_HOOK};

mod hook;
#[derive(Debug, Default)]
pub struct MouseHook;

impl MouseHook {
    pub fn init(&self, owner: Option<HWND>) {
        unsafe {
            MOUSE_HOOK.owner = owner;
        }
    }

    pub fn is_enabled(&self) -> bool {
        match unsafe { MOUSE_HOOK.handle } {
            Some(_) => true,
            None => false,
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        if enabled {
            install_mouse_hook()
        } else {
            uninstall_mouse_hook()
        }
    }

    pub fn is_notify_enabled(&self) -> bool {
        unsafe { MOUSE_HOOK.is_notify_enabled }
    }

    pub fn set_notify_enabled(&self, enabled: bool) {
        unsafe { MOUSE_HOOK.is_notify_enabled = enabled }

        if enabled {
            debug!("Notifications enabled");
        } else {
            debug!("Notifications disabled");
        }
    }
}