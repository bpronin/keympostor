use crate::res::RESOURCES;
use crate::res::res_ids::IDS_APP_TITLE;
use crate::rs;
use native_windows_gui as nwg;

pub(crate) fn warn(text: &str) {
    nwg::message(&nwg::MessageParams {
        title: rs!(IDS_APP_TITLE),
        content: text,
        buttons: nwg::MessageButtons::Ok,
        icons: nwg::MessageIcons::Warning,
    });
}

#[macro_export]
macro_rules! ui_warn {
    ($($arg:tt)*) => {
        crate::ui::utils::warn(&format!($($arg)*));
    }
}

// pub(crate) fn dos_line_endings(unix_text: &str) -> String {
//     unix_text
//         .replace("\r\n", "\n")
//         .replace('\r', "\n")
//         .replace('\n', "\r\n")
// }
