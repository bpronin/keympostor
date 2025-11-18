use log::{debug, warn};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, WH_MOUSE_LL,
};

pub(crate) static mut MOUSE_HOOK: MouseHookState = {
    MouseHookState {
        handle: None,
        owner: None,
        is_notify_enabled: false,
    }
};

pub(crate) struct MouseHookState {
    pub(crate) handle: Option<HHOOK>,
    pub(crate) owner: Option<HWND>,
    pub(crate) is_notify_enabled: bool,
}

impl Drop for MouseHookState {
    fn drop(&mut self) {
        uninstall_mouse_hook();
        self.owner = None;
    }
}

extern "system" fn mouse_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    // if code == HC_ACTION as i32 {
    //     if handle_mouse_action(l_param) {
    //         return LRESULT(1);
    //     }
    // }
    debug!(
        "Mouse hook called: code = {:?}, w_param = {:?}, l_param = {:?}",
        code, w_param, l_param
    );
    unsafe { CallNextHookEx(MOUSE_HOOK.handle, code, w_param, l_param) }
}

pub(crate) fn install_mouse_hook() {
    unsafe {
        if let Some(_) = MOUSE_HOOK.handle {
            warn!("Mouse hook already installed");

            return;
        }

        match SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, 0) {
            Ok(handle) => {
                MOUSE_HOOK.handle = Some(handle);

                debug!("Mouse hook installed");
            }
            Err(e) => {
                MOUSE_HOOK.handle = None;

                warn!("Failed to install mouse hook: {}", e);
            }
        }
    }
}

pub(crate) fn uninstall_mouse_hook() {
    unsafe {
        if let Some(handle) = MOUSE_HOOK.handle {
            match UnhookWindowsHookEx(handle) {
                Ok(_) => debug!("Mouse hook uninstalled"),
                Err(e) => warn!("Failed to uninstall mouse hook: {}", e),
            }
            MOUSE_HOOK.handle = None;
        }
    }
}
