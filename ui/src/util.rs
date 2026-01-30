use log::warn;
use std::ptr::null_mut;
use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows::Win32::Globalization::GetLocaleInfoW;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};
use windows::Win32::Storage::FileSystem::SYNCHRONIZE;
use windows::Win32::System::Threading::CreateMutexExA;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, GetKeyboardLayout, HKL, VK_NUMLOCK,
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
        ).unwrap();

        handle.is_invalid() || GetLastError() == ERROR_ALREADY_EXISTS
    }
}

pub(crate) fn get_keyboard_locale(keyboard_layout: HKL) -> String {
    let lang_id = (keyboard_layout.0 as u32) & 0xFFFF;

    let get_locale_info = |lc_type: u32| -> String {
        unsafe {
            let buffer_size = GetLocaleInfoW(lang_id, lc_type, None) as usize;
            let mut buffer = vec![0u16; buffer_size];
            GetLocaleInfoW(lang_id, lc_type, Some(&mut buffer));
            buffer.set_len(buffer_size - 1); /* remove null terminator */
            String::from_utf16_lossy(&buffer)
        }
    };

    format!("{}_{}", get_locale_info(0x59), get_locale_info(0x5A)).to_lowercase()
}

pub(crate) fn get_current_keyboard_layout() -> HKL {
    unsafe { GetKeyboardLayout(GetWindowThreadProcessId(GetForegroundWindow(), None)) }
}

pub(crate) fn is_num_lock_on() -> bool {
    unsafe { (GetKeyState(VK_NUMLOCK.0 as i32) & 1) != 0 }
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
