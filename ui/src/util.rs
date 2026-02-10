use log::{debug, warn};
use regex::Regex;
use std::cell::RefCell;
use std::error::Error;
use std::ptr::null_mut;
use windows::core::{PCSTR, PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HWND, MAX_PATH};
use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};
use windows::Win32::Storage::FileSystem::SYNCHRONIZE;
use windows::Win32::System::Threading::{
    CreateMutexExA, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, GetKeyboardLayout, HKL, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
};

pub(crate) fn is_app_running() -> bool {
    const APP_MUTEX_ID: &[u8] = b"Global\\8e32f9ab-067f-0f01-8dc2-6047b7aa2a99\0";

    unsafe {
        let handle = CreateMutexExA(
            Some(null_mut()),
            PCSTR(APP_MUTEX_ID.as_ptr()),
            0,
            SYNCHRONIZE.0,
        )
        .unwrap();

        handle.is_invalid() || GetLastError() == ERROR_ALREADY_EXISTS
    }
}

pub(crate) fn play_sound(filename: &str) {
    unsafe {
        let w_filename: Vec<u16> = filename.encode_utf16().chain(std::iter::once(0)).collect();
        let result = PlaySoundW(
            PCWSTR(w_filename.as_ptr()),
            None,
            SND_FILENAME | SND_NODEFAULT | SND_ASYNC,
        );

        if !result.as_bool() {
            warn!(
                "Unable to play sound: `{}` : {:?}",
                filename,
                GetLastError()
            );
        }
    }
}

pub(crate) fn get_current_keyboard_layout() -> HKL {
    unsafe { GetKeyboardLayout(GetWindowThreadProcessId(GetForegroundWindow(), None)) }
}

pub(crate) fn get_keyboard_lock_state(vk: VIRTUAL_KEY) -> bool {
    unsafe { (GetKeyState(vk.0 as i32) & 1) != 0 }
}

thread_local! {
    static PROCESS_PATH_BUFFER: RefCell<[u16;MAX_PATH as usize]> = RefCell::new([0u16;MAX_PATH as usize]);
}

pub(crate) fn with_process_path<R>(hwnd: HWND, f: impl FnOnce(&str) -> R) -> Option<R> {
    PROCESS_PATH_BUFFER.with(|buffer_cell| unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

        let mut buffer = buffer_cell.borrow_mut();
        let mut buffer_size = buffer.len() as u32;

        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut buffer_size,
        );

        let _ = CloseHandle(handle);

        if result.is_ok() {
            let path = String::from_utf16_lossy(&buffer[..buffer_size as usize]);
            Some(f(&path))
        } else {
            None
        }
    })
}

thread_local! {
    static WINDOW_TEXT_BUFFER: RefCell<Vec<u16>> = RefCell::new(Vec::with_capacity(256));
}

pub(crate) fn with_window_title<R>(hwnd: HWND, f: impl FnOnce(&str) -> R) -> Option<R> {
    WINDOW_TEXT_BUFFER.with(|buffer_cell| unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return None;
        }

        let mut buffer = buffer_cell.borrow_mut();
        let needed = (len + 1) as usize;
        if buffer.len() < needed {
            buffer.resize(needed, 0);
        }

        let copied = GetWindowTextW(hwnd, &mut buffer);
        if copied == 0 {
            return None;
        }

        let title = String::from_utf16_lossy(&buffer[..copied as usize]);

        Some(f(&title))
    })
}

#[cfg(test)]

pub mod tests {
    #[macro_export]
    macro_rules! str {
        ($str:literal) => {
            String::from($str)
        };
    }

    #[macro_export]
    macro_rules! map {
    ( $( $key:expr => $val:expr ),* $(,)? ) => {{
        let mut map = ::std::collections::HashMap::new();
        $(map.insert($key, $val);)*
        map
    }}}
}
