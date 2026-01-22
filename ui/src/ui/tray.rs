use log::warn;
use crate::layout::Layout;
use crate::res::res_ids::{IDI_ICON_APP, IDS_EXIT, IDS_OPEN, IDS_TRAY_TIP};
use crate::res::RESOURCES;
use crate::ui::App;
use crate::{r_icon, rs};
use native_windows_gui::{
    ControlHandle, Event, GlobalCursor, Icon, Menu, MenuItem, MenuSeparator, MousePressEvent,
    NwgError, TrayNotification, Window,
};

#[derive(Default)]
pub(crate) struct Tray {
    notification: TrayNotification,
    menu: Menu,
    open_app_item: MenuItem,
    exit_app_item: MenuItem,
    separator: MenuSeparator,
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

        MenuItem::builder()
            .text(rs!(IDS_OPEN))
            .parent(&self.menu)
            .build(&mut self.open_app_item)?;

        MenuSeparator::builder()
            .parent(&self.menu)
            .build(&mut self.separator)?;

        MenuItem::builder()
            .text(rs!(IDS_EXIT))
            .parent(&self.menu)
            .build(&mut self.exit_app_item)
    }

    pub(crate) fn update_ui(&self, current_layout: Option<&Layout>) {
        let mut icon = r_icon!(IDI_ICON_APP);

        if let Some(layout) = current_layout {
            Icon::builder()
                .source_file(layout.icon.as_deref())
                .strict(true)
                .size(Some((16, 16)))
                .build(&mut icon)
                .unwrap_or_else(|e| {
                    warn!("Failed to load layout icon: {:?}", e);
                })
        };

        self.notification.set_icon(&icon);
    }

    fn on_show_menu(&self) {
        let (x, y) = GlobalCursor::position();
        self.menu.popup(x, y);
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
                }
            }
            _ => {}
        }
    }
}
