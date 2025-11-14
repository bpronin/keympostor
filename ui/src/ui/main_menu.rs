use crate::res::res_ids::{
    IDS_AUTO_SWITCH_LAYOUT, IDS_CLEAR_LOG, IDS_ENABLED, IDS_EXIT, IDS_FILE, IDS_LOGGING_ENABLED,
};
use crate::res::RESOURCES;
use crate::rs;
use crate::ui::layouts_menu::LayoutsMenu;
use crate::ui::App;
use keympostor::layout::Layouts;
use log::warn;
use native_windows_gui as nwg;

#[derive(Default)]
pub(crate) struct MainMenu {
    menu: nwg::Menu,
    layout_menu: LayoutsMenu,
    toggle_processing_enabled_item: nwg::MenuItem,
    toggle_logging_enabled_item: nwg::MenuItem,
    clear_log_item: nwg::MenuItem,
    separator: nwg::MenuSeparator,
    exit_app_item: nwg::MenuItem,
    toggle_auto_switch_layout_item: nwg::MenuItem,
}

impl MainMenu {
    pub(crate) fn build(&mut self, parent: &nwg::Window) -> Result<(), nwg::NwgError> {
        nwg::Menu::builder()
            .parent(parent)
            .text(rs!(IDS_FILE))
            .build(&mut self.menu)?;

        self.layout_menu.build(parent)?;

        nwg::MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_ENABLED))
            .build(&mut self.toggle_processing_enabled_item)?;

        nwg::MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_AUTO_SWITCH_LAYOUT))
            .build(&mut self.toggle_auto_switch_layout_item)?;

        nwg::MenuSeparator::builder()
            .parent(&self.menu)
            .build(&mut self.separator)?;

        nwg::MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_LOGGING_ENABLED))
            .build(&mut self.toggle_logging_enabled_item)?;

        nwg::MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_CLEAR_LOG))
            .build(&mut self.clear_log_item)?;

        nwg::MenuSeparator::builder()
            .parent(&self.menu)
            .build(&mut self.separator)?;

        nwg::MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_EXIT))
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

    pub(crate) fn handle_event(&self, app: &App, evt: nwg::Event, handle: nwg::ControlHandle) {
        match evt {
            nwg::Event::OnMenuItemSelected => {
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
