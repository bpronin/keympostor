use crate::ui::layouts_menu::LayoutsMenu;
use crate::ui::App;
use keympostor::layout::Layouts;
use log::warn;
use native_windows_gui::{ControlHandle, Event, Menu, MenuItem, MenuSeparator, NwgError, Window};
use crate::res::RES;

#[derive(Default)]
pub(crate) struct MainMenu {
    menu: Menu,
    layout_menu: LayoutsMenu,
    toggle_processing_enabled_item: MenuItem,
    toggle_logging_enabled_item: MenuItem,
    clear_log_item: MenuItem,
    separator: MenuSeparator,
    exit_app_item: MenuItem,
    toggle_auto_switch_layout_item: MenuItem,
}

impl MainMenu {
    pub(crate) fn build(&mut self, parent: &Window) -> Result<(), NwgError> {
        Menu::builder()
            .parent(parent)
            .text(RES.strings.file.as_str())
            .build(&mut self.menu)?;

        self.layout_menu.build(parent)?;

        MenuItem::builder()
            .parent(&self.menu)
            .text(RES.strings.enabled.as_str())
            .build(&mut self.toggle_processing_enabled_item)?;

        MenuItem::builder()
            .parent(&self.menu)
            .text(RES.strings.auto_switch_layout.as_str())
            .build(&mut self.toggle_auto_switch_layout_item)?;

        MenuSeparator::builder()
            .parent(&self.menu)
            .build(&mut self.separator)?;

        MenuItem::builder()
            .parent(&self.menu)
            .text(RES.strings.logging_enabled.as_str())
            .build(&mut self.toggle_logging_enabled_item)?;

        MenuItem::builder()
            .parent(&self.menu)
            .text(RES.strings.clear_log.as_str())
            .build(&mut self.clear_log_item)?;

        MenuSeparator::builder()
            .parent(&self.menu)
            .build(&mut self.separator)?;

        MenuItem::builder()
            .parent(&self.menu)
            .text(RES.strings.exit.as_str())
            .build(&mut self.exit_app_item)
    }

    pub(crate) fn update_ui(
        &self,
        is_processing_enabled: bool,
        is_auto_switch_layout_enabled: bool,
        is_logging_enabled: bool,
        current_layout_name: &Option<String>,
    ) {
        self.toggle_processing_enabled_item
            .set_checked(is_processing_enabled);
        self.toggle_auto_switch_layout_item
            .set_checked(is_auto_switch_layout_enabled);
        self.toggle_logging_enabled_item
            .set_checked(is_logging_enabled);
        self.layout_menu.update_ui(current_layout_name);
    }

    pub(crate) fn build_layouts_menu(&self, layouts: &Layouts) {
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
                } else if &handle == &self.toggle_processing_enabled_item {
                    app.on_toggle_processing_enabled();
                } else if &handle == &self.toggle_auto_switch_layout_item {
                    app.on_toggle_auto_switch_layout();
                }
            }
            _ => {}
        };

        self.layout_menu.handle_event(app, evt, handle);
    }
}
