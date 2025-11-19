use crate::key;
use crate::keyboard::action::KeyTransition::Down;
use crate::keyboard::action::{KeyAction, KeyTransition};
use crate::keyboard::event::{KeyEvent, SELF_EVENT_MARKER};
use crate::keyboard::key::{Key, VirtualKey};
use crate::keyboard::modifiers::ModifierKeys;
use crate::keyboard::transform::KeyTransformMap;
use log::{debug, warn};
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, VIRTUAL_KEY, VK_LBUTTON};
use windows::Win32::UI::WindowsAndMessaging::*;
use KeyTransition::Up;

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
        HOOK.keyboard_state[event.action.key.vk_code as usize] = event.action.transition == Down;
    }

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

extern "system" fn key_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        if handle_key_event(build_key_event(l_param)) {
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(HOOK.key_hook, code, w_param, l_param) }
}

fn build_key_event<'a>(l_param: LPARAM) -> KeyEvent<'a> {
    let input = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
    KeyEvent {
        action: KeyAction::from(input),
        modifiers: ModifierKeys::from(&unsafe { HOOK.keyboard_state }),
        is_injected: input.flags.contains(LLKHF_INJECTED),
        is_private: input.dwExtraInfo as *const u8 == SELF_EVENT_MARKER.as_ptr(),
        time: input.time,
        rule: None,
    }
}

extern "system" fn mouse_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let handled = match w_param.0 as u32 {
        WM_MOUSEMOVE => handle_mouse_move(l_param),
        WM_MOUSEWHEEL => handle_mouse_wheel(l_param),
        WM_LBUTTONDOWN => handle_mouse_button(l_param, key!("LEFT_BUTTON"), Down),
        WM_RBUTTONDOWN => handle_mouse_button(l_param, key!("RIGHT_BUTTON"), Down),
        WM_MBUTTONDOWN => handle_mouse_button(l_param, key!("MIDDLE_BUTTON"), Down),
        WM_LBUTTONUP => handle_mouse_button(l_param, key!("LEFT_BUTTON"), Up),
        WM_RBUTTONUP => handle_mouse_button(l_param, key!("RIGHT_BUTTON"), Up),
        WM_MBUTTONUP => handle_mouse_button(l_param, key!("MIDDLE_BUTTON"), Up),
        WM_XBUTTONDOWN => handle_mouse_x_button(l_param, Down),
        WM_XBUTTONUP => handle_mouse_x_button(l_param, Up),
        _ => {
            warn!(
                "Unhandled mouse event: c = {:?}, w = {:?}, l = {:?}",
                code, w_param, l_param
            );
            false
        }
    };

    if handled {
        LRESULT(1)
    } else {
        unsafe { CallNextHookEx(HOOK.mouse_hook, code, w_param, l_param) }
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

    debug!("Mouse moved: dx = {}, dy = {}", dx, dy);
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

fn handle_mouse_wheel(l_param: LPARAM) -> bool {
    let input = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };
    debug!("Mouse wheel: delta = {}", input.mouseData);
    false
}

fn build_button_event<'a>(l_param: LPARAM, key: Key, transition: KeyTransition) -> KeyEvent<'a> {
    let input = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };
    KeyEvent {
        action:KeyAction { key, transition },
        modifiers: ModifierKeys::from(&unsafe { HOOK.keyboard_state }),
        is_injected: (input.flags & (LLMHF_INJECTED | LLMHF_LOWER_IL_INJECTED)) != 0,
        is_private: input.dwExtraInfo as *const u8 == SELF_EVENT_MARKER.as_ptr(),
        time: input.time,
        rule: None,
    }
}

fn handle_mouse_button(l_param: LPARAM, key: Key, transition: KeyTransition) -> bool {
    handle_key_event(build_button_event(l_param, key, transition))
}

fn handle_mouse_x_button(l_param: LPARAM, transition: KeyTransition) -> bool {
    let input = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };
    debug!(
        "Mouse button {}: action = {:?}",
        transition, input.mouseData
    );
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
