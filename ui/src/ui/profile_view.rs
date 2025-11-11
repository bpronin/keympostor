use crate::ui::utils::mono_font;
use keympostor::profile::Profile;
use native_windows_gui as nwg;

#[derive(Default)]
pub(crate) struct ProfileView {
    view: nwg::TextBox,
}

impl ProfileView {
    pub(crate) fn update_ui(&self, profile: &Profile) {
        let mut text = String::new();
        text.push_str(&format!("{}\r\n", profile.title));
        text.push_str(&"-".repeat(profile.title.len()));
        text.push_str("\r\n");
        for rule in profile.rules.iter() {
            text.push_str(&format!("{:22} : {}\r\n", rule.trigger, rule.actions));
        }

        self.view.set_text(&text);
    }
}

impl ProfileView {
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
}
