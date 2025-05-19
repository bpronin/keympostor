#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use crate::res::RESOURCES;
use crate::rs;
use crate::keyboard::key_hook::KeyboardHandler;
use crate::keyboard::transform_rules::KeyTransformProfile;
use crate::settings::AppSettings;
use crate::ui::ui_log_view::LogView;
use crate::ui::ui_main_menu::MainMenu;
use crate::ui::ui_profile_view::ProfileView;
use crate::ui::ui_tray::Tray;
use crate::util::default_profile_path;
use native_windows_gui as nwg;
use native_windows_gui::NativeUi;
use crate::{ui_panic, ui_warn};

mod ui_log_view;
mod ui_main;
mod ui_main_menu;
mod ui_profile_view;
mod ui_tray;
mod ui_util;

#[derive(Default)]
pub(crate) struct App {
    keyboard_handler: KeyboardHandler,
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
        let settings = AppSettings::load().unwrap_or_else(|e| {
            ui_panic!("{}", e);
        });

        self.keyboard_handler
            .set_enabled(settings.key_processing_enabled);
        self.keyboard_handler
            .set_silent(settings.silent_key_processing);
    }

    fn write_settings(&self) {
        let mut settings = AppSettings::load().unwrap_or_else(|e| {
            ui_panic!("{}", e);
        });

        settings.key_processing_enabled = self.keyboard_handler.is_enabled();
        settings.silent_key_processing = self.keyboard_handler.is_silent();

        settings.save().unwrap_or_else(|e| {
            ui_warn!("{}", e);
        });
    }

    fn read_profile(&self, path: &str) {
        let profile = KeyTransformProfile::load(path).unwrap_or_else(|e| {
            ui_panic!("{}", e);
        });
        self.profile_view.update_ui(&profile);
        self.keyboard_handler.set_profile(profile);
    }

    fn update_controls(&self) {
        self.main_menu.update_ui(
            self.keyboard_handler.is_enabled(),
            self.keyboard_handler.is_silent(),
        );

        self.tray.update_ui(self.keyboard_handler.is_enabled());
    }

    pub(crate) fn run(&self) {
        self.read_settings();
        self.read_profile(&default_profile_path());

        self.update_controls();
        self.log_view.init();
        self.log_view
            .update_log_enabled(!self.keyboard_handler.is_silent());

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
        nwg::stop_thread_dispatch();
    }

    pub(crate) fn on_open_window(&self) {
        self.window.set_visible(true);
        // self.keyboard_handler.set_silent(false);
    }

    pub(crate) fn on_toggle_window_visibility(&self) {
        self.window.set_visible(!self.window.visible());
    }

    pub(crate) fn on_load_profile(&self) {
        let mut dialog = nwg::FileDialog::default();

        nwg::FileDialog::builder()
            .title(rs!(load_profile))
            .filters(rs!(load_profile_filter))
            .action(nwg::FileDialogAction::Open)
            .build(&mut dialog)
            .unwrap();

        if dialog.run(Some(self.window.handle)) {
            let path = dialog.get_selected_item().unwrap();
            self.read_profile(path.to_str().unwrap());
        }
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
