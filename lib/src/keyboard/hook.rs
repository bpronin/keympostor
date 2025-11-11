use crate::ifd;
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

//todo: bug: when holding one key on numpad, press and release another, the key events are not sent
//todo: probably raw handler should be used instead of hook

static mut HOOK_HANDLE: Option<HHOOK> = None;
static mut KEYBOARD_STATE: [bool; 256] = [false; 256];
static mut TRANSFORM_MAP: Option<KeyTransformMap> = None;
static mut LISTENER: Option<Box<dyn Fn(&KeyEvent)>> = None;
static mut IS_SILENT: bool = true;

#[derive(Debug, Default)]
pub struct KeyboardHook {}

impl KeyboardHook {
    pub fn apply_rules(&self, profile: &KeyTransformRules) {
        unsafe { TRANSFORM_MAP = Some(KeyTransformMap::new(profile)) };
    }

    pub fn set_listener(&self, listener: Option<Box<dyn Fn(&KeyEvent)>>) {
        unsafe { LISTENER = listener }
    }

    pub fn is_enabled(&self) -> bool {
        unsafe {
            match HOOK_HANDLE {
                Some(_) => true,
                None => false,
            }
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        if enabled {
            install_hook()
        } else {
            uninstall_hook()
        }
    }

    pub fn is_silent(&self) -> bool {
        unsafe { IS_SILENT }
    }

    pub fn set_silent(&self, silent: bool) {
        unsafe { IS_SILENT = silent }

        debug!("Silent processing is {}", ifd!(silent, "on", "off"));
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        uninstall_hook();
        unsafe {
            TRANSFORM_MAP = None;
            LISTENER = None;
        }
    }
}

pub(crate) fn install_hook() {
    unsafe {
        if let Some(_) = HOOK_HANDLE {
            warn!("Keyboard hook already installed");
            return
        }

        match SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0) {
            Ok(handle) => {
                HOOK_HANDLE = Some(handle);
                debug!("Keyboard hook installed");
            }
            Err(e) => {
                HOOK_HANDLE = None;
                warn!("Failed to install keyboard hook: {}", e);
            }
        }
    }
}

pub(crate) fn uninstall_hook() {
    unsafe {
        if let Some(handle) = HOOK_HANDLE {
            match UnhookWindowsHookEx(handle) {
                Ok(_) => debug!("Keyboard hook uninstalled"),
                Err(e) => warn!("Failed to uninstall keyboard hook: {}", e),
            }
            HOOK_HANDLE = None;
        }
    }
}

extern "system" fn keyboard_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && handle_key_action(l_param) {
        return LRESULT(1);
    }
    unsafe { CallNextHookEx(HOOK_HANDLE, code, w_param, l_param) }
}

fn handle_key_action(l_param: LPARAM) -> bool {
    let Ok(mut event) = build_key_event(l_param) else {
        return false;
    };

    if !(event.is_injected && event.is_private) {
        unsafe {
            if let Some(ref map) = TRANSFORM_MAP {
                event.rule = map.get(&event);
            }
        }
    }

    debug!("EVENT: {}", event);

    notify_listener(&event);
    apply_transform(&event)
}

fn build_key_event<'a>(l_param: LPARAM) -> Result<KeyEvent<'a>, ()> {
    let input = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };

    let action = KeyAction::from_keyboard_input(&input);
    unsafe { KEYBOARD_STATE[action.key.vk_code as usize] = action.transition == Down }

    Ok(KeyEvent {
        action,
        modifiers: ModifierKeys::from(&unsafe { KEYBOARD_STATE }),
        is_injected: input.flags.contains(LLKHF_INJECTED),
        is_private: input.dwExtraInfo as *const u8 == SELF_EVENT_MARKER.as_ptr(),
        time: input.time,
        rule: None,
    })
}

fn apply_transform(event: &KeyEvent) -> bool {
    let Some(rule) = event.rule else { return false };

    debug!("RULE: {}", rule);

    unsafe { SendInput(&rule.actions.input, size_of::<INPUT>() as i32) };
    true
}

fn notify_listener(event: &KeyEvent) {
    unsafe {
        if IS_SILENT {
            return;
        }
        if let Some(ref listener) = LISTENER {
            listener(event)
        }
    }
}
