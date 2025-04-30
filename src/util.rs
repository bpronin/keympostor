use crate::rs;
use crate::RESOURCE_STRINGS;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::Win32::Media::Audio::{PlaySoundW, SND_FILENAME, SND_NODEFAULT};

pub(crate) fn warn(text: &str) {
    nwg::message(&nwg::MessageParams {
        title: rs!(app_title),
        content: text,
        buttons: nwg::MessageButtons::Ok,
        icons: nwg::MessageIcons::Warning,
    });
}

#[macro_export]
macro_rules! ui_warn {
    ($($arg:tt)*) => {
        crate::util::warn(format_str!($($arg)*));
    }
}

#[macro_export]
macro_rules! ui_panic {
    ($($arg:tt)*) => {
        nwg::fatal_message(rs!(app_title), &format!($($arg)*));
    }
}

#[macro_export]
macro_rules! format_str {
    ($($arg:tt)*) => {
        format!($($arg)*).as_str()
    }
}

pub(crate) fn slices_equal<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    a.len() == b.len() && a.len() == a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count()
}

pub(crate) fn play_sound(filename: &str) {
    let wide: Vec<u16> = OsStr::new(filename)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    if unsafe { !PlaySoundW(PCWSTR(wide.as_ptr()), None, SND_FILENAME | SND_NODEFAULT).as_bool() } {
        eprintln!("Failed to play sound {}", filename);
    }
}
