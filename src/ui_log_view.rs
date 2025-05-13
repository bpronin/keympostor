use crate::res::RESOURCE_STRINGS;
use crate::util::mono_font;
use crate::{rs, ui};
use keyboard::key_event::KeyEvent;
use native_windows_gui::{NwgError, Tab, TextBox};

const MAX_LOG_LINES: usize = 256;

#[derive(Default)]
pub struct LogView {
    view: TextBox,
}

impl LogView {
    pub(crate) fn build_ui(&mut self, parent: &Tab) -> Result<(), NwgError> {
        TextBox::builder()
            .parent(parent)
            .readonly(true)
            .font(Some(&mono_font(15)))
            .build(&mut self.view)
    }

    pub(crate) fn view(&self) -> &TextBox {
        &self.view
    }

    pub(crate) fn init(&self) {
        #[cfg(feature = "dev")]
        {
            self.view.appendln("--- Debug UI");
            self.view
                .appendln(&format!("--- {}", &ui::default_profile_path()));
        }
    }

    pub(crate) fn update_ui(&self, event: &KeyEvent) {
        let action = event.action();
        let key = action.key;
        let scan_code = key.scan_code();
        let virtual_key = key.virtual_key();
        let line = format!(
            "{:1}{:1}{:1} T: {:9} | {:20}| {:22}| {:18} | {:1}",
            if event.rule.is_some() { "!" } else { "" },
            if event.is_injected() { ">" } else { "" },
            if event.is_private() { "<" } else { "" },
            event.time(),
            key,
            virtual_key,
            scan_code,
            action.transition
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
