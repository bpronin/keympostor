use crate::res::RESOURCES;
use crate::rs;
use keympostor::layout::Layouts;
use native_windows_gui as nwg;
use std::cell::RefCell;

use crate::res::res_ids::IDS_LAYOUT;
use crate::ui::App;

#[derive(Default)]
pub(crate) struct LayoutsMenu {
    menu: nwg::Menu,
    items: RefCell<Vec<(nwg::MenuItem, String)>>,
}

impl LayoutsMenu {
    pub(crate) fn build(&mut self, parent: &nwg::Window) -> Result<(), nwg::NwgError> {
        nwg::Menu::builder()
            .parent(parent)
            .text(rs!(IDS_LAYOUT))
            .build(&mut self.menu)?;

        Ok(())
    }

    pub(crate) fn build_items(&self, layouts: &Layouts) -> Result<(), nwg::NwgError> {
        let mut items = vec![];

        for layout in layouts.iter() {
            let mut item: nwg::MenuItem = nwg::MenuItem::default();
            nwg::MenuItem::builder()
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

    pub(crate) fn handle_event(&self, app: &App, evt: nwg::Event, handle: nwg::ControlHandle) {
        match evt {
            nwg::Event::OnMenuItemSelected => {
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
