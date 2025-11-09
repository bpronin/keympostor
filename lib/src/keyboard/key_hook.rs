use crate::keyboard::key_event::KeyEvent;
use crate::keyboard::transform_map::KeyTransformMap;
use crate::keyboard::transform_rules::KeyTransformRules;
use log::{debug, warn};
use std::cell::RefCell;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardState, SendInput, INPUT};
use windows::Win32::UI::WindowsAndMessaging::*;

thread_local! {
    pub(crate) static KEY_HOOK: RefCell<KeyboardHook> = RefCell::new(KeyboardHook::default());
}

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
            if self.listener.is_some() {
                "set"
            } else {
                "removed"
            }
        );
    }

    pub(crate) fn set_silent(&mut self, silent: bool) {
        self.is_silent = silent;

        debug!("Silent processing: {silent}");
    }

    pub(crate) fn apply_rules(&mut self, profile: &KeyTransformRules) {
        self.transform_map = KeyTransformMap::new(&profile);
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
        unsafe {
            let input = *(l_param.0 as *const KBDLLHOOKSTRUCT);

            let mut keyboard_state = [0u8; 256];
            GetKeyboardState(&mut keyboard_state)
                .map_err(|e| warn!("Error getting keyboard state: {}", e))?;

            Ok(KeyEvent::new(&input, keyboard_state))
        }
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
