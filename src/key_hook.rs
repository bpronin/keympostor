use crate::key_action::KeyAction;
use crate::key_code::{KeyCode, Key, MAX_SC_CODE, MAX_VK_CODE};
use crate::key_modifier::KeyModifiers;
use crate::transform::KeyTransformMap;
use log::{debug, warn};
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use crate::key_transition::KeyTransition;

/// A marker to detect self generated keyboard events.
/// Must be exactly `static` not `const`! Because of `const` ptrs may point at different addresses.
/// Content does not matter.
pub static SELF_MARKER: &str = "self";

pub(crate) struct KeyboardEvent {
    kb: KBDLLHOOKSTRUCT,
    pub(crate) action: KeyAction,
}

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
        silent_processing: false
    });
}

impl KeyboardEvent {
    fn from(l_param: LPARAM, modifiers: Option<KeyModifiers>) -> Self {
        let kb = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
        Self {
            kb,
            action: KeyAction {
                key: Key::from_kb(&kb),
                transition: KeyTransition::from_kb(&kb),
                modifiers,
            },
        }
    }

    pub(crate) fn flags(&self) -> u32 {
        self.kb.flags.0
    }

    pub(crate) fn time(&self) -> u32 {
        self.kb.time
    }

    pub(crate) fn is_injected(&self) -> bool {
        self.kb.flags.contains(LLKHF_INJECTED)
    }

    pub(crate) fn is_private(&self) -> bool {
        self.is_injected() && (self.kb.dwExtraInfo as *const u8 == SELF_MARKER.as_ptr())
    }

    pub(crate) fn is_valid(&self) -> bool {
        if self.kb.scanCode > MAX_SC_CODE as u32 {
            warn!("Ignored invalid scancode: 0x{:04X}.", self.kb.scanCode);
            false
        } else if self.kb.vkCode > MAX_VK_CODE as u32 {
            warn!("Ignored invalid virtual key: 0x{:04X}.", self.kb.vkCode);
            false
        } else if self.kb.time == 0 {
            warn!("Ignored invalid time: {}.", self.kb.time);
            false
        } else {
            true
        }
    }

    pub(crate) fn is_processable(&self) -> bool {
        STATICS.with_borrow(|g| !g.transform_map.get(&self.action).is_some())
    }
}

impl Display for KeyboardEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let scancode = self.action.key.scancode.unwrap();
        let virtual_key = self.action.key.virtual_key.unwrap();
        let modifiers = if let Some(m) = self.action.modifiers {
            &format!("{}", m)
        } else {
            "ANY"
        };
        write!(
            f,
            "T: {:>9} | {:18} | SC: {} | VK: {} | M: {} | F: {:08b} | {:8} | {:8}",
            self.time(),
            scancode.name(),
            scancode,
            virtual_key,
            modifiers,
            self.flags(),
            if self.is_injected() { "INJECTED" } else { "" },
            if self.is_private() { "PRIVATE" } else { "" }
        )
    }
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
                let event = KeyboardEvent::from(l_param, Some(KeyModifiers::capture_state()));

                debug!("{}", event);

                if event.is_valid() {
                    if !g.silent_processing {
                        if let Some(callback) = &g.callback {
                            callback(&event)
                        }
                    }

                    if !event.is_private() {
                        if let Some(target) = g.transform_map.get(&event.action) {
                            target.send();
                            return LRESULT(1);
                        }
                    }
                }
            }

            unsafe { CallNextHookEx(g.handle, code, w_param, l_param) }
        })
    }
}

// impl Drop for KeyboardHandler {
//     fn drop(&mut self) {
//         self.uninstall_hook()
//     }
// }
