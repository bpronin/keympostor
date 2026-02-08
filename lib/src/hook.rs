use crate::event::KeyEvent;
use crate::key::Key;
use crate::notify::install_notify_listener;
use crate::rules::KeyTransformRules;
use crate::state::KeyboardState;
use crate::transform::KeyTransformMap;
use crate::{input, notify};
use fxhash::FxHashSet;
use log::{debug, trace, warn};
use std::cell::RefCell;
use std::rc::Rc;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT};
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Debug, Default)]
pub struct KeyboardHook {}

impl KeyboardHook {
    pub fn install(&self, owner: HWND) {
        install_notify_listener(owner);
        install_keyboard_hook();

        #[cfg(feature = "no_mouse")]
        warn!("Mouse hook is disabled by feature flag");
        #[cfg(not(feature = "no_mouse"))]
        install_mouse_hook();
    }

    pub fn set_rules(&self, rules: Option<&KeyTransformRules>) {
        TRANSFOFM_MAP.replace(match rules {
            None => None,
            Some(r) => Some(KeyTransformMap::new(r)),
        });
    }

    pub fn suppress_keys(&self, keys: &[&Key]) {
        let mut set: FxHashSet<Key> = FxHashSet::default();
        set.extend(keys.iter().cloned());
        SUPPRESSED_KEYS.replace(set);
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        uninstall_key_hook();

        #[cfg(not(feature = "no_mouse"))]
        uninstall_mouse_hook();
    }
}

thread_local! {
    static KEY_HOOK: RefCell<Option<HHOOK>> = RefCell::new(None);
    static MOUSE_HOOK: RefCell<Option<HHOOK>> = RefCell::new(None);
    static KEYBOARD_STATE: RefCell<KeyboardState> = RefCell::new(KeyboardState::new());
    static LAST_MOUSE_POSITION: RefCell<Option<POINT>> = RefCell::new(None);
    static TRANSFOFM_MAP: RefCell<Option<KeyTransformMap>> = RefCell::new(None);
    static SUPPRESSED_KEYS: RefCell<FxHashSet<Key>> = RefCell::new(FxHashSet::default());
}

fn install_keyboard_hook() {
    KEY_HOOK.with_borrow_mut(|hook| {
        if hook.is_some() {
            warn!("Keyboard hook already installed");

            return;
        }

        match unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_hook_proc), None, 0) } {
            Ok(handle) => {
                *hook = Some(handle);

                debug!("Keyboard hook installed");
            }
            Err(e) => {
                *hook = None;

                warn!("Failed to install keyboard hook: {}", e);
            }
        }
    })
}

fn uninstall_key_hook() {
    KEY_HOOK.with_borrow_mut(|hook| {
        if let Some(handle) = hook.take() {
            match unsafe { UnhookWindowsHookEx(handle) } {
                Ok(_) => debug!("Keyboard hook uninstalled"),
                Err(e) => warn!("Failed to uninstall keyboard hook: {}", e),
            }
        } else {
            warn!("Keyboard hook already uninstalled");
        }
    })
}

fn install_mouse_hook() {
    MOUSE_HOOK.with_borrow_mut(|hook| {
        if hook.is_some() {
            warn!("Mouse hook already installed");

            return;
        }

        match unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0) } {
            Ok(handle) => {
                *hook = Some(handle);

                debug!("Mouse hook installed");
            }
            Err(e) => {
                *hook = None;

                warn!("Failed to install mouse hook: {}", e);
            }
        }
    });
}

fn uninstall_mouse_hook() {
    MOUSE_HOOK.with_borrow_mut(|hook| {
        if let Some(handle) = hook.take() {
            match unsafe { UnhookWindowsHookEx(handle) } {
                Ok(_) => debug!("Mouse hook uninstalled"),
                Err(e) => warn!("Failed to uninstall mouse hook: {}", e),
            }
        } else {
            warn!("Mouse hook already uninstalled");
        }
    })
}

extern "system" fn key_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let event = KeyEvent::new_key_event(
            unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) },
            &KEYBOARD_STATE.with(|state| *state.borrow()),
        );
        if handle_event(event) {
            return LRESULT(1);
        }
    }

    KEY_HOOK.with_borrow(|hook| unsafe { CallNextHookEx(*hook, code, w_param, l_param) })
}

extern "system" fn mouse_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let msg = w_param.0 as u32;
    let input = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };

    if handle_mouse_button(msg, input) {
        return LRESULT(1);
    }

    MOUSE_HOOK.with_borrow(|hook| unsafe { CallNextHookEx(*hook, code, w_param, l_param) })
}

fn handle_event(mut event: KeyEvent) -> bool {
    KEYBOARD_STATE.with_borrow_mut(|state| state.update(event.action));

    let handled = if SUPPRESSED_KEYS.with_borrow(|set| set.contains(&event.action.key)) {
        trace!("Event suppressed: {event}");
        true
    } else if event.is_private {
        trace!("Private event ignored: {event}");
        false
    } else {
        TRANSFOFM_MAP.with_borrow(|transform_map| {
            if let Some(map) = transform_map {
                if let Some(rule) = map.get(&event) {
                    event.rule = Some(Rc::clone(rule));
                }
            }
        });

        trace!("Processing event: {event}");
        apply_transform(&event)
    };

    notify::notify_listener(event);
    handled
}

fn handle_mouse_button(msg: u32, input: MSLLHOOKSTRUCT) -> bool {
    if msg == WM_MOUSEMOVE {
        return false;
    }

    let keyboard_state = KEYBOARD_STATE.with_borrow(|state| *state);
    match KeyEvent::new_mouse_event(msg, input, &keyboard_state) {
        Ok(event) => handle_event(event),
        Err(e) => {
            warn!("Failed to build event: {}", e);
            false
        }
    }
}

fn apply_transform(event: &KeyEvent) -> bool {
    if let Some(rule) = &event.rule {
        debug!("Applying rule: {}", rule);

        let input = input::build_input(&rule.actions);
        unsafe { SendInput(&input, size_of::<INPUT>() as i32) };
        true
    } else {
        false
    }
}
