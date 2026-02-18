use crate::event::{build_action_from_kbd_input, build_action_from_mouse_input, KeyEvent};
use crate::key::Key;
use crate::notify::install_notify_listener;
use crate::rules::{KeyTransformRule, KeyTransformRules};
use crate::state::KeyboardState;
use crate::transform::KeyTransformMap;
use crate::{input, notify};
use fxhash::FxHashSet;
use log::{debug, trace, warn};
use notify::notify_key_event;
use std::cell::RefCell;
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
        let state = get_keyboard_state(build_action_from_kbd_input(input).key);
        let event = KeyEvent::from_kbd_input(input, state);

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
        let state = get_keyboard_state(build_action_from_mouse_input(msg, input).key);
        let event = KeyEvent::from_mouse_input(msg, input, state);

        if handle_event(event) {
            return LRESULT(1);
        }
    }

    MOUSE_HOOK.with_borrow(|hook| unsafe { CallNextHookEx(*hook, code, w_param, l_param) })
}

fn handle_event(mut event: KeyEvent) -> bool {
    update_keyboard_state(&mut event);

    if SUPPRESSED_KEYS.with_borrow(|set| set.contains(&event.trigger.action.key)) {
        trace!("Event suppressed: {event}");
        notify_key_event(&event, None);
        true
    } else if event.is_private {
        trace!("Private event ignored: {event}");
        notify_key_event(&event, None);
        false
    } else {
        TRANSFOFM_MAP.with_borrow(|transform_map| {
            if let Some(map) = transform_map {
                trace!("Processing event: {event}");
                let rule = map.get(&event.trigger);
                notify_key_event(&event, rule);
                apply_transform(rule)
            } else {
                false
            }
        })
    }
}

fn apply_transform(rule: Option<&KeyTransformRule>) -> bool {
    if let Some(rule) = rule {
        debug!("Applying rule: {}", rule);

        let input = input::build_input(&rule.actions);
        unsafe { SendInput(&input, size_of::<INPUT>() as i32) };
        true
    } else {
        false
    }
}

fn get_keyboard_state(excluded: Key) -> KeyboardState {
    KEYBOARD_STATE.with_borrow_mut(|state| {
        state.exclude(excluded);
        *state
    })
}

fn update_keyboard_state(event: &KeyEvent) {
    KEYBOARD_STATE.with_borrow_mut(|state| state.update(event.trigger.action));
}
