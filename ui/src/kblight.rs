use libloading::os::windows::{Library, LOAD_WITH_ALTERED_SEARCH_PATH};
use std::sync::LazyLock;

type FnGetColors = extern "stdcall" fn(*mut ColorsData);
type FnSetColors = extern "stdcall" fn(*const ColorsData);

static DLL: LazyLock<Option<Library>> = LazyLock::new(|| unsafe {
    Library::load_with_flags("lomen.dll", LOAD_WITH_ALTERED_SEARCH_PATH).ok()
});

#[repr(C)]
#[derive(Debug, Default)]
pub struct ColorsData {
    pub right: u64,
    pub center: u64,
    pub left: u64,
    pub game: u64,
}

pub fn get_colors() -> Option<ColorsData> {
    if let Some(lib) = DLL.as_ref() {
        unsafe {
            let mut colors = ColorsData::default();

            let fun = lib.get::<FnGetColors>(b"get_colors\0").unwrap();
            fun(&mut colors);

            Some(colors)
        }
    } else {
        None
    }
}

pub fn set_colors(colors: ColorsData) {
    if let Some(lib) = DLL.as_ref() {
        unsafe {
            let fun = lib.get::<FnSetColors>(b"set_colors\0").unwrap();
            fun(&colors);
        }
    }
}
