use crate::keyboard::key_event::KeyEvent;
use crate::res::RESOURCE_STRINGS;
use crate::rs;
use crate::util::{default_profile_path, mono_font};
use native_windows_gui as nwg;

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

    pub(crate) fn view(&self) -> &nwg::TextBox {
        &self.view
    }

    pub(crate) fn init(&self) {
        #[cfg(feature = "dev")]
        {
            self.view.appendln("--- Debug UI");
            self.view
                .appendln(&format!("--- {}", default_profile_path()));
        }
    }

    pub(crate) fn update_ui(&self, event: &KeyEvent) {
        let action = event.action();
        let line = format!(
            "{:1}{:1}{:1} | [{:8}] {:20}| {:22}| {:18} | {:1} | T: {:9} ",
            if event.rule.is_some() { "!" } else { "" },
            if event.is_injected() { ">" } else { "" },
            if event.is_private() { "<" } else { "" },
            event.modifiers,
            action.key,
            action.key.virtual_key(),
            action.key.scan_code(),
            action.transition,
            event.time(),
        );

        self.trim_log_text();
        self.view.appendln(&line);
    }

    pub(crate) fn update_log_enabled(&self, is_log_enabled: bool) {
        if is_log_enabled {
            self.view.appendln(rs!(_logging_enabled_));
        } else {
            self.view.appendln(rs!(_logging_disabled_));
        }
    }

    pub(crate) fn clear(&self) {
        self.view.clear();
    }

    fn trim_log_text(&self) {
        let text = self.view.text();

        let skip_count = text.lines().count().saturating_sub(MAX_LOG_LINES);
        let trimmed_text = text
            .lines()
            .skip(skip_count)
            .fold(String::new(), |mut acc, line| {
                acc.push_str(line);
                acc.push_str("\r\n");
                acc
            });

        self.view.set_text(&trimmed_text);
    }
}
