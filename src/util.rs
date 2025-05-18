extern crate native_windows_gui as nwg;

use crate::res::RESOURCE_STRINGS;
use crate::rs;
use std::env;

pub(crate) fn dos_line_endings(unix_text: &str) -> String {
    unix_text
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n")
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

#[macro_export]
macro_rules! assert_not {
    ($a:expr) => {
        assert!(!$a)
    };
}

#[macro_export]
macro_rules! assert_none {
    ($a:expr) => {
        assert!($a.is_none())
    };
}

#[macro_export]
macro_rules! assert_some {
    ($a:expr) => {
        assert!($a.is_some())
    };
}

#[macro_export]
macro_rules! append_prefix {
    ($s:expr, $pref:literal) => {
        if $s.starts_with($pref) {
            $s
        } else {
            &format!("{}{}", $pref, $s)
        }
    };
}

#[macro_export]
macro_rules! write_joined {
    ($dst:expr, $src:expr, $separator:expr) => {{
        let mut first = true;
        for item in $src {
            if !first {
                write!($dst, $separator)?;
            }
            write!($dst, "{}", item)?;
            first = false;
        }
        Ok(())
    }};
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

pub(crate) fn default_profile_path() -> String {
    let mut args = env::args();
    args.next(); /* executable name */
    args.next().unwrap_or("profiles/default.toml".to_string())
}
