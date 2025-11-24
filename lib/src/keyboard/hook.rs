use crate::keyboard::event::KeyEvent;
use crate::keyboard::input;
use crate::keyboard::transform::KeyTransformMap;
use log::{debug, warn};
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT};
use windows::Win32::UI::WindowsAndMessaging::*;

pub const WM_KEY_HOOK_NOTIFY: u32 = 88475;

pub(crate) static mut HOOK: HookState = {
    HookState {
        key_hook: None,
        mouse_hook: None,
        owner: None,
        transform_map: None,
        last_mouse_position: None,
        keyboard_state: [false; 256],
        is_notify_enabled: false,
    }
};

pub(crate) struct HookState {
    key_hook: Option<HHOOK>,
    mouse_hook: Option<HHOOK>,
    pub(crate) is_notify_enabled: bool,
    pub(crate) owner: Option<HWND>,
    pub(crate) transform_map: Option<KeyTransformMap>,
    pub(crate) last_mouse_position: Option<POINT>,
    keyboard_state: [bool; 256],
}

impl Drop for HookState {
    fn drop(&mut self) {
        self.transform_map = None;
        self.owner = None;
    }
}

pub(crate) fn install_key_hook() {
    unsafe {
        if let Some(_) = HOOK.key_hook {
            warn!("Keyboard hook already installed");

            return;
        }

        match SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_hook_proc), None, 0) {
            Ok(handle) => {
                HOOK.key_hook = Some(handle);

                debug!("Keyboard hook installed");
            }
            Err(e) => {
                HOOK.key_hook = None;

                warn!("Failed to install keyboard hook: {}", e);
            }
        }
    }
}

pub(crate) fn uninstall_key_hook() {
    unsafe {
        if let Some(handle) = HOOK.key_hook {
            match UnhookWindowsHookEx(handle) {
                Ok(_) => debug!("Keyboard hook uninstalled"),
                Err(e) => warn!("Failed to uninstall keyboard hook: {}", e),
            }
            HOOK.key_hook = None;
        }
    }
}

pub(crate) fn install_mouse_hook() {
    unsafe {
        if let Some(_) = HOOK.mouse_hook {
            warn!("Mouse hook already installed");

            return;
        }

        match SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0) {
            Ok(handle) => {
                HOOK.mouse_hook = Some(handle);

                debug!("Mouse hook installed");
            }
            Err(e) => {
                HOOK.mouse_hook = None;

                warn!("Failed to install mouse hook: {}", e);
            }
        }
    }
}

pub(crate) fn uninstall_mouse_hook() {
    unsafe {
        if let Some(handle) = HOOK.mouse_hook {
            match UnhookWindowsHookEx(handle) {
                Ok(_) => debug!("Mouse hook uninstalled"),
                Err(e) => warn!("Failed to uninstall mouse hook: {}", e),
            }
            HOOK.mouse_hook = None;
        }
    }
}

fn handle_key_event(mut event: KeyEvent) -> bool {
    unsafe {
        HOOK.keyboard_state[event.action.key.vk.0 as usize] = event.action.transition.into_bool();
    }

    if !(event.is_injected && event.is_private) {
        unsafe {
            if let Some(ref map) = HOOK.transform_map {
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
        if !HOOK.is_notify_enabled {
            return;
        }
        if let Some(ref hwnd) = HOOK.owner {
            let l = LPARAM(&event as *const KeyEvent as isize);
            SendMessageW(*hwnd, WM_KEY_HOOK_NOTIFY, None, Some(l));
        }
    }
}

extern "system" fn key_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let event =
            KeyEvent::new_key_event(unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) }, &unsafe {
                HOOK.keyboard_state
            });
        if handle_key_event(event) {
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(HOOK.key_hook, code, w_param, l_param) }
}

extern "system" fn mouse_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let msg = w_param.0 as u32;
    let input = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };

    let handled = if msg == WM_MOUSEMOVE {
        handle_mouse_motion(input)
    } else {
        handle_mouse_button(msg, input)
    };

    if handled {
        return LRESULT(1);
    }
    unsafe { CallNextHookEx(HOOK.mouse_hook, code, w_param, l_param) }
}

fn handle_mouse_button(msg: u32, input: MSLLHOOKSTRUCT) -> bool {
    match KeyEvent::new_mouse_event(msg, input, &unsafe { HOOK.keyboard_state }) {
        Ok(event) => handle_key_event(event),
        Err(e) => {
            warn!("Failed to build event: {}", e);
            false
        }
    }
}

fn handle_mouse_motion(input: MSLLHOOKSTRUCT) -> bool {
    let last = unsafe { HOOK.last_mouse_position.unwrap_or_else(|| input.pt) };
    let current = input.pt;
    let dx = current.x - last.x;
    let dy = current.y - last.y;
    unsafe { HOOK.last_mouse_position = Some(current) }

    let (x_event, y_event) =
        KeyEvent::new_mouse_move_events(input, dx, dy, &unsafe { HOOK.keyboard_state });
    handle_key_event(x_event) | handle_key_event(y_event)
}
