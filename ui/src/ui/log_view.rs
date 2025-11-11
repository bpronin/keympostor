use crate::res::RESOURCES;
use crate::ui::utils::mono_font;
use crate::rs;
use keympostor::keyboard::event::KeyEvent;
use native_windows_gui as nwg;
use crate::res::res_ids::{IDS__AUTO_SWITCH_DISABLED_, IDS__AUTO_SWITCH_ENABLED_, IDS__LOGGING_DISABLED_, IDS__LOGGING_ENABLED_, IDS__PROCESSING_DISABLED_, IDS__PROCESSING_ENABLED_};

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

    pub(crate) fn view(&self) -> impl Into<nwg::ControlHandle> {
        &self.view
    }

    pub(crate) fn on_key_event(&self, event: &KeyEvent) {
        let line = format!(
            "{:1}{:1}{:1} | [{:8}] {:20}| {:22}| {:18} | {:1} | T: {:9} ",
            if event.rule.is_some() { "!" } else { "" },
            if event.is_injected { ">" } else { "" },
            if event.is_private { "<" } else { "" },
            event.modifiers.to_string_short(),
            event.action.key,
            event.action.key.virtual_key(),
            event.action.key.scan_code(),
            event.action.transition,
            event.time,
        );

        self.append_line(&line);
    }

    pub(crate) fn on_processing_enabled(&self, is_enabled: bool) {
        if is_enabled {
            self.append_line(rs!(IDS__PROCESSING_ENABLED_));
        } else {
            self.append_line(rs!(IDS__PROCESSING_DISABLED_));
        }
    }

    pub(crate) fn on_auto_switch_profile_enabled(&self, is_enabled: bool) {
        if is_enabled {
            self.append_line(rs!(IDS__AUTO_SWITCH_ENABLED_));
        } else {
            self.append_line(rs!(IDS__AUTO_SWITCH_DISABLED_));
        }
    }

    pub(crate) fn on_log_enabled(&self, is_enabled: bool) {
        if is_enabled {
            self.append_line(rs!(IDS__LOGGING_ENABLED_));
        } else {
            self.append_line(rs!(IDS__LOGGING_DISABLED_));
        }
    }

    pub(crate) fn clear(&self) {
        self.view.clear();
    }

    fn append_line(&self, s: &str) {
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
