use crate::keyboard::transform_rules::KeyTransformProfile;
use crate::ui::ui_util::mono_font;
use native_windows_gui::{NwgError, Tab, TextBox};

#[derive(Default)]
pub(crate) struct ProfileView {
    view: TextBox,
}

impl ProfileView {
    pub(crate) fn update_ui(&self, profile: &KeyTransformProfile) {
        let mut text = String::new();
        for rule in &profile.rules.items {
            text.push_str(&format!("{:22} : {}\r\n", rule.trigger, rule.actions));
        }
        self.view.set_text(&text);
    }
}

impl ProfileView {
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
}
