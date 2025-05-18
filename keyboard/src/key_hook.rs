use crate::key_event::KeyEvent;
use crate::transform_map::KeyTransformMap;
use crate::transform_rules::KeyTransformProfile;
use log::debug;
use std::cell::RefCell;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardState, INPUT, SendInput};
use windows::Win32::UI::WindowsAndMessaging::*;

thread_local! {
    static HOOK: RefCell<KeyboardHook> = RefCell::new(KeyboardHook::default());
}

#[derive(Default)]
struct KeyboardHook {
    transform_map: KeyTransformMap,
    handle: Option<HHOOK>,
    callback: Option<Box<dyn Fn(&KeyEvent)>>,
    is_silent: bool,
}

impl KeyboardHook {
    fn install(&mut self) {
        self.handle = Some(
            unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0) }
                .expect("Failed to install keyboard hook."),
        );

        debug!("Keyboard hook installed");
    }

    fn uninstall(&mut self) {
        if let Some(handle) = self.handle {
            unsafe { UnhookWindowsHookEx(handle) }.expect("Failed to uninstall keyboard hook.");
            self.handle = None;

            debug!("Keyboard hook uninstalled");
        }
    }

    fn load_profile(&mut self, profile: KeyTransformProfile) {
        self.transform_map = KeyTransformMap::from_profile(profile);
    }

    fn set_silent(&mut self, silent: bool) {
        self.is_silent = silent;

        debug!("Silent processing: {silent}");
    }

    fn set_callback(&mut self, callback: Option<Box<dyn Fn(&KeyEvent)>>) {
        self.callback = callback;
        if self.callback.is_some() {
            debug!("Callback set");
        } else {
            debug!("Callback removed");
        }
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        self.uninstall();
    }
}

#[derive(Debug, Default)]
pub struct KeyboardHandler {}

impl KeyboardHandler {
    pub fn set_profile(&self, profile: KeyTransformProfile) {
        HOOK.with_borrow_mut(|hook| hook.load_profile(profile));
    }

    pub fn set_callback(&self, callback: Option<Box<dyn Fn(&KeyEvent)>>) {
        HOOK.with_borrow_mut(|hook| hook.set_callback(callback));
    }

    pub fn is_enabled(&self) -> bool {
        HOOK.with_borrow(|hook| hook.handle.is_some())
    }

    pub fn set_enabled(&self, enabled: bool) {
        HOOK.with_borrow_mut(|hook| {
            if enabled {
                hook.install()
            } else {
                hook.uninstall()
            }
        })
    }

    pub fn is_silent(&self) -> bool {
        HOOK.with_borrow(|hook| hook.is_silent)
    }

    pub fn set_silent(&self, silent: bool) {
        HOOK.with_borrow_mut(|inner| inner.set_silent(silent));
    }
}

extern "system" fn keyboard_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    HOOK.with_borrow(|hook| {
        if code == HC_ACTION as i32 {
            let mut event = unsafe {
                let mut keyboard_state = [0; 256];
                GetKeyboardState(&mut keyboard_state).unwrap();
                KeyEvent::new(*(l_param.0 as *const KBDLLHOOKSTRUCT), keyboard_state)
            };

            debug!("EVENT: {}", event);

            if !event.is_private() {
                event.rule = hook.transform_map.get(&event)
            };

            if let Some(callback) = &hook.callback {
                if !&hook.is_silent {
                    callback(&event);
                }
            }

            if let Some(rule) = event.rule {
                debug!("RULE: {}", rule);

                let input = rule.target.create_input();
                unsafe { SendInput(&input, size_of::<INPUT>() as i32) };
                return LRESULT(1);
            }
        }

        unsafe { CallNextHookEx(hook.handle, code, w_param, l_param) }
    })
}
