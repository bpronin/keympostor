use log::warn;
use regex::Regex;
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

pub(crate) fn get_process_name(hwnd: HWND) -> Result<String, Box<dyn Error>> {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return Err("Failed to get process ID".into());
        }

        let p_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)?;

        let mut buffer = [0u16; MAX_PATH as usize];
        let mut buffer_size = buffer.len() as u32;
        let success = QueryFullProcessImageNameW(
            p_handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut buffer_size,
        )
        .is_ok();

        CloseHandle(p_handle)?;

        if success {
            Ok(String::from_utf16_lossy(&buffer[..buffer_size as usize]))
        } else {
            Err("Failed to get process name".into())
        }
    }
}

pub(crate) fn get_window_title(hwnd: HWND) -> Result<String, Box<dyn Error>> {
    unsafe {
        if GetWindowTextLengthW(hwnd) == 0 {
            return Err("Invalid window handle".into());
        }

        let mut buffer = vec![0u16; (GetWindowTextLengthW(hwnd) + 1) as usize];
        let bytes_read = GetWindowTextW(hwnd, &mut buffer);
        if bytes_read == 0 {
            return Err("Failed to get window title".into());
        }

        let result = String::from_utf16_lossy(&buffer[..bytes_read as usize]);
        Ok(result)
    }
}

#[cfg(test)]

pub mod tests {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    use crate::util::{get_process_name, get_window_title};

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

    #[test]
    fn test_get_window_title() {
        let hwnd = unsafe { GetForegroundWindow() };
        
        assert!(get_window_title(hwnd).is_ok());
    }

    #[test]
    fn test_get_process_name() {
        let hwnd = unsafe { GetForegroundWindow() };

        assert!(get_process_name(hwnd).is_ok());
    }
}
