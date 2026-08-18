use keympostor::hook::KeyboardHook;
use log::debug;
use std::cell::RefCell;
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::System::Power::{
    HPOWERNOTIFY, RegisterSuspendResumeNotification, UnregisterSuspendResumeNotification,
};
use windows::Win32::UI::WindowsAndMessaging::{DEVICE_NOTIFY_WINDOW_HANDLE, WM_POWERBROADCAST};

#[derive(Default)]
pub(crate) struct PowerWatcher {
    power_handle: RefCell<HPOWERNOTIFY>,
}

impl PowerWatcher {
    pub(crate) fn setup(&self, owner: HWND) {
        let handle = unsafe {
            RegisterSuspendResumeNotification(HANDLE::from(owner), DEVICE_NOTIFY_WINDOW_HANDLE)
        }
        .expect("Failed to register suspend/resume notification");
        self.power_handle.replace(handle);
    }

    pub(crate) fn handle_raw_event(&self, msg: u32, key_hook: &KeyboardHook) {
        match msg {
            WM_POWERBROADCAST => {
                debug!("Power event received");
                key_hook.reset();
            }
            _ => {}
        }
    }
}

impl Drop for PowerWatcher {
    fn drop(&mut self) {
        unsafe {
            let handle = self.power_handle.take();
            UnregisterSuspendResumeNotification(handle)
        }
        .expect("Failed to unregister suspend/resume notification");
    }
}
