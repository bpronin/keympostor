use native_windows_gui::ControlHandle;
use std::{env, mem};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, SetWindowPos, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE, SWP_NOOWNERZORDER,
    SWP_NOZORDER,
};

pub fn profile_path_from_args() -> Option<String> {
    let mut args = env::args();
    args.next(); /* executable name */
    args.next()
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

fn hwnd(handle: ControlHandle) -> HWND {
    HWND(handle.hwnd().unwrap() as _)
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

mod test {
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
}
