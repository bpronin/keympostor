use crate::res::res_ids::IDI_ICON_APP;
use crate::res::RESOURCES;
use crate::settings::AppSettings;
use crate::ui::ui_log_view::LogView;
use crate::ui::ui_main_menu::MainMenu;
use crate::ui::ui_profile_view::ProfileView;
use crate::ui::ui_tray::Tray;
use crate::ui::win_watcher::WindowWatcher;
use crate::ui_warn;
use crate::{r_icon, rs};
use keympostor::keyboard::key_hook::KeyboardHandler;
use keympostor::profile::Profile;
use native_windows_gui as nwg;
use native_windows_gui::NativeUi;
use std::cell::RefCell;
use keympostor::util::{get_window_size, profile_path_from_args, set_window_size};

mod ui_log_view;
mod ui_main;
mod ui_main_menu;
mod ui_profile_view;
mod ui_profiles_menu;
mod ui_tray;
mod ui_util;
mod win_watcher;

#[derive(Default)]
pub(crate) struct App {
    current_profile_path: RefCell<Option<String>>,
    keyboard_handler: KeyboardHandler,
    window_watcher: WindowWatcher,
    window: nwg::Window,
    layout: nwg::FlexboxLayout,
    tab_log_layout: nwg::FlexboxLayout,
    tab_profiles_layout: nwg::FlexboxLayout,
    text_editor: nwg::TextInput,
    tab_container: nwg::TabsContainer,
    tab_log: nwg::Tab,
    tab_profile: nwg::Tab,
    log_view: LogView,
    profile_view: ProfileView,
    main_menu: MainMenu,
    tray: Tray,
}

impl App {
    fn read_settings(&self) {
        let settings = AppSettings::load();

        self.read_profile(profile_path_from_args().or(settings.transform_profile));

        self.keyboard_handler
            .set_enabled(settings.key_processing_enabled);
        self.keyboard_handler
            .set_silent(settings.silent_key_processing);

        if let Some(position) = settings.main_window_position {
            self.window.set_position(position.0, position.1);
        }
        if let Some(size) = settings.main_window_size {
            set_window_size(self.window.handle, size);
        }
        if let Some(page) = settings.main_window_selected_page {
            self.tab_container.set_selected_tab(page);
        }
    }

    fn write_settings(&self) {
        let mut settings = AppSettings::load();

        settings.transform_profile = self.current_profile_path.borrow().to_owned();
        settings.key_processing_enabled = self.keyboard_handler.is_enabled();
        settings.silent_key_processing = self.keyboard_handler.is_silent();
        settings.main_window_position = Some(self.window.position());
        settings.main_window_size = Some(get_window_size(self.window.handle));
        settings.main_window_selected_page = Some(self.tab_container.selected_tab());

        settings.save().unwrap_or_else(|e| {
            ui_warn!("{}", e);
        });
    }

    fn read_profile(&self, profile_path: Option<String>) {
        let profile = if let Some(ref path) = profile_path {
            match Profile::load(&path) {
                Ok(profile) => profile,
                Err(error) => {
                    ui_warn!("{}", error);
                    return;
                }
            }
        } else {
            Default::default()
        };

        self.current_profile_path.replace(profile_path);
        self.write_settings();

        self.log_view
            .appendln(&format!("--- Loaded profile: `{}` ---", profile.title));
        self.profile_view.update_ui(&profile);
        self.keyboard_handler.apply_rules(&profile.rules);
        // self.window_watcher.apply_profile(&profile.auto_activation);

        self.update_controls();
    }

    fn update_controls(&self) {
        self.main_menu.update_ui(
            self.keyboard_handler.is_enabled(),
            self.keyboard_handler.is_silent(),
            &self.current_profile_path.borrow(),
        );

        self.tray.update_ui(self.keyboard_handler.is_enabled());
    }

    pub(crate) fn run(&self) {
        self.read_settings();
        self.update_controls();

        self.log_view.init();
        self.log_view
            .update_log_enabled(!self.keyboard_handler.is_silent());

        #[cfg(feature = "dev")]
        self.window.set_visible(true);
        self.window.set_icon(Some(r_icon!(IDI_ICON_APP))); /* bug workaround */

        nwg::dispatch_thread_events();
    }

    fn on_window_close(&self) {
        // self.keyboard_handler.set_silent(true);
        #[cfg(feature = "dev")]
        self.on_app_exit();
    }

    pub(crate) fn on_toggle_processing_enabled(&self) {
        self.keyboard_handler
            .set_enabled(!self.keyboard_handler.is_enabled());
        self.update_controls();
        self.write_settings();
    }

    pub(crate) fn on_toggle_logging_enabled(&self) {
        self.keyboard_handler
            .set_silent(!self.keyboard_handler.is_silent());

        self.log_view
            .update_log_enabled(!self.keyboard_handler.is_silent());
        self.update_controls();
        self.write_settings();
    }

    pub(crate) fn on_app_exit(&self) {
        self.write_settings();
        nwg::stop_thread_dispatch();
    }

    pub(crate) fn on_open_window(&self) {
        self.window.set_visible(true);
        // self.keyboard_handler.set_silent(false);
    }

    pub(crate) fn on_toggle_window_visibility(&self) {
        self.window.set_visible(!self.window.visible());
    }

    pub(crate) fn on_select_profile(&self, profile_path: Option<String>) {
        self.read_profile(profile_path);
    }

    pub(crate) fn on_log_view_clear(&self) {
        self.log_view.clear();
    }
}

pub(crate) fn run_app() {
    nwg::init().expect("Failed to init Native Windows GUI.");
    App::build_ui(Default::default())
        .expect("Failed to build application UI.")
        .run();
}
