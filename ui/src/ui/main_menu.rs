use crate::layout::{KeyTransformLayout, KeyTransformLayouts};
use crate::res::res_ids::{IDS_CLEAR_LOG, IDS_EXIT, IDS_FILE, IDS_LOGGING_ENABLED};
use crate::res::RESOURCES;
use crate::rs;
use crate::ui::layouts_menu::LayoutsMenu;
use crate::ui::App;
use log::warn;
use native_windows_gui::{ControlHandle, Event, Menu, MenuItem, MenuSeparator, NwgError, Window};

#[derive(Default)]
pub(crate) struct MainMenu {
    menu: Menu,
    layout_menu: LayoutsMenu,
    toggle_logging_enabled_item: MenuItem,
    clear_log_item: MenuItem,
    separator: MenuSeparator,
    exit_app_item: MenuItem,
}

impl MainMenu {
    pub(crate) fn build(&mut self, parent: &Window) -> Result<(), NwgError> {
        Menu::builder()
            .parent(parent)
            .text(rs!(IDS_FILE))
            .build(&mut self.menu)?;

        self.layout_menu.build(parent)?;

        MenuSeparator::builder()
            .parent(&self.menu)
            .build(&mut self.separator)?;

        MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_LOGGING_ENABLED))
            .build(&mut self.toggle_logging_enabled_item)?;

        MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_CLEAR_LOG))
            .build(&mut self.clear_log_item)?;

        MenuSeparator::builder()
            .parent(&self.menu)
            .build(&mut self.separator)?;

        MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_EXIT))
            .build(&mut self.exit_app_item)
    }

    pub(crate) fn update_ui(
        &self,
        is_auto_switch_layout_enabled: bool,
        is_logging_enabled: bool,
        current_layout: &KeyTransformLayout,
    ) {
        self.toggle_logging_enabled_item
            .set_checked(is_logging_enabled);
        self.layout_menu
            .update_ui(is_auto_switch_layout_enabled, current_layout);
    }

    pub(crate) fn build_layouts_menu(&self, layouts: &KeyTransformLayouts) {
        self.layout_menu.build_items(layouts).unwrap_or_else(|e| {
            warn!("Failed to build layouts menu: {}", e);
        });
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        match evt {
            Event::OnMenuItemSelected => {
                if &handle == &self.clear_log_item {
                    app.on_log_view_clear();
                } else if &handle == &self.exit_app_item {
                    app.on_app_exit();
                } else if &handle == &self.toggle_logging_enabled_item {
                    app.on_toggle_logging_enabled();
                }
            }
            _ => {}
        };

        self.layout_menu.handle_event(app, evt, handle);
    }
}
