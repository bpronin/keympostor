use crate::key_event::KeyEvent;
use crate::key_transform_map::KeyTransformMap;
use crate::key_transform_rule::KeyTransformProfile;
use crate::keyboard_state::KeyboardState;
use log::debug;
use std::cell::RefCell;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardState, SendInput};
use windows::Win32::UI::WindowsAndMessaging::*;

struct Statics {
    transform_map: KeyTransformMap,
    handle: Option<HHOOK>,
    callback: Option<Box<dyn Fn(&KeyEvent)>>,
    silent_processing: bool,
}

thread_local! {
    static STATICS: RefCell<Statics> = RefCell::new(Statics {
        transform_map: KeyTransformMap::new(),
        handle: None,
        callback: None,
        silent_processing: false,
    });
}

#[derive(Debug, Default)]
pub struct KeyboardHandler {}

impl KeyboardHandler {
    pub fn set_profile(&self, profile: KeyTransformProfile) {
        STATICS.with_borrow_mut(|g| {
            g.transform_map = KeyTransformMap::from_profile(profile);
        });
    }

    pub fn set_callback(&self, callback: Option<Box<dyn Fn(&KeyEvent)>>) {
        STATICS.with_borrow_mut(|g| {
            g.callback = callback;
            if g.callback.is_some() {
                debug!("Callback set");
            } else {
                debug!("Callback removed");
            }
        });
    }

    // pub(crate) fn set_callback<F>(&self, callback: Option<Box<F>>)
    // where
    //     F: Fn(&KeyboardEvent) + 'static,
    // {
    //     GLOBALS.with_borrow_mut(|g| {
    //         g.callback = callback;
    //         if g.callback.is_some() {
    //             debug!("Callback set");
    //         } else {
    //             debug!("Callback removed");
    //         }
    //     });
    // }

    pub fn is_enabled(&self) -> bool {
        STATICS.with_borrow(|g| g.handle.is_some())
    }

    pub fn set_enabled(&self, enabled: bool) {
        if enabled {
            self.install_hook()
        } else {
            self.uninstall_hook()
        }
    }

    pub fn is_silent(&self) -> bool {
        STATICS.with_borrow(|g| g.silent_processing)
    }

    pub fn set_silent(&self, silent: bool) {
        STATICS.with_borrow_mut(|g| g.silent_processing = silent)
    }

    fn install_hook(&self) {
        STATICS.with_borrow_mut(|g| {
            g.handle = Some(
                unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(Self::keyboard_proc), None, 0) }
                    .expect("Failed to install keyboard hook"),
            )
        });

        debug!("Keyboard hook installed");
    }

    fn uninstall_hook(&self) {
        STATICS.with_borrow_mut(|g| {
            if let Some(hook_handle) = g.handle {
                unsafe { UnhookWindowsHookEx(hook_handle) }
                    .expect("Failed to uninstall keyboard hook");
                g.handle = None;

                debug!("Keyboard hook uninstalled");
            }
        });
    }

    extern "system" fn keyboard_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        STATICS.with_borrow(|g| {
            if code == HC_ACTION as i32 {
                let mut event = KeyEvent::new(unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) });

                debug!("{}", event);

                if event.is_valid() {
                    if !event.is_private() {
                        event.rule = g
                            .transform_map
                            .get_rule(&event, || KeyboardState::capture())
                    };

                    if !g.silent_processing {
                        if let Some(callback) = &g.callback {
                            callback(&event);
                        }
                    }

                    if let Some(r) = event.rule {
                        let input = r.target.create_input();
                        unsafe { SendInput(&input, input.len() as i32) };
                        return LRESULT(1);
                    }
                }
            }

            unsafe { CallNextHookEx(g.handle, code, w_param, l_param) }
        })
    }
}

// todo: impl Drop for KeyboardHandler {
//     fn drop(&mut self) {
//         self.uninstall_hook()
//     }
// }
