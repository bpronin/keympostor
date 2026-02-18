use crate::event::KeyEvent;
use crate::rules::KeyTransformRule;
use std::cell::RefCell;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;

pub const WM_KEY_HOOK_NOTIFY: u32 = 88475;

thread_local! {
    static RECEIVER: RefCell<Option<HWND>> = RefCell::new(Default::default());
}

pub struct KeyEventNotification {
    pub event: KeyEvent,
    pub rule: Option<KeyTransformRule>,
}

impl KeyEventNotification {
    fn new(event: &KeyEvent, rule: Option<KeyTransformRule>) -> Self {
        Self {
            event: event.clone(),
            rule,
        }
    }
}

pub(crate) fn install_notify_listener(owner: HWND) {
    RECEIVER.replace(Some(owner));
}

pub(crate) fn notify_key_event(event: &KeyEvent, rule: Option<&KeyTransformRule>) {
    RECEIVER.with_borrow(|receiver| {
        if receiver.is_some() {
            let notification = KeyEventNotification::new(event, rule.and_then(|r| Some(r.clone())));
            let raw_ptr = Box::into_raw(Box::new(notification)) as isize;
            unsafe {
                PostMessageW(*receiver, WM_KEY_HOOK_NOTIFY, WPARAM(0), LPARAM(raw_ptr))
                    .expect("Failed to post message")
            };
        }
    })
}
