use crate::keyboard::action::{KeyAction, KeyActionSequence};
use crate::keyboard::error::KeyError;
use crate::keyboard::event::{KeyEvent, SELF_EVENT_MARKER};
use crate::keyboard::input;
use crate::keyboard::key::{
    key_by_code, Key, KEY_LEFT_BUTTON, KEY_MIDDLE_BUTTON, KEY_MOUSE_X, KEY_MOUSE_Y,
    KEY_RIGHT_BUTTON, KEY_WHEEL_X, KEY_WHEEL_Y, KEY_XBUTTON1, KEY_XBUTTON2,
};
use crate::keyboard::modifiers::ModifierKeys;
use crate::keyboard::sc::ScanCode;
use crate::keyboard::transform::KeyTransformMap;
use crate::keyboard::transition::KeyTransition;
use crate::keyboard::transition::KeyTransition::Down;
use crate::keyboard::transition::KeyTransition::Up;
use crate::keyboard::vk::VirtualKey;
use log::{debug, warn};
use std::result;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MOUSEINPUT, VIRTUAL_KEY,
};
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

fn handle_key_event(mut event: KeyEvent, delta: i32) -> bool {
    unsafe {
        HOOK.keyboard_state[event.action.key.vk_code as usize] = event.action.transition == Down;
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

    let result = apply_transform(&event, delta);
    notify_listener(event);
    result
}

fn apply_transform(event: &KeyEvent, delta: i32) -> bool {
    if let Some(rule) = event.rule {
        debug!("Applying rule: {}", rule);
        let input = input::build_input(&rule.actions, delta);
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
        let input = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
        if handle_key_event(build_key_event(input), 0) {
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(HOOK.key_hook, code, w_param, l_param) }
}

extern "system" fn mouse_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let input = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };
    let handled = match w_param.0 as u32 {
        WM_MOUSEMOVE => handle_mouse_move(&input),
        WM_MOUSEWHEEL => handle_mouse_wheel(&input, false),
        WM_MOUSEHWHEEL => handle_mouse_wheel(&input, true),
        WM_LBUTTONDOWN => handle_mouse_button(&input, &KEY_LEFT_BUTTON, Down),
        WM_RBUTTONDOWN => handle_mouse_button(&input, &KEY_RIGHT_BUTTON, Down),
        WM_MBUTTONDOWN => handle_mouse_button(&input, &KEY_MIDDLE_BUTTON, Down),
        WM_LBUTTONUP => handle_mouse_button(&input, &KEY_LEFT_BUTTON, Up),
        WM_RBUTTONUP => handle_mouse_button(&input, &KEY_RIGHT_BUTTON, Up),
        WM_MBUTTONUP => handle_mouse_button(&input, &KEY_MIDDLE_BUTTON, Up),
        WM_XBUTTONDOWN => handle_mouse_x_button(&input, Down),
        WM_XBUTTONUP => handle_mouse_x_button(&input, Up),
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

fn build_key_event<'a>(input: KBDLLHOOKSTRUCT) -> KeyEvent<'a> {
    KeyEvent {
        action: KeyAction {
            key: key_by_code(
                input.vkCode as u8,
                (input.scanCode as u8, input.flags.contains(LLKHF_EXTENDED)),
            ),
            transition: if input.flags.contains(LLKHF_UP) {
                Up
            } else {
                Down
            },
        },
        modifiers: ModifierKeys::from(&unsafe { HOOK.keyboard_state }),
        is_injected: input.flags.contains(LLKHF_INJECTED),
        is_private: input.dwExtraInfo == SELF_EVENT_MARKER,
        time: input.time,
        rule: None,
    }
}

fn build_mouse_event<'a>(
    input: &MSLLHOOKSTRUCT,
    key: &'static Key,
    transition: KeyTransition,
) -> KeyEvent<'a> {
    KeyEvent {
        action: KeyAction { key, transition },
        modifiers: ModifierKeys::from(&unsafe { HOOK.keyboard_state }),
        is_injected: (input.flags & (LLMHF_INJECTED | LLMHF_LOWER_IL_INJECTED)) != 0,
        is_private: input.dwExtraInfo == SELF_EVENT_MARKER,
        time: input.time,
        rule: None,
    }
}

fn build_x_button_event(
    input: &MSLLHOOKSTRUCT,
    transition: KeyTransition,
) -> Result<KeyEvent, KeyError> {
    let key = match (input.mouseData >> 16) as u16 {
        1 => &KEY_XBUTTON1,
        2 => &KEY_XBUTTON2,
        b => {
            return Err(KeyError::new(&format!("Unsupported button: {}", b)));
        }
    };
    Ok(build_mouse_event(input, key, transition))
}

fn handle_mouse_button(
    input: &MSLLHOOKSTRUCT,
    key: &'static Key,
    transition: KeyTransition,
) -> bool {
    handle_key_event(build_mouse_event(input, key, transition), 0)
}

fn handle_mouse_x_button(input: &MSLLHOOKSTRUCT, transition: KeyTransition) -> bool {
    match build_x_button_event(input, transition) {
        Ok(event) => handle_key_event(event, 0),
        Err(e) => {
            warn!("Failed to build mouse x button event: {}", e);
            false
        }
    }
}

fn handle_mouse_wheel(input: &MSLLHOOKSTRUCT, tilt: bool) -> bool {
    let key = if tilt { &KEY_MOUSE_X } else { &KEY_WHEEL_Y };
    let d = (input.mouseData >> 16) as i16; /* do not cast to i32 to preserve sign */
    let event = build_mouse_event(input, key, KeyTransition::from(d < 0));
    handle_key_event(event, d as i32)
}

fn handle_mouse_move(input: &MSLLHOOKSTRUCT) -> bool {
    let last = unsafe { HOOK.last_mouse_position.unwrap_or_else(|| input.pt) };
    let current = input.pt;
    let dx = current.x - last.x;
    let dy = current.y - last.y;
    unsafe { HOOK.last_mouse_position = Some(current) }

    let x_event = build_mouse_event(input, &KEY_MOUSE_X, KeyTransition::from(dx > 0));
    let y_event = build_mouse_event(input, &KEY_MOUSE_Y, KeyTransition::from(dy > 0));

    handle_key_event(x_event, dx) | handle_key_event(y_event, dy)
}
