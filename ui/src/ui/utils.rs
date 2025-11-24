use crate::res::res_ids::IDS_APP_TITLE;
use crate::rs;
use crate::ui::RESOURCES;
use native_windows_gui::{
    ControlHandle, ListView, MessageButtons, MessageIcons, MessageParams, message,
};
use std::mem;
use windows::Win32::Foundation::{HWND, RECT, WPARAM};
use windows::Win32::UI::Controls::{LVM_ENSUREVISIBLE, LVM_GETCOLUMNWIDTH};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOZORDER,
    SendMessageW, SetWindowPos,
};

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

/// workaround for nwg bug
pub fn get_list_view_column_width(view: &ListView, index: usize) -> isize {
    unsafe {
        SendMessageW(
            hwnd(view.handle),
            LVM_GETCOLUMNWIDTH,
            Some(WPARAM(index)),
            None,
        )
        .0
    }
}

pub fn scroll_list_view_to_end(view: &ListView) {
    let len = view.len();
    if len > 0 {
        let hwnd = hwnd(view.handle);
        unsafe {
            SendMessageW(hwnd, LVM_ENSUREVISIBLE, Some(WPARAM(len - 1)), None);
        }
    }
}

pub(crate) fn warn(text: &str) {
    message(&MessageParams {
        title: rs!(IDS_APP_TITLE),
        content: text,
        buttons: MessageButtons::Ok,
        icons: MessageIcons::Warning,
    });
}

#[macro_export]
macro_rules! ui_warn {
    ($($arg:tt)*) => {
        crate::ui::utils::warn(&format!($($arg)*));
    }
}
