use crate::keyboard::action::KeyAction;
use crate::keyboard::action::KeyTransition::Down;
use crate::keyboard::event::{KeyEvent, SELF_EVENT_MARKER};
use crate::keyboard::modifiers::ModifierKeys;
use crate::keyboard::rules::KeyTransformRules;
use crate::keyboard::transform::KeyTransformMap;
use log::{debug, warn};
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT};
use windows::Win32::UI::WindowsAndMessaging::*;

pub const WM_KEY_HOOK_NOTIFY: u32 = 88475;

#[derive(Debug, Default)]
pub struct KeyboardHook;

impl KeyboardHook {
    pub fn init(&self, owner: Option<HWND>) {
        unsafe {
            HOOK.owner = owner;
        }
    }

    pub fn apply_rules(&self, rules: &KeyTransformRules) {
        unsafe {
            HOOK.transform_map = Some(KeyTransformMap::new(rules));
        }
    }

    pub fn is_enabled(&self) -> bool {
        match unsafe { HOOK.handle } {
            Some(_) => true,
            None => false,
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        if enabled {
            install_hook()
        } else {
            uninstall_hook()
        }
    }

    pub fn is_notify_enabled(&self) -> bool {
        unsafe { HOOK.is_notify_enabled }
    }

    pub fn set_notify_enabled(&self, enabled: bool) {
        unsafe { HOOK.is_notify_enabled = enabled }
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        self.set_enabled(false);
        unsafe {
            HOOK.transform_map = None;
            HOOK.owner = None;
        }
    }
}

const MAX_KEYS: usize = 256;

struct HookState {
    handle: Option<HHOOK>,
    owner: Option<HWND>,
    transform_map: Option<KeyTransformMap>,
    keyboard_state: [bool; MAX_KEYS],
    is_notify_enabled: bool,
}

static mut HOOK: HookState = {
    HookState {
        handle: None,
        owner: None,
        transform_map: None,
        keyboard_state: [false; 256],
        is_notify_enabled: false,
    }
};

extern "system" fn keyboard_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        if handle_key_action(l_param) {
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(HOOK.handle, code, w_param, l_param) }
}

fn install_hook() {
    unsafe {
        if let Some(_) = HOOK.handle {
            warn!("Keyboard hook already installed");

            return;
        }

        match SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0) {
            Ok(handle) => {
                HOOK.handle = Some(handle);

                debug!("Keyboard hook installed");
            }
            Err(e) => {
                HOOK.handle = None;

                warn!("Failed to install keyboard hook: {}", e);
            }
        }
    }
}

fn uninstall_hook() {
    unsafe {
        if let Some(handle) = HOOK.handle {
            match UnhookWindowsHookEx(handle) {
                Ok(_) => debug!("Keyboard hook uninstalled"),
                Err(e) => warn!("Failed to uninstall keyboard hook: {}", e),
            }
            HOOK.handle = None;
        }
    }
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
            let l_param = LPARAM(&event as *const _ as isize);
            SendMessageW(*hwnd, WM_KEY_HOOK_NOTIFY, None, Some(l_param));
        }
    }
}

pub(crate) fn is_any_key_pressed() -> bool {
    unsafe {
        for i in 0..MAX_KEYS {
            if HOOK.keyboard_state[i] {
                return true;
            }
        }
    }
    false
}

pub(crate) fn get_pressed_keys() -> Vec<usize> {
    let mut result = Vec::new();
    unsafe {
        for i in 0..MAX_KEYS {
            if HOOK.keyboard_state[i] {
                result.push(i);
            }
        }
    }
    result
}
