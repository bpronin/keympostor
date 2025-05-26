use crate::keyboard::key_event::KeyEvent;
use crate::keyboard::transform_map::KeyTransformMap;
use crate::keyboard::transform_rules::KeyTransformProfile;
use log::debug;
use std::cell::RefCell;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardState, SendInput, INPUT};
use windows::Win32::UI::WindowsAndMessaging::*;

thread_local! {
    static HOOK: RefCell<KeyboardHook> = RefCell::new(KeyboardHook::default());
}

extern "system" fn keyboard_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    HOOK.with_borrow(|hook| hook.handle(code, w_param, l_param))
}

#[derive(Default)]
struct KeyboardHook {
    transform_map: KeyTransformMap,
    handle: Option<HHOOK>,
    listener: Option<Box<dyn Fn(&KeyEvent)>>,
    is_silent: bool,
}

impl KeyboardHook {
    fn install(&mut self) {
        self.handle = Some(
            unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0) }
                .expect("Failed to install keyboard hook."),
        );

        debug!("Keyboard hook installed");
    }

    fn uninstall(&mut self) {
        if let Some(handle) = self.handle {
            unsafe { UnhookWindowsHookEx(handle) }.expect("Failed to uninstall keyboard hook.");
            self.handle = None;

            debug!("Keyboard hook uninstalled");
        }
    }

    fn handle(&self, code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        if code == HC_ACTION as i32 {
            let event = self.create_event(l_param);
            self.notify_processing(&event);
            if self.process_event(event) {
                return LRESULT(1);
            }
        }
        unsafe { CallNextHookEx(self.handle, code, w_param, l_param) }
    }

    fn create_event(&self, l_param: LPARAM) -> KeyEvent {
        let mut event = unsafe {
            let mut state = [0u8; 256];
            GetKeyboardState(&mut state).unwrap();

            let input = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            KeyEvent::new(&input, state)
        };

        if !(event.is_injected && event.is_private) {
            event.rule = self.transform_map.get(&event)
        };

        debug!("EVENT: {}", event);

        event
    }

    fn process_event(&self, event: KeyEvent) -> bool {
        if let Some(rule) = event.rule {
            debug!("RULE: {}", rule);

            unsafe { SendInput(&rule.actions.input, size_of::<INPUT>() as i32) };
            true
        } else {
            false
        }
    }

    fn set_silent(&mut self, silent: bool) {
        self.is_silent = silent;

        debug!("Silent processing: {silent}");
    }

    fn set_listener(&mut self, listener: Option<Box<dyn Fn(&KeyEvent)>>) {
        self.listener = listener;
        if self.listener.is_some() {
            debug!("Listener set");
        } else {
            debug!("Listener removed");
        }
    }

    fn notify_processing(&self, event: &KeyEvent) {
        if let Some(listener) = &self.listener {
            if !&self.is_silent {
                listener(&event);
            }
        }
    }

    fn load_profile(&mut self, profile: KeyTransformProfile) {
        self.transform_map = KeyTransformMap::from_profile(profile);
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        self.uninstall();
    }
}

#[derive(Debug, Default)]
pub(crate) struct KeyboardHandler {}

impl KeyboardHandler {
    pub(crate) fn set_profile(&self, profile: KeyTransformProfile) {
        HOOK.with_borrow_mut(|hook| hook.load_profile(profile));
    }

    pub(crate) fn set_listener(&self, listener: Option<Box<dyn Fn(&KeyEvent)>>) {
        HOOK.with_borrow_mut(|hook| hook.set_listener(listener));
    }

    pub(crate) fn is_enabled(&self) -> bool {
        HOOK.with_borrow(|hook| hook.handle.is_some())
    }

    pub(crate) fn set_enabled(&self, enabled: bool) {
        HOOK.with_borrow_mut(|hook| {
            if enabled {
                hook.install()
            } else {
                hook.uninstall()
            }
        })
    }

    pub(crate) fn is_silent(&self) -> bool {
        HOOK.with_borrow(|hook| hook.is_silent)
    }

    pub(crate) fn set_silent(&self, silent: bool) {
        HOOK.with_borrow_mut(|inner| inner.set_silent(silent));
    }
}
