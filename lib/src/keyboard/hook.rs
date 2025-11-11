use crate::ifd;
use crate::keyboard::action::KeyAction;
use crate::keyboard::action::KeyTransition::Down;
use crate::keyboard::event::{KeyEvent, SELF_EVENT_MARKER};
use crate::keyboard::modifiers::KeyModifiersState;
use crate::keyboard::rules::KeyTransformRules;
use crate::keyboard::transform::KeyTransformMap;
use log::{debug, warn};
use std::cell::RefCell;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT};
use windows::Win32::UI::WindowsAndMessaging::*;

thread_local! {
    pub (crate) static KEY_HOOK: RefCell<KeyboardHook> = RefCell::new(KeyboardHook::default());
}

static mut KEYBOARD_STATE: [bool; 256] = [false; 256];

#[derive(Default)]
pub(crate) struct KeyboardHook {
    transform_map: KeyTransformMap,
    handle: Option<HHOOK>,
    listener: Option<Box<dyn Fn(&KeyEvent)>>,
    is_silent: bool,
}

impl KeyboardHook {
    pub(crate) fn install(&mut self) {
        match unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0) } {
            Ok(h) => {
                self.handle = Some(h);
                debug!("Keyboard hook installed");
            }
            Err(e) => {
                self.handle = None;
                warn!("Failed to install keyboard hook: {}", e);
            }
        }
    }

    pub(crate) fn uninstall(&mut self) {
        if let Some(handle) = self.handle {
            match unsafe { UnhookWindowsHookEx(handle) } {
                Ok(_) => debug!("Keyboard hook uninstalled"),
                Err(e) => warn!("Failed to uninstall keyboard hook: {}", e),
            }
            self.handle = None;
        }
    }

    pub(crate) fn set_listener(&mut self, listener: Option<Box<dyn Fn(&KeyEvent)>>) {
        self.listener = listener;

        debug!(
            "Listener {}",
            ifd!(self.listener.is_some(), "set", "removed")
        );
    }

    pub(crate) fn set_silent(&mut self, silent: bool) {
        self.is_silent = silent;

        debug!("Silent processing: {silent}");
    }

    pub(crate) fn apply_rules(&mut self, rules: &KeyTransformRules) {
        self.transform_map = KeyTransformMap::new(&rules);
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.handle.is_some()
    }

    pub(crate) fn is_silent(&self) -> bool {
        self.is_silent
    }

    fn handle_key_action(&self, l_param: LPARAM) -> bool {
        let Ok(mut event) = self.build_event(l_param) else {
            return false;
        };

        if !(event.is_injected && event.is_private) {
            event.rule = self.transform_map.get(&event);
        }

        debug!("EVENT: {}", event);

        self.notify_listener(&event);
        self.apply_transform(&event)
    }

    fn build_event(&self, l_param: LPARAM) -> Result<KeyEvent, ()> {
        let input = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };

        let action = KeyAction::from_keyboard_input(&input);
        unsafe {
            KEYBOARD_STATE[action.key.vk_code as usize] = action.transition == Down;
        }

        let event = KeyEvent {
            action,
            modifiers: KeyModifiersState::from(&unsafe { KEYBOARD_STATE }),
            rule: None,
            time: input.time,
            is_injected: input.flags.contains(LLKHF_INJECTED),
            is_private: input.dwExtraInfo as *const u8 == SELF_EVENT_MARKER.as_ptr(),
        };

        Ok(event)
    }

    fn notify_listener(&self, event: &KeyEvent) {
        if self.is_silent {
            return;
        }
        if let Some(listener) = &self.listener {
            listener(event);
        }
    }

    fn apply_transform(&self, event: &KeyEvent) -> bool {
        let Some(rule) = event.rule else {
            return false;
        };

        debug!("RULE: {}", rule);

        unsafe { SendInput(&rule.actions.input, size_of::<INPUT>() as i32) };
        true
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        self.uninstall();
    }
}

extern "system" fn keyboard_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    KEY_HOOK.with_borrow(|hook| {
        if code == HC_ACTION as i32 && hook.handle_key_action(l_param) {
            return LRESULT(1);
        }
        unsafe { CallNextHookEx(hook.handle, code, w_param, l_param) }
    })
}
