use crate::ui::App;
use native_windows_gui as nwg;
use crate::ui::style::{BIG_MONO_FONT};

const MAX_LENGTH: usize = 150;

#[derive(Default)]
pub(crate) struct TypeTestEditor {
    view: nwg::TextInput,
}

impl TypeTestEditor {
    pub(crate) fn editor(&self) -> impl Into<nwg::ControlHandle> {
        &self.view
    }

    pub(crate) fn build_ui(&mut self, parent: &nwg::Window) -> Result<(), nwg::NwgError> {
        nwg::TextInput::builder()
            .parent(parent)
            .focus(true)
            .font(Some(&BIG_MONO_FONT))
            .build(&mut self.view)
    }

    pub(crate) fn handle_event(&self, app: &App, evt: nwg::Event, handle: nwg::ControlHandle) {
        match evt {
            nwg::Event::OnTextInput => {
                let text = self.view.text();
                let len = text.len();
                if len > MAX_LENGTH {
                    self.view.set_text(&text[len - MAX_LENGTH..]);

                    let pos = len as u32;
                    self.view.set_selection(pos..pos)
                }
            }
            _ => {}
        };
    }
}
