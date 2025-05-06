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