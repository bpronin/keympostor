use crate::keyboard::transform_rules::KeyTransformProfile;
use native_windows_gui::{NwgError, Tab, TextBox};
use crate::ui::ui_util::{dos_line_endings, mono_font};

#[derive(Default)]
pub(crate) struct ProfileView {
    view: TextBox,
}

impl ProfileView {
    pub(crate) fn update_ui(&self, profile: &KeyTransformProfile) {
        self.view.set_text(&dos_line_endings(&profile.to_string()));
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
