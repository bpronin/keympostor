use crate::ui::style::BIG_MONO_FONT;
use native_windows_gui::{ControlHandle, Event, NwgError, TextInput, Window};

const MAX_LENGTH: usize = 150;

#[derive(Default)]
pub(crate) struct TypeTestEditor {
    view: TextInput,
}

impl TypeTestEditor {
    pub(crate) fn build(&mut self, parent: &Window) -> Result<(), NwgError> {
        TextInput::builder()
            .parent(parent)
            .focus(true)
            .font(Some(&BIG_MONO_FONT))
            .build(&mut self.view)
    }

    pub(crate) fn handle_event(&self, evt: Event) {
        match evt {
            Event::OnTextInput => {
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

    pub(crate) fn editor(&self) -> impl Into<ControlHandle> {
        &self.view
    }
}
