use crate::event::KeyEvent;
use crate::input;
use crate::rules::KeyTransformRules;
use crate::state::Bit256;
use crate::transform::KeyTransformMap;
use log::{debug, warn};
use std::cell::RefCell;
use std::ptr::addr_of_mut;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{INPUT, SendInput};
use windows::Win32::UI::WindowsAndMessaging::*;

pub const WM_KEY_HOOK_NOTIFY: u32 = 88475;

#[derive(Debug, Default)]
pub struct KeyboardHook {
    is_enabled: RefCell<bool>,
}

impl KeyboardHook {
    pub fn init(&self, owner: Option<HWND>) {
        unsafe { STATE.owner = owner };
    }

    pub fn apply_rules(&self, rules: &KeyTransformRules) {
        unsafe { STATE.transform_map = Some(KeyTransformMap::new(rules)) };
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled.borrow().to_owned()
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
        unsafe { STATE.is_notify_enabled }
    }

    pub fn set_notify_enabled(&self, enabled: bool) {
        unsafe { STATE.is_notify_enabled = enabled }

        if enabled {
            debug!("Hooks notifications enabled");
        } else {
            debug!("Hooks notifications disabled");
        }
    }
}

static mut STATE: HookState = {
    HookState {
        key_hook: None,
        mouse_hook: None,
        owner: None,
        transform_map: None,
        last_mouse_position: None,
        keyboard_state: Bit256::new(),
        is_notify_enabled: false,
    }
};

struct HookState {
    key_hook: Option<HHOOK>,
    mouse_hook: Option<HHOOK>,
    is_notify_enabled: bool,
    owner: Option<HWND>,
    transform_map: Option<KeyTransformMap>,
    last_mouse_position: Option<POINT>,
    keyboard_state: Bit256,
}

impl Drop for HookState {
    fn drop(&mut self) {
        self.transform_map = None;
        self.owner = None;
    }
}

fn install_key_hook() {
    unsafe {
        if let Some(_) = STATE.key_hook {
            warn!("Keyboard hook already installed");

            return;
        }

        match SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_hook_proc), None, 0) {
            Ok(handle) => {
                STATE.key_hook = Some(handle);

                debug!("Keyboard hook installed");
            }
            Err(e) => {
                STATE.key_hook = None;

                warn!("Failed to install keyboard hook: {}", e);
            }
        }
    }
}

fn uninstall_key_hook() {
    unsafe {
        if let Some(handle) = STATE.key_hook {
            match UnhookWindowsHookEx(handle) {
                Ok(_) => debug!("Keyboard hook uninstalled"),
                Err(e) => warn!("Failed to uninstall keyboard hook: {}", e),
            }
            STATE.key_hook = None;
        }
    }
}

fn install_mouse_hook() {
    unsafe {
        if let Some(_) = STATE.mouse_hook {
            warn!("Mouse hook already installed");

            return;
        }

        match SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0) {
            Ok(handle) => {
                STATE.mouse_hook = Some(handle);

                debug!("Mouse hook installed");
            }
            Err(e) => {
                STATE.mouse_hook = None;

                warn!("Failed to install mouse hook: {}", e);
            }
        }
    }
}

fn uninstall_mouse_hook() {
    unsafe {
        if let Some(handle) = STATE.mouse_hook {
            match UnhookWindowsHookEx(handle) {
                Ok(_) => debug!("Mouse hook uninstalled"),
                Err(e) => warn!("Failed to uninstall mouse hook: {}", e),
            }
            STATE.mouse_hook = None;
        }
    }
}

fn handle_key_event(mut event: KeyEvent) -> bool {
    unsafe {
        let state = addr_of_mut!(STATE);
        (*state)
            .keyboard_state
            .set(event.action.key.vk.0, event.action.transition.into_bool());
    };

    if !(event.is_injected && event.is_private) {
        unsafe {
            if let Some(ref map) = STATE.transform_map {
                event.rule = map.get(&event);
            }
        }
        debug!("Processing event: {}", event);
    } else {
        debug!("Ignoring event: {}", event);
    };

    let result = apply_transform(&event);
    notify_listener(event);
    result
}

fn apply_transform(event: &KeyEvent) -> bool {
    if let Some(rule) = event.rule {
        debug!("Applying rule: {}", rule);
        let input = input::build_input(&rule.actions, event.distance);
        unsafe { SendInput(&input, size_of::<INPUT>() as i32) };
        true
    } else {
        false
    }
}

fn notify_listener(event: KeyEvent) {
    unsafe {
        if !STATE.is_notify_enabled {
            return;
        }
        if let Some(ref hwnd) = STATE.owner {
            let l = LPARAM(&event as *const KeyEvent as isize);
            SendMessageW(*hwnd, WM_KEY_HOOK_NOTIFY, None, Some(l));
        }
    }
}

extern "system" fn key_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let event =
            KeyEvent::new_key_event(unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) }, &unsafe {
                STATE.keyboard_state
            });
        if handle_key_event(event) {
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(STATE.key_hook, code, w_param, l_param) }
}

extern "system" fn mouse_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let msg = w_param.0 as u32;
    let input = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };

    let handled = if msg == WM_MOUSEMOVE {
        false
        // handle_mouse_motion(input)
    } else {
        handle_mouse_button(msg, input)
    };

    if handled {
        return LRESULT(1);
    }
    unsafe { CallNextHookEx(STATE.mouse_hook, code, w_param, l_param) }
}

fn handle_mouse_button(msg: u32, input: MSLLHOOKSTRUCT) -> bool {
    match KeyEvent::new_mouse_event(msg, input, &unsafe { STATE.keyboard_state }) {
        Ok(event) => handle_key_event(event),
        Err(e) => {
            warn!("Failed to build event: {}", e);
            false
        }
    }
}

fn handle_mouse_motion(input: MSLLHOOKSTRUCT) -> bool {
    let last = unsafe { STATE.last_mouse_position.unwrap_or_else(|| input.pt) };
    let current = input.pt;
    let dx = current.x - last.x;
    let dy = current.y - last.y;
    unsafe { STATE.last_mouse_position = Some(current) }

    let (x_event, y_event) =
        KeyEvent::new_mouse_move_events(input, dx, dy, &unsafe { STATE.keyboard_state });
    handle_key_event(x_event) | handle_key_event(y_event)
}
