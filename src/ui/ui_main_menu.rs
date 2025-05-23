use crate::res::RES;
use crate::rs;

use crate::ui::App;
use native_windows_gui as nwg;
use crate::res::res_ids::{IDS_CLEAR_LOG, IDS_ENABLED, IDS_EXIT, IDS_FILE, IDS_LOAD_PROFILE, IDS_LOGGING_ENABLED};

#[derive(Default)]
pub(crate) struct MainMenu {
    menu: nwg::Menu,
    toggle_processing_enabled_item: nwg::MenuItem,
    toggle_logging_enabled_item: nwg::MenuItem,
    clear_log_item: nwg::MenuItem,
    load_profile_item: nwg::MenuItem,
    separator: nwg::MenuSeparator,
    exit_app_item: nwg::MenuItem,
}

impl MainMenu {
    pub(crate) fn build_ui(&mut self, parent: &nwg::Window) -> Result<(), nwg::NwgError> {
        nwg::Menu::builder()
            .parent(parent)
            .text(rs!(IDS_FILE))
            .build(&mut self.menu)?;

        nwg::MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_ENABLED))
            .build(&mut self.toggle_processing_enabled_item)?;

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

        nwg::MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_LOAD_PROFILE))
            .build(&mut self.load_profile_item)?;

        nwg::MenuSeparator::builder()
            .parent(&self.menu)
            .build(&mut self.separator)?;

        nwg::MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_EXIT))
            .build(&mut self.exit_app_item)
    }

    pub(crate) fn update_ui(&self, is_processing_enabled: bool, is_silent: bool) {
        self.toggle_processing_enabled_item
            .set_checked(is_processing_enabled);

        self.toggle_logging_enabled_item.set_checked(!is_silent);
    }

    pub(crate) fn handle_event(&self, app: &App, evt: nwg::Event, handle: nwg::ControlHandle) {
        match evt {
            nwg::Event::OnMenuItemSelected => {
                if &handle == &self.load_profile_item {
                    app.on_load_profile();
                } else if &handle == &self.clear_log_item {
                    app.on_log_view_clear();
                } else if &handle == &self.exit_app_item {
                    app.on_app_exit();
                } else if &handle == &self.toggle_logging_enabled_item {
                    app.on_toggle_logging_enabled();
                } else if &handle == &self.toggle_processing_enabled_item {
                    app.on_toggle_processing_enabled();
                }
            }
            _ => {}
        };
    }
}
