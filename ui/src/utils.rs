use native_windows_gui::ControlHandle;
use std::mem;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, SetWindowPos, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE, SWP_NOOWNERZORDER,
    SWP_NOZORDER,
};

// pub fn layout_path_from_args() -> Option<String> {
//     let mut args = env::args();
//     args.next(); /* executable name */
//     args.next()
// }

// pub fn str_fmt(template: String, args: &[String]) -> String {
//     let mut s = template.to_string();
//     for arg in args {
//         s = s.replacen("{}", arg.as_str(), 1)
//     }
//     s
// }

pub fn hwnd(handle: ControlHandle) -> HWND {
    try_hwnd(handle).expect("Failed to get HWND from control handle.")
}

pub fn try_hwnd(handle: ControlHandle) -> Option<HWND> {
    handle.hwnd().map(|h| HWND(h as _))
}

/// workaround for nwg bug
pub fn get_window_size(handle: ControlHandle) -> (u32, u32) {
    unsafe {
        let mut r: RECT = mem::zeroed();
        GetWindowRect(hwnd(handle), &mut r).unwrap();
        ((r.right - r.left) as u32, (r.bottom - r.top) as u32)
    }
}

/// workaround for nwg bug
pub fn set_window_size(handle: ControlHandle, size: (u32, u32)) {
    unsafe {
        SetWindowPos(
            hwnd(handle),
            None,
            0,
            0,
            size.0 as i32,
            size.1 as i32,
            SWP_NOZORDER | SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOOWNERZORDER,
        )
        .unwrap()
    }
}
