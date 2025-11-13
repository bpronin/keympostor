use keympostor::layout::Layout;
use native_windows_gui as nwg;
use crate::ui::style::SMALL_MONO_FONT;

#[derive(Default)]
pub(crate) struct LayoutView {
    view: nwg::TextBox,
}

impl LayoutView {
    pub(crate) fn view(&self) -> impl Into<nwg::ControlHandle> {
        &self.view
    }

    pub(crate) fn build_ui(&mut self, parent: &nwg::Tab) -> Result<(), nwg::NwgError> {
        nwg::TextBox::builder()
            .parent(parent)
            .readonly(true)
            .font(Some(&SMALL_MONO_FONT))
            .build(&mut self.view)
    }

    pub(crate) fn update_ui(&self, layout: &Layout) {
        let mut text = String::new();
        text.push_str(&format!("{}\r\n", layout.title));
        text.push_str(&"-".repeat(layout.title.len()));
        text.push_str("\r\n");
        for rule in layout.rules.iter() {
            text.push_str(&format!("{:22} : {}\r\n", rule.trigger, rule.actions));
        }

        self.view.set_text(&text);
    }
}
