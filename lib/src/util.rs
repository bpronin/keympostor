#[macro_export]
macro_rules! ifd {
    ($condition:expr, $a:expr, $b:expr) => {
        if $condition {
            $a
        } else {
            $b
        }
    };
}

#[macro_export]
macro_rules! map {
    ( $( $key:expr => $val:expr ),* $(,)? ) => {{
        let mut map = ::std::collections::HashMap::new();
        $(map.insert($key, $val);)*
        map
    }};
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

// unsafe fn format_keyboard_state() -> String {
//     let mut s = String::new();
//     for i in 0..256 {
//         if KEYBOARD_STATE[i] {
//             let result = VirtualKey::from_code(i as u8).unwrap();
//             s = s + format!(" {}", result).as_str();
//         }
//     }
//     s
// }

#[cfg(test)]
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
