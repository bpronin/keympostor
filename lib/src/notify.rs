use crate::event::KeyEvent;
use std::cell::RefCell;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;

pub const WM_KEY_HOOK_NOTIFY: u32 = 88475;

thread_local! {
    static RECEIVER: RefCell<Option<HWND>> = RefCell::new(Default::default());
}

pub(crate) fn install_notify_listener(owner: Option<HWND>) {
    RECEIVER.replace(owner);
}

pub(crate) fn notify_listener(event: KeyEvent) {
    RECEIVER.with_borrow(|receiver| {
        if receiver.is_some() {
            let raw_ptr = Box::into_raw(Box::new(event)) as isize;
            unsafe {
                PostMessageW(*receiver, WM_KEY_HOOK_NOTIFY, WPARAM(0), LPARAM(raw_ptr))
                    .expect("Failed to post message")
            };
        }
    })
}
