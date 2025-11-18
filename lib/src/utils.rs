#[macro_export]
macro_rules! ife {
    ($condition:expr, $a:expr, $b:expr) => {
        if $condition { $a } else { $b }
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

#[macro_export]
macro_rules! serialize_to_string {
    () => {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.to_string())
        }
    };
}

#[macro_export]
macro_rules! deserialize_from_string {
    () => {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            String::deserialize(deserializer)?
                .parse()
                .map_err(de::Error::custom)
        }
    };
}

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
pub(crate) mod test {
    use serde::{Deserialize, Serialize};

    /// To test serialization. TOML requires root node to be annotated
    /// as #[derive(Serialize, Deserialize)]
    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub(crate) struct SerdeWrapper<T> {
        value: T,
    }

    impl<T> SerdeWrapper<T> {
        pub(crate) fn new(value: T) -> Self {
            Self { value }
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

    // use log::LevelFilter;
    // use simple_logger::SimpleLogger;
    // pub fn setup_test_logger() {
    //     SimpleLogger::new()
    //         .with_level(LevelFilter::Debug)
    //         .init()
    //         .expect("Failed to initialize logger.");
    // }
}
