use crate::keyboard::action::KeyTransition::Down;
use crate::keyboard::action::{KeyAction};
use crate::keyboard::event::{KeyEvent, SELF_EVENT_MARKER};
use crate::keyboard::modifiers::ModifierKeys;
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

extern "system" fn key_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        if handle_key_action(l_param) {
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(HOOK.key_hook, code, w_param, l_param) }
}

extern "system" fn mouse_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if w_param.0 == WM_MOUSEMOVE as usize {
        if handle_mouse_move(l_param) {
            return LRESULT(1);
        }
    }
    debug!(
        "Mouse hook called: code = {:?}, w_param = {:?}, l_param = {:?}",
        code, w_param, l_param
    );
    unsafe { CallNextHookEx(HOOK.mouse_hook, code, w_param, l_param) }
}

fn handle_key_action(l_param: LPARAM) -> bool {
    let mut event = build_key_event(l_param);

    if !(event.is_injected && event.is_private) {
        unsafe {
            if let Some(ref map) = HOOK.transform_map {
                event.rule = map.get(&event);
            }
        }
    }

    debug!("Processing event: {}", event);

    let result = apply_transform(&event);
    notify_listener(event);

    result
}

fn build_key_event<'a>(l_param: LPARAM) -> KeyEvent<'a> {
    let input = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
    let action = KeyAction::from(input);

    unsafe {
        HOOK.keyboard_state[action.key.vk_code as usize] = action.transition == Down;
    }

    KeyEvent {
        action,
        modifiers: ModifierKeys::from(&unsafe { HOOK.keyboard_state }),
        is_injected: input.flags.contains(LLKHF_INJECTED),
        is_private: input.dwExtraInfo as *const u8 == SELF_EVENT_MARKER.as_ptr(),
        time: input.time,
        rule: None,
    }
}

fn handle_mouse_move(l_param: LPARAM) -> bool {
    let input = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };

    let is_injected = (input.flags & (LLMHF_INJECTED | LLMHF_LOWER_IL_INJECTED)) != 0;
    let is_private = input.dwExtraInfo as *const u8 == SELF_EVENT_MARKER.as_ptr();

    let last = unsafe { HOOK.last_mouse_position.unwrap_or_else(|| input.pt) };
    let current = input.pt;

    let dx = current.x - last.x;
    let dy = current.y - last.y;

    unsafe { HOOK.last_mouse_position = Some(current) }

    // let rule = if !(is_injected && is_private) {
    //     // unsafe {
    //     //     if let Some(ref map) = MOUSE_HOOK.transform_map {
    //     //         event.rule = map.get(&event);
    //     //     }
    //     // }
    //     None
    // } else {
    //     None
    // };

    // let event = Event {
    //     // action: MouseAction { dx, dy },
    //     is_injected,
    //     is_private,
    //     time: input.time,
    //     rule,
    // };
    //
    // debug!("Processing mouse event: {}", event);
    //
    // let result = apply_transform(&event);
    // notify_listener(event);
    //
    // result

    false
}

fn apply_transform(event: &KeyEvent) -> bool {
    if let Some(rule) = event.rule {
        debug!("Applying rule: {}", rule);

        unsafe { SendInput(&rule.actions.input, size_of::<INPUT>() as i32) };
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
            SendMessageW(*hwnd, WM_KEY_HOOK_NOTIFY, None, Some(event.into()));
        }
    }
}
