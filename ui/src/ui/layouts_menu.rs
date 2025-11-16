use crate::res::RES;
use crate::ui::App;
use keympostor::layout::Layouts;
use native_windows_gui::{ControlHandle, Event, Menu, MenuItem, NwgError, Window};
use std::cell::RefCell;

#[derive(Default)]
pub(crate) struct LayoutsMenu {
    menu: Menu,
    items: RefCell<Vec<(MenuItem, String)>>,
}

impl LayoutsMenu {
    pub(crate) fn build(&mut self, parent: &Window) -> Result<(), NwgError> {
        Menu::builder()
            .parent(parent)
            .text(RES.strings.layout.as_str())
            .build(&mut self.menu)?;

        Ok(())
    }

    pub(crate) fn build_items(&self, layouts: &Layouts) -> Result<(), NwgError> {
        let mut items = vec![];

        for layout in layouts.iter() {
            let mut item: MenuItem = MenuItem::default();
            MenuItem::builder()
                .parent(&self.menu)
                .text(&layout.title)
                .build(&mut item)?;

            items.push((item, layout.name.clone()));
        }

        self.items.replace(items);

        Ok(())
    }

    pub(crate) fn update_ui(&self, layout_name: &Option<String>) {
        for (item, item_layout_name) in self.items.borrow().iter() {
            item.set_checked(match layout_name {
                Some(name) => item_layout_name == name,
                None => false,
            });
        }
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        match evt {
            Event::OnMenuItemSelected => {
                for (item, layout_name) in self.items.borrow().iter() {
                    if item.handle == handle {
                        app.select_layout(&Some(layout_name.to_string()));
                        break;
                    }
                }
            }
            _ => {}
        };
    }
}
