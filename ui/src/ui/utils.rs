use crate::res::{RES};
use native_windows_gui::{message, MessageButtons, MessageIcons, MessageParams};

pub(crate) fn warn(text: &str) {
    message(&MessageParams {
        title: RES.strings.app_title.as_str(),
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
