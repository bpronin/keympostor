use log::warn;
use std::ptr::null_mut;
use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};
use windows::Win32::Storage::FileSystem::SYNCHRONIZE;
use windows::Win32::System::Threading::CreateMutexExA;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, GetKeyboardLayout, HKL, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

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

pub(crate) fn get_lock_state(vk: VIRTUAL_KEY) -> bool {
    unsafe { (GetKeyState(vk.0 as i32) & 1) != 0 }
}

#[cfg(test)]
pub mod tests {
    #[macro_export]
    macro_rules! str {
        ($str:literal) => {
            String::from($str)
        };
    }
}
