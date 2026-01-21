use crate::event::KeyEvent;
use crate::input;
use crate::rules::KeyTransformRules;
use crate::state::KeyboardState;
use crate::transform::KeyTransformMap;
use log::{debug, warn};
use std::cell::RefCell;
use std::rc::Rc;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT};
use windows::Win32::UI::WindowsAndMessaging::*;

pub const WM_KEY_HOOK_NOTIFY: u32 = 88475;

#[derive(Debug, Default)]
pub struct KeyboardHook {
    is_enabled: RefCell<bool>,
}

impl KeyboardHook {
    pub fn init(&self, owner: Option<HWND>) {
        NOTIFY.with_borrow_mut(|state| state.target = owner);
    }

    pub fn apply_rules(&self, rules: Option<&KeyTransformRules>) {
        match rules {
            Some(rules) => {
                TRANSFOFM_MAP.replace(Some(KeyTransformMap::new(rules)));
            }
            None => {
                TRANSFOFM_MAP.replace(None);
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        *self.is_enabled.borrow()
    }

    pub fn set_enabled(&self, enabled: bool) {
        if enabled {
            install_key_hook();
            install_mouse_hook();
        } else {
            uninstall_key_hook();
            uninstall_mouse_hook();
        }
        self.is_enabled.replace(enabled);
    }

    pub fn is_notify_enabled(&self) -> bool {
        NOTIFY.with_borrow(|notify| notify.is_enabled)
    }

    pub fn set_notify_enabled(&self, enabled: bool) {
        NOTIFY.with_borrow_mut(|notify| {
            notify.is_enabled = enabled;
            if notify.is_enabled {
                debug!("Hooks notifications enabled");
            } else {
                debug!("Hooks notifications disabled");
            }
        });
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        self.set_enabled(false);
    }
}

#[derive(Default)]
struct NotifyState {
    target: Option<HWND>,
    is_enabled: bool,
}

impl Drop for NotifyState {
    fn drop(&mut self) {
        self.target = None;
    }
}

thread_local! {
    static KEY_HOOK: RefCell<Option<HHOOK>> = RefCell::new(None);
    static MOUSE_HOOK: RefCell<Option<HHOOK>> = RefCell::new(None);
    static KEYBOARD_STATE: RefCell<KeyboardState> = RefCell::new(KeyboardState::new());
    static LAST_MOUSE_POSITION: RefCell<Option<POINT>> = RefCell::new(None);
    static TRANSFOFM_MAP: RefCell<Option<KeyTransformMap>> = RefCell::new(None);
    static NOTIFY: RefCell<NotifyState> = RefCell::new(Default::default());
}

fn install_key_hook() {
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

    if !(event.is_injected && event.is_private) {
        TRANSFOFM_MAP.with_borrow(|transform_map| {
            if let Some(map) = transform_map {
                if let Some(rule) = map.get(&event) {
                    event.rule = Some(Rc::clone(rule));
                }
            }
        });

        debug!("Processing event: {}", event);
    } else {
        debug!("Ignoring event: {}", event);
    };

    let transformed = apply_transform(&event);
    notify_listener(event);
    transformed
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

fn notify_listener(event: KeyEvent) {
    NOTIFY.with_borrow(|notify| {
        if !notify.is_enabled {
            return;
        }

        if let Some(hwnd) = notify.target {
            let l_param = LPARAM(&event as *const KeyEvent as isize);
            unsafe { SendMessageW(hwnd, WM_KEY_HOOK_NOTIFY, None, Some(l_param)) };
        }
    })
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
