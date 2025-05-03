use crate::key_event::KeyboardEvent;
use crate::transform::KeyTransformMap;
use log::debug;
use std::cell::RefCell;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardState;
use windows::Win32::UI::WindowsAndMessaging::*;

/// A marker to detect self generated keyboard events.
/// Must be exactly `static` not `const`! Because of `const` ptrs may point at different addresses.
/// Content does not matter.
pub static SELF_MARKER: &str = "self";

struct Statics {
    transform_map: KeyTransformMap,
    handle: Option<HHOOK>,
    callback: Option<Box<dyn Fn(&KeyboardEvent)>>,
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
pub(crate) struct KeyboardHandler {}

impl KeyboardHandler {
    pub(crate) fn set_rules(&self, transform_map: KeyTransformMap) {
        STATICS.with_borrow_mut(|g| g.transform_map = transform_map);
    }

    pub(crate) fn set_callback(&self, callback: Option<Box<dyn Fn(&KeyboardEvent)>>) {
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

    pub(crate) fn is_enabled(&self) -> bool {
        STATICS.with_borrow(|g| g.handle.is_some())
    }

    pub(crate) fn set_enabled(&self, enabled: bool) {
        if enabled {
            self.install_hook()
        } else {
            self.uninstall_hook()
        }
    }

    pub(crate) fn is_silent(&self) -> bool {
        STATICS.with_borrow(|g| g.silent_processing)
    }

    pub(crate) fn set_silent(&self, silent: bool) {
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
                let kb = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
                
                let mut kb_state = [0u8; 256];
                unsafe { GetKeyboardState(&mut kb_state) }.unwrap();
                
                let mut event = KeyboardEvent::from_kb(kb, kb_state);

                debug!("{}", event);

                if event.is_valid() {
                    let target = if !event.is_private() {
                        event.is_trigger = true;
                        g.transform_map.get(&event.action)
                    } else {
                        event.is_trigger = false;
                        None
                    };

                    if !g.silent_processing {
                        if let Some(callback) = &g.callback {
                            callback(&event)
                        }
                    }

                    if let Some(t) = target {
                        t.send();
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
