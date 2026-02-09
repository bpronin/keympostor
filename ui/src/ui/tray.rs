use crate::layout::{KeyTransformLayout, KeyTransformLayoutList};
use crate::ui::res_ids::{IDI_ICON_APP, IDS_EXIT, IDS_LAYOUT, IDS_SETTINGS, IDS_TRAY_TIP};
use crate::ui::res::RESOURCES;
use crate::app::App;
use crate::{r_icon, rs};
use log::warn;
use native_windows_gui::{
    ControlHandle, Event, GlobalCursor, Icon, Menu, MenuItem, MenuSeparator, MousePressEvent,
    NwgError, TrayNotification, Window,
};
use std::cell::RefCell;

#[derive(Default)]
pub(crate) struct Tray {
    notification: TrayNotification,
    menu: Menu,
    open_app_item: MenuItem,
    exit_app_item: MenuItem,
    layouts_item: Menu,
    separator: MenuSeparator,
    layout_items: RefCell<Vec<(MenuItem, String)>>,
}

impl Tray {
    pub(crate) fn build(&mut self, parent: &Window) -> Result<(), NwgError> {
        TrayNotification::builder()
            .parent(parent)
            .icon(Some(&r_icon!(IDI_ICON_APP)))
            .tip(Some(rs!(IDS_TRAY_TIP)))
            .build(&mut self.notification)?;

        Menu::builder()
            .popup(true)
            .parent(parent)
            .build(&mut self.menu)?;

        Menu::builder()
            .text(rs!(IDS_LAYOUT))
            .parent(&self.menu)
            .build(&mut self.layouts_item)?;

        MenuSeparator::builder()
            .parent(&self.menu)
            .build(&mut self.separator)?;

        MenuItem::builder()
            .text(rs!(IDS_SETTINGS))
            .parent(&self.menu)
            .build(&mut self.open_app_item)?;

        MenuItem::builder()
            .text(rs!(IDS_EXIT))
            .parent(&self.menu)
            .build(&mut self.exit_app_item)
    }

    pub(crate) fn build_layout_menu(&self, layouts: &KeyTransformLayoutList) {
        let mut layout_items = vec![];

        for layout in layouts {
            let mut item: MenuItem = MenuItem::default();
            MenuItem::builder()
                .parent(&self.layouts_item)
                .text(&layout.title)
                .build(&mut item)
                .unwrap();

            layout_items.push((item, layout.name.clone()));
        }

        self.layout_items.replace(layout_items);


    }

    pub(crate) fn update_ui(&self, layout: &KeyTransformLayout) {
        let mut icon = r_icon!(IDI_ICON_APP);

        Icon::builder()
            .source_file(layout.icon.as_deref())
            .strict(true)
            .size(Some((16, 16)))
            .build(&mut icon)
            .unwrap_or_else(|e| {
                warn!("Failed to load layout icon: {:?}", e);
            });

        self.notification.set_icon(&icon);

        for (item, item_layout_name) in self.layout_items.borrow().iter() {
            item.set_checked(item_layout_name == &layout.name);
        }
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        match evt {
            Event::OnMousePress(MousePressEvent::MousePressLeftUp) => {
                if &handle == &self.notification {
                    app.on_toggle_window_visibility();
                }
            }
            Event::OnContextMenu => {
                if &handle == &self.notification {
                    self.on_show_menu();
                }
            }
            Event::OnMenuItemSelected => {
                if &handle == &self.open_app_item {
                    app.on_show_main_window();
                } else if &handle == &self.exit_app_item {
                    app.on_app_exit();
                } else {
                    for (item, layout_name) in self.layout_items.borrow().iter() {
                        if item.handle == handle {
                            app.on_select_layout(layout_name);
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn on_show_menu(&self) {
        let (x, y) = GlobalCursor::position();
        self.menu.popup(x, y);
    }
}
