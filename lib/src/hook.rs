use crate::event::KeyEvent;
use crate::key::Key;
use crate::notify::install_notify_listener;
use crate::rules::{KeyTransformRule, KeyTransformRules};
use crate::state::KeyboardState;
use crate::transform::KeyTransformMap;
use crate::{input, notify};
use fxhash::FxHashSet;
use log::{debug, trace, warn};
use std::cell::RefCell;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT};
use windows::Win32::UI::WindowsAndMessaging::*;
use crate::action::KeyAction;

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
        let map = rules.and_then(|r| Some(KeyTransformMap::new(r.iter())));
        TRANSFOFM_MAP.replace(map);
    }

    pub fn suppress_keys(&self, keys: &[Key]) {
        SUPPRESSED_KEYS.replace(FxHashSet::from_iter(keys.iter().cloned()));
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
    static KEYBOARD_STATE: RefCell<KeyboardState> = RefCell::new(KeyboardState::default());
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
        let input = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
        let state = KEYBOARD_STATE.with(|state| *state.borrow());
        let event = KeyEvent::from_key_input(input, state);
        if handle_event(event) {
            return LRESULT(1);
        }
    }

    KEY_HOOK.with_borrow(|hook| unsafe { CallNextHookEx(*hook, code, w_param, l_param) })
}

extern "system" fn mouse_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let msg = w_param.0 as u32;
    if msg != WM_MOUSEMOVE {
        let input = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };
        let state = KEYBOARD_STATE.with_borrow(|state| *state);
        match KeyEvent::from_mouse_input(msg, input, state) {
            Ok(event) => {
                if handle_event(event) {
                    return LRESULT(1);
                }
            }
            Err(error) => warn!("Failed to build mouse event: {}", error),
        }
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
                event.rule = map.get(&event).cloned();
            }
        });

        trace!("Processing event: {event}");
        apply_transform(&event)
    };

    notify::notify_listener(event);
    handled
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
