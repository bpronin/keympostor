use crate::layout::{KeyTransformLayout, KeyTransformLayouts};
use crate::res::res_ids::{IDS_AUTO_SWITCH_LAYOUT, IDS_LAYOUT};
use crate::res::RESOURCES;
use crate::rs;
use crate::ui::App;
use native_windows_gui::{ControlHandle, Event, Menu, MenuItem, MenuSeparator, NwgError, Window};
use std::cell::RefCell;

#[derive(Default)]
pub(crate) struct LayoutsMenu {
    menu: Menu,
    toggle_auto_switch_layout_item: MenuItem,
    items: RefCell<Vec<(MenuItem, String)>>,
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

    pub(crate) fn build_items(&self, layouts: &KeyTransformLayouts) -> Result<(), NwgError> {
        let items = build_layout_items(&self.menu, layouts)?;
        self.items.replace(items);
        Ok(())
    }

    pub(crate) fn update_ui(
        &self,
        is_auto_switch_layout_enabled: bool,
        current_layout: &KeyTransformLayout,
    ) {
        self.toggle_auto_switch_layout_item
            .set_checked(is_auto_switch_layout_enabled);

        for (item, item_layout_name) in self.items.borrow().iter() {
            item.set_checked(item_layout_name == &current_layout.name);
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
                            app.on_select_layout(layout_name);
                            break;
                        }
                    }
                }
            }
            _ => {}
        };
    }
}

pub(crate) fn build_layout_items(
    parent: &Menu,
    layouts: &KeyTransformLayouts,
) -> Result<Vec<(MenuItem, String)>, NwgError> {
    let mut items = vec![];

    for layout in layouts {
        let mut item: MenuItem = MenuItem::default();
        MenuItem::builder()
            .parent(parent)
            .text(&layout.title)
            .build(&mut item)?;

        items.push((item, layout.name.clone()));
    }

    Ok(items)
}
