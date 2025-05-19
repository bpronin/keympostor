use crate::res::RES;
use crate::res::RESOURCE_STRINGS;
use crate::res::res_ids::{IDI_ICON_GAME_LOCK_OFF, IDI_ICON_GAME_LOCK_ON};
use crate::{r_icon, rs};
use crate::ui::ui_main::App;
use native_windows_gui as nwg;

#[derive(Default)]
pub(crate) struct Tray {
    notification: nwg::TrayNotification,
    menu: nwg::Menu,
    toggle_processing_enabled_item: nwg::MenuItem,
    open_app_item: nwg::MenuItem,
    exit_app_item: nwg::MenuItem,
    separator: nwg::MenuSeparator,
}

impl Tray {
    pub(crate) fn build_ui(&mut self, parent: &nwg::Window) -> Result<(), nwg::NwgError> {
        nwg::TrayNotification::builder()
            .parent(parent)
            .icon(Some(r_icon!(IDI_ICON_GAME_LOCK_OFF)))
            .tip(Some(rs!(tray_tip)))
            .build(&mut self.notification)?;

        nwg::Menu::builder()
            .popup(true)
            .parent(parent)
            .build(&mut self.menu)?;

        nwg::MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(enabled))
            .build(&mut self.toggle_processing_enabled_item)?;

        nwg::MenuItem::builder()
            .text(rs!(open))
            .parent(&self.menu)
            .build(&mut self.open_app_item)?;

        nwg::MenuSeparator::builder()
            .parent(&self.menu)
            .build(&mut self.separator)?;

        nwg::MenuItem::builder()
            .text(rs!(exit))
            .parent(&self.menu)
            .build(&mut self.exit_app_item)
    }

    pub(crate) fn update_ui(&self, is_processing_enabled: bool) {
        self.toggle_processing_enabled_item
            .set_checked(is_processing_enabled);

        if is_processing_enabled {
            self.notification
                .set_icon(r_icon!(IDI_ICON_GAME_LOCK_ON));
        } else {
            self.notification
                .set_icon(r_icon!(IDI_ICON_GAME_LOCK_OFF));
        }
    }

    fn on_show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.menu.popup(x, y);
    }

    pub(crate) fn handle_event(&self, app: &App, evt: nwg::Event, handle: nwg::ControlHandle) {
        match evt {
            nwg::Event::OnMousePress(nwg::MousePressEvent::MousePressLeftUp) => {
                if &handle == &self.notification {
                    app.on_toggle_window_visibility();
                }
            }
            nwg::Event::OnContextMenu => {
                if &handle == &self.notification {
                    self.on_show_menu();
                }
            }
            nwg::Event::OnMenuItemSelected => {
                if &handle == &self.open_app_item {
                    app.on_open_window();
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
