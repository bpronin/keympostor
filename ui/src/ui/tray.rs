use crate::res::res_ids::{
    IDI_ICON_APP, IDI_ICON_GAME_LOCK_OFF, IDI_ICON_GAME_LOCK_ON,
};
use crate::res::{RES, RESOURCES};
use crate::ui::App;
use crate::{r_icon};
use native_windows_gui::{
    ControlHandle, Event, GlobalCursor, Menu, MenuItem, MenuSeparator, MousePressEvent, NwgError,
    TrayNotification, Window,
};

#[derive(Default)]
pub(crate) struct Tray {
    notification: TrayNotification,
    menu: Menu,
    toggle_processing_enabled_item: MenuItem,
    open_app_item: MenuItem,
    exit_app_item: MenuItem,
    separator: MenuSeparator,
}

impl Tray {
    pub(crate) fn build(&mut self, parent: &Window) -> Result<(), NwgError> {
        TrayNotification::builder()
            .parent(parent)
            .icon(Some(r_icon!(IDI_ICON_APP)))
            .tip(Some(RES.strings.tray_tip.as_str()))
            .build(&mut self.notification)?;

        Menu::builder()
            .popup(true)
            .parent(parent)
            .build(&mut self.menu)?;

        MenuItem::builder()
            .parent(&self.menu)
            .text(RES.strings.enabled.as_str())
            .build(&mut self.toggle_processing_enabled_item)?;

        MenuItem::builder()
            .text(RES.strings.open.as_str())
            .parent(&self.menu)
            .build(&mut self.open_app_item)?;

        MenuSeparator::builder()
            .parent(&self.menu)
            .build(&mut self.separator)?;

        MenuItem::builder()
            .text(RES.strings.exit.as_str())
            .parent(&self.menu)
            .build(&mut self.exit_app_item)
    }

    pub(crate) fn update_ui(&self, is_processing_enabled: bool) {
        self.toggle_processing_enabled_item
            .set_checked(is_processing_enabled);

        if is_processing_enabled {
            self.notification.set_icon(r_icon!(IDI_ICON_GAME_LOCK_ON));
        } else {
            self.notification.set_icon(r_icon!(IDI_ICON_GAME_LOCK_OFF));
        }
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
                } else if &handle == &self.toggle_processing_enabled_item {
                    app.on_toggle_processing_enabled();
                } else if &handle == &self.exit_app_item {
                    app.on_app_exit();
                }
            }
            _ => {}
        }
    }
}
