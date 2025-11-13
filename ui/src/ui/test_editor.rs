use crate::ui::utils::mono_font;
use crate::ui::App;
use native_windows_gui as nwg;
use native_windows_gui::{ControlHandle, Event, Window};

const MAX_LENGTH: usize = 150;

#[derive(Default)]
pub(crate) struct TypeTestEditor {
    view: nwg::TextInput,
}

impl TypeTestEditor {}

impl TypeTestEditor {
    pub(crate) fn view(&self) -> impl Into<nwg::ControlHandle> {
        &self.view
    }

    pub(crate) fn build_ui(&mut self, parent: &mut Window) -> Result<(), nwg::NwgError> {
        nwg::TextInput::builder()
            .parent(parent)
            .focus(true)
            .font(Some(&mono_font(16)))
            .build(&mut self.view)
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        match evt {
            Event::OnTextInput => {
                let text = self.view.text();
                let len = text.len();
                if len > MAX_LENGTH {
                    self.view.set_text(&text[len - MAX_LENGTH..]);
                    let l = text.len() as u32;
                    self.view.set_selection(l..l)
                }
            }
            _ => {}
        };
    }
}
