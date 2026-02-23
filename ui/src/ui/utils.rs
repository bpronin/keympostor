use crate::rs;
use crate::ui::res::RESOURCES;
use crate::ui::res_ids::IDS_APP_TITLE;
use native_windows_gui::{
    message, ControlHandle, ListView, MessageButtons, MessageIcons, MessageParams, Window,
};
use std::mem;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::UI::Controls::{LVIF_PARAM, LVITEMW, LVM_ENSUREVISIBLE, LVM_GETCOLUMNWIDTH, LVM_INSERTITEMW, LVM_SETITEMW};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, PeekMessageW, SendMessageW, SetWindowPos, MSG, PM_REMOVE, SWP_NOACTIVATE,
    SWP_NOCOPYBITS, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOZORDER, WM_TIMER,
};

pub fn try_hwnd(handle: ControlHandle) -> Option<HWND> {
    handle.hwnd().map(|h| HWND(h as _))
}

pub fn hwnd(handle: ControlHandle) -> HWND {
    try_hwnd(handle).expect("Failed to get HWND from control handle.")
}

/// workaround for nwg bug
pub fn get_window_size(window: &Window) -> (u32, u32) {
    unsafe {
        let mut r: RECT = mem::zeroed();
        GetWindowRect(hwnd(window.handle), &mut r).unwrap();
        ((r.right - r.left) as u32, (r.bottom - r.top) as u32)
    }
}

/// workaround for nwg bug
pub fn set_window_size(window: &Window, size: (u32, u32)) {
    unsafe {
        SetWindowPos(
            hwnd(window.handle),
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

pub fn set_list_view_item_data(view: &ListView, index: usize, data: usize) {
    let mut item = LVITEMW::default();
    item.mask = LVIF_PARAM;
    item.iItem = index as i32;
    item.lParam = LPARAM(data as isize);

    unsafe {
        SendMessageW(
            hwnd(view.handle),
            LVM_SETITEMW,
            None,
            Some(LPARAM(&item as *const _ as _)),
        );
    }
}

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
        unsafe {
            SendMessageW(
                hwnd(view.handle),
                LVM_ENSUREVISIBLE,
                Some(WPARAM(len - 1)),
                None,
            );
        }
    }
}

pub(crate) fn show_warn_message(text: &str) {
    message(&MessageParams {
        title: rs!(IDS_APP_TITLE),
        content: text,
        buttons: MessageButtons::Ok,
        icons: MessageIcons::Warning,
    });
}

pub(crate) fn drain_timer_msg_queue() {
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, WM_TIMER, WM_TIMER, PM_REMOVE).as_bool() {
            /* do nothing, just drain the queue until the timers are killed */
        }
    }
}

#[macro_export]
macro_rules! show_warn_message {
    ($($arg:tt)*) => {
        crate::ui::utils::show_warn_message(&format!($($arg)*));
    }
}

#[derive(Default)]
pub struct RelaxedAtomicBool(AtomicBool);

impl RelaxedAtomicBool {
    pub fn load(&self) -> bool {
        self.0.load(Relaxed)
    }

    pub fn store(&self, val: bool) {
        self.0.store(val, Relaxed)
    }

    pub fn toggle(&self) {
        self.store(!self.load());
    }
}
