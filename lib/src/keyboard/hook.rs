use crate::keyboard::action::KeyAction;
use crate::keyboard::action::KeyTransition::Down;
use crate::keyboard::event::{KeyEvent, SELF_EVENT_MARKER};
use crate::keyboard::modifiers::ModifierKeys;
use crate::keyboard::transform::KeyTransformMap;
use log::{debug, warn};
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT};
use windows::Win32::UI::WindowsAndMessaging::*;

pub const WM_KEY_HOOK_NOTIFY: u32 = 88475;

pub(crate) static mut KEY_HOOK: KeyHookState = {
    KeyHookState {
        handle: None,
        owner: None,
        transform_map: None,
        keyboard_state: [false; 256],
        is_notify_enabled: false,
    }
};

pub(crate) struct KeyHookState {
    pub(crate) handle: Option<HHOOK>,
    pub(crate) is_notify_enabled: bool,
    pub(crate) owner: Option<HWND>,
    pub(crate) transform_map: Option<KeyTransformMap>,
    keyboard_state: [bool; 256],
}

impl Drop for KeyHookState {
    fn drop(&mut self) {
        uninstall_key_hook();
        unsafe {
            self.transform_map = None;
            self.owner = None;
        }
    }
}

extern "system" fn key_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        if handle_key_action(l_param) {
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(KEY_HOOK.handle, code, w_param, l_param) }
}

pub(crate) fn install_key_hook() {
    unsafe {
        if let Some(_) = KEY_HOOK.handle {
            warn!("Keyboard hook already installed");

            return;
        }

        match SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_hook_proc), None, 0) {
            Ok(handle) => {
                KEY_HOOK.handle = Some(handle);

                debug!("Keyboard hook installed");
            }
            Err(e) => {
                KEY_HOOK.handle = None;

                warn!("Failed to install keyboard hook: {}", e);
            }
        }
    }
}

pub(crate) fn uninstall_key_hook() {
    unsafe {
        if let Some(handle) = KEY_HOOK.handle {
            match UnhookWindowsHookEx(handle) {
                Ok(_) => debug!("Keyboard hook uninstalled"),
                Err(e) => warn!("Failed to uninstall keyboard hook: {}", e),
            }
            KEY_HOOK.handle = None;
        }
    }
}

fn handle_key_action(l_param: LPARAM) -> bool {
    let mut event = build_event(l_param);

    if !(event.is_injected && event.is_private) {
        unsafe {
            if let Some(ref map) = KEY_HOOK.transform_map {
                event.rule = map.get(&event);
            }
        }
    }

    debug!("Processing event: {}", event);

    let result = apply_transform(&event);
    notify_listener(event);

    result
}

fn build_event<'a>(l_param: LPARAM) -> KeyEvent<'a> {
    let input = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
    let action = KeyAction::from(input);

    unsafe {
        KEY_HOOK.keyboard_state[action.key.vk_code as usize] = action.transition == Down;
    }

    KeyEvent {
        action,
        modifiers: ModifierKeys::from(&unsafe { KEY_HOOK.keyboard_state }),
        is_injected: input.flags.contains(LLKHF_INJECTED),
        is_private: input.dwExtraInfo as *const u8 == SELF_EVENT_MARKER.as_ptr(),
        time: input.time,
        rule: None,
    }
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
        if !KEY_HOOK.is_notify_enabled {
            return;
        }
        if let Some(ref hwnd) = KEY_HOOK.owner {
            SendMessageW(*hwnd, WM_KEY_HOOK_NOTIFY, None, Some(event.into()));
        }
    }
}
