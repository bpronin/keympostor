use crate::key_event::KeyEvent;
use crate::key_transform_map::KeyTransformMap;
use crate::key_transform_rule::KeyTransformProfile;
use log::debug;
use std::cell::RefCell;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT};
use windows::Win32::UI::WindowsAndMessaging::*;
use crate::key_modifiers::KeyModifiers;

thread_local! {
    static INNER: RefCell<InnerKeyboardHandler> = RefCell::new(InnerKeyboardHandler::default());
}

#[derive(Default)]
struct InnerKeyboardHandler {
    handle: Option<HHOOK>,
    transform_map: KeyTransformMap,
    callback: Option<Box<dyn Fn(&KeyEvent)>>,
    silent_processing: bool,
}

impl InnerKeyboardHandler {
    fn load_profile(&mut self, profile: KeyTransformProfile) {
        self.transform_map = KeyTransformMap::from_profile(profile);
    }

    fn set_callback(&mut self, callback: Option<Box<dyn Fn(&KeyEvent)>>) {
        self.callback = callback;
        if self.callback.is_some() {
            debug!("Callback set");
        } else {
            debug!("Callback removed");
        }
    }

    fn install_hook(&mut self) {
        self.handle = Some(
            unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(Self::keyboard_proc), None, 0) }
                .expect("Failed to install keyboard hook"),
        );
            
        debug!("Keyboard hook installed");
    }

    fn uninstall_hook(&mut self) {
        if let Some(handle) = self.handle {
            unsafe { UnhookWindowsHookEx(handle) }.expect("Failed to uninstall keyboard hook");
            self.handle = None;

            debug!("Keyboard hook uninstalled");
        }
    }

    extern "system" fn keyboard_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        INNER.with_borrow(|inner| {
            if code == HC_ACTION as i32 {
                let mut event = KeyEvent::new(unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) });

                debug!("EVENT: {}", event);

                if event.is_valid() {
                    if !event.is_private() {
                        event.rule = inner.transform_map.get(&event, || KeyModifiers::capture())
                    };

                    if !inner.silent_processing {
                        if let Some(callback) = &inner.callback {
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
            }

            unsafe { CallNextHookEx(inner.handle, code, w_param, l_param) }
        })
    }
}

impl Drop for InnerKeyboardHandler {
    fn drop(&mut self) {
        self.set_callback(None);
        self.uninstall_hook();
    }
}

#[derive(Debug, Default)]
pub struct KeyboardHandler {}

impl KeyboardHandler {
    pub fn set_profile(&self, profile: KeyTransformProfile) {
        INNER.with_borrow_mut(|inner| inner.load_profile(profile));
    }

    pub fn set_callback(&self, callback: Option<Box<dyn Fn(&KeyEvent)>>) {
        INNER.with_borrow_mut(|inner| inner.set_callback(callback));
    }

    pub fn is_enabled(&self) -> bool {
        INNER.with_borrow(|inner| inner.handle.is_some())
    }

    pub fn set_enabled(&self, enabled: bool) {
        INNER.with_borrow_mut(|inner| {
            if enabled {
                inner.install_hook()
            } else {
                inner.uninstall_hook()
            }
        })
    }

    pub fn is_silent(&self) -> bool {
        INNER.with_borrow(|inner| inner.silent_processing)
    }

    pub fn set_silent(&self, silent: bool) {
        INNER.with_borrow_mut(|inner| inner.silent_processing = silent);
        debug!("Silent processing: {silent}.");
    }
}
