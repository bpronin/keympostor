use crate::ui::style::SMALL_MONO_FONT;
use crate::layout::Layout;
use native_windows_gui::{ControlHandle, NwgError, Tab, TextBox};

#[derive(Default)]
pub(crate) struct LayoutView {
    view: TextBox,
}

impl LayoutView {
    pub(crate) fn view(&self) -> impl Into<ControlHandle> {
        &self.view
    }

    pub(crate) fn build(&mut self, parent: &Tab) -> Result<(), NwgError> {
        TextBox::builder()
            .parent(parent)
            .readonly(true)
            .font(Some(&SMALL_MONO_FONT))
            .build(&mut self.view)
    }

    pub(crate) fn update_ui(&self, layout: &Option<&Layout>) {
        let mut text = String::new();
        match layout {
            None => {
                text.push_str(&format!("{}\r\n", "NONE"));
            }
            Some(l) => {
                text.push_str(&format!("{}\r\n", l.title));
                text.push_str(&"-".repeat(l.title.len()));
                text.push_str("\r\n");
                for rule in l.rules.iter() {
                    text.push_str(&format!("{:22} : {}\r\n", rule.trigger, rule.actions));
                }
            }
        }

        self.view.set_text(&text);
    }
}
