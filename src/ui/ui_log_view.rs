use crate::res::res_ids::{IDS__LOGGING_DISABLED_, IDS__LOGGING_ENABLED_};
use crate::res::RESOURCES;
use crate::rs;
use crate::ui::ui_util::mono_font;
use keympostor::keyboard::key_event::KeyEvent;
use keympostor::util::profile_path_from_args;
use native_windows_gui as nwg;
use native_windows_gui::ControlHandle;

const MAX_LOG_LINES: usize = 256;

#[derive(Default)]
pub(crate) struct LogView {
    view: nwg::TextBox,
}

impl LogView {
    pub(crate) fn build_ui(&mut self, parent: &nwg::Tab) -> Result<(), nwg::NwgError> {
        nwg::TextBox::builder()
            .parent(parent)
            .readonly(true)
            .font(Some(&mono_font(15)))
            .build(&mut self.view)
    }

    pub(crate) fn view(&self) -> impl Into<ControlHandle> {
        &self.view
    }

    pub(crate) fn init(&self) {
        #[cfg(feature = "dev")]
        {
            self.append_text("--- Debug UI ---");
        }
    }

    pub(crate) fn update_ui(&self, event: &KeyEvent) {
        let line = format!(
            "{:1}{:1}{:1} | [{:8}] {:20}| {:22}| {:18} | {:1} | T: {:9} ",
            if event.rule.is_some() { "!" } else { "" },
            if event.is_injected { ">" } else { "" },
            if event.is_private { "<" } else { "" },
            event.modifiers_state.to_string_short(),
            event.action.key,
            event.action.key.virtual_key(),
            event.action.key.scan_code(),
            event.action.transition,
            event.time,
        );

        self.append_text(&line);
    }

    pub(crate) fn update_log_enabled(&self, is_log_enabled: bool) {
        if is_log_enabled {
            self.append_text(rs!(IDS__LOGGING_ENABLED_));
        } else {
            self.append_text(rs!(IDS__LOGGING_DISABLED_));
        }
    }

    pub(crate) fn clear(&self) {
        self.view.clear();
    }

    pub(crate) fn append_text(&self, s: &str) {
        let text = self.view.text();

        let skip_count = text.lines().count().saturating_sub(MAX_LOG_LINES);
        let mut trimmed_text =
            text.lines()
                .skip(skip_count)
                .fold(String::new(), |mut acc, line| {
                    acc.push_str(line);
                    acc.push_str("\r\n");
                    acc
                });

        trimmed_text.push_str(s);
        trimmed_text.push_str("\r\n");

        self.view.set_text(&trimmed_text);
        self.view.scroll_lastline();
    }
}
