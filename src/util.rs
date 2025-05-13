extern crate native_windows_gui as nwg;
use crate::res::RESOURCE_STRINGS;
use crate::rs;

pub(crate) fn dos_line_endings(unix_text: &str) -> String {
    unix_text.replace("\r\n", "\n").replace('\r', "\n").replace('\n', "\r\n")
}

pub(crate) fn default_font(size: u32) -> nwg::Font {
    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .family("Segoe UI")
        .size(size)
        .build(&mut font)
        .expect("Failed to build font");
    font
}

pub(crate) fn mono_font(size: u32) -> nwg::Font {
    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .family("Consolas")
        .size(size)
        .build(&mut font)
        .expect("Failed to build font");
    font
}

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
        crate::util::warn(&format!($($arg)*));
    }
}

#[macro_export]
macro_rules! ui_panic {
    ($($arg:tt)*) => {
        nwg::fatal_message(rs!(app_title), &format!($($arg)*));
    }
}

// pub(crate) fn play_sound(filename: &str) {
//     let wide: Vec<u16> = OsStr::new(filename)
//         .encode_wide()
//         .chain(std::iter::once(0))
//         .collect();
//
//     if unsafe { !PlaySoundW(PCWSTR(wide.as_ptr()), None, SND_FILENAME | SND_NODEFAULT).as_bool() } {
//         eprintln!("Failed to play sound {}", filename);
//     }
// }
