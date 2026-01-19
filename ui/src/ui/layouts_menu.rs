use crate::layout::{Layout, Layouts};
use crate::res::res_ids::{IDS_AUTO_SWITCH_LAYOUT, IDS_LAYOUT, IDS_NO_LAYOUT};
use crate::res::RESOURCES;
use crate::rs;
use crate::ui::App;
use native_windows_gui::{ControlHandle, Event, Menu, MenuItem, MenuSeparator, NwgError, Window};
use std::cell::RefCell;

#[derive(Default)]
pub(crate) struct LayoutsMenu {
    menu: Menu,
    toggle_auto_switch_layout_item: MenuItem,
    items: RefCell<Vec<(MenuItem, Option<String>)>>,
    separator: MenuSeparator,
}

impl LayoutsMenu {
    pub(crate) fn build(&mut self, parent: &Window) -> Result<(), NwgError> {
        Menu::builder()
            .parent(parent)
            .text(rs!(IDS_LAYOUT))
            .build(&mut self.menu)?;

        MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_AUTO_SWITCH_LAYOUT))
            .build(&mut self.toggle_auto_switch_layout_item)?;

        MenuSeparator::builder()
            .parent(&self.menu)
            .build(&mut self.separator)?;

        Ok(())
    }

    pub(crate) fn build_items(&self, layouts: &Layouts) -> Result<(), NwgError> {
        let mut items = vec![];

        let mut item: MenuItem = MenuItem::default();
        MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_NO_LAYOUT))
            .build(&mut item)?;

        items.push((item, None));

        for layout in layouts.iter() {
            let mut item: MenuItem = MenuItem::default();
            MenuItem::builder()
                .parent(&self.menu)
                .text(&layout.title)
                .build(&mut item)?;

            items.push((item, Some(layout.name.clone())));
        }

        self.items.replace(items);

        Ok(())
    }

    pub(crate) fn update_ui(
        &self,
        is_auto_switch_layout_enabled: bool,
        current_layout: &Option<&Layout>,
    ) {
        self.toggle_auto_switch_layout_item
            .set_checked(is_auto_switch_layout_enabled);

        let layout_name = match current_layout {
            None => None,
            Some(l) => Some(l.name.clone()),
        };

        for (item, item_layout_name) in self.items.borrow().iter() {
            item.set_checked(item_layout_name == &layout_name);
        }
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        match evt {
            Event::OnMenuItemSelected => {
                if &handle == &self.toggle_auto_switch_layout_item {
                    app.on_toggle_auto_switch_layout();
                } else {
                    for (item, layout_name) in self.items.borrow().iter() {
                        if item.handle == handle {
                            app.select_layout(&layout_name.as_ref());
                            break;
                        }
                    }
                }
            }
            _ => {}
        };
    }
}
