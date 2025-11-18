use crate::ui::RESOURCES;
use native_windows_gui::{message, MessageButtons, MessageIcons, MessageParams};
use crate::res::res_ids::IDS_APP_TITLE;
use crate::rs;

pub(crate) fn warn(text: &str) {
    message(&MessageParams {
        title: rs!(IDS_APP_TITLE),
        content: text,
        buttons: MessageButtons::Ok,
        icons: MessageIcons::Warning,
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
