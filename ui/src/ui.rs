use crate::res::res_ids::{IDI_ICON_APP, IDS_APP_TITLE};
use crate::res::RESOURCES;
use crate::settings::AppSettings;
use crate::ui::log_view::LogView;
use crate::ui::main_menu::MainMenu;
use crate::ui::profile_view::ProfileView;
use crate::ui::tray::Tray;
use crate::ui_warn;
use crate::utils::{get_window_size, profile_path_from_args, set_window_size};
use crate::win_watch::WinWatcher;
use crate::{r_icon, rs};
use keympostor::keyboard::hook::KeyboardHook;
use keympostor::profile::{Profile, Profiles};
use log::debug;
use native_windows_gui as nwg;
use native_windows_gui::NativeUi;
use std::cell::RefCell;

mod log_view;
mod main_window;
mod main_menu;
mod profile_view;
mod profiles_menu;
mod tray;
mod utils;

#[derive(Default)]
pub(crate) struct App {
    current_profile_name: RefCell<Option<String>>,
    key_hook: KeyboardHook,
    win_watcher: WinWatcher,
    profiles: Profiles,
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
        let settings = AppSettings::load_default();

        self.select_profile(&profile_path_from_args().or(settings.profile));

        self.key_hook.set_enabled(settings.processing_enabled);
        self.log_view
            .on_processing_enabled(self.key_hook.is_enabled());

        self.win_watcher.set_rules(settings.window_profile);
        self.win_watcher
            .set_enabled(settings.window_profile_enabled);

        self.log_view
            .on_auto_switch_profile_enabled(self.win_watcher.is_enabled());
        self.key_hook.set_silent(!settings.logging_enabled);

        if let Some(position) = settings.main_window.position {
            self.window.set_position(position.0, position.1);
        }
        if let Some(size) = settings.main_window.size {
            set_window_size(self.window.handle, size);
        }
        if let Some(page) = settings.main_window.selected_page {
            self.tab_container.set_selected_tab(page);
        }
    }

    fn write_settings(&self) {
        let mut settings = AppSettings::load_default();

        settings.profile = self.current_profile_name.borrow().to_owned();
        settings.processing_enabled = self.key_hook.is_enabled();
        settings.window_profile_enabled = self.win_watcher.is_enabled();
        settings.logging_enabled = !self.key_hook.is_silent();
        settings.main_window.position = Some(self.window.position());
        settings.main_window.size = Some(get_window_size(self.window.handle));
        settings.main_window.selected_page = Some(self.tab_container.selected_tab());

        settings.save_default().unwrap_or_else(|e| {
            ui_warn!("{}", e);
        });
    }

    fn find_profile(&self, profile_name: &Option<String>) -> &Profile {
        if let Some(name) = profile_name {
            if let Some(profile) = self.profiles.items.get(name) {
                return profile;
            }
        };

        self.profiles
            .items
            .get("default")
            .expect("No default profile found.")
    }

    fn select_profile(&self, profile_name: &Option<String>) {
        let profile = self.find_profile(profile_name);

        debug!("Selected profile: {:?}", profile.name);

        self.current_profile_name
            .replace(Some(profile.name.clone()));
        self.profile_view.update_ui(profile);
        self.key_hook.apply_rules(&profile.rules);
        self.update_controls();

        self.write_settings();
    }

    fn update_controls(&self) {
        #[cfg(feature = "debug")]
        self.window
            .set_text(format!("{} - DEBUG", self.build_title()).as_str());

        #[cfg(not(feature = "debug"))]
        self.window.set_text(self.build_title().as_str());

        self.main_menu.update_ui(
            self.key_hook.is_enabled(),
            self.win_watcher.is_enabled(),
            self.key_hook.is_silent(),
            &self.current_profile_name.borrow(),
        );

        self.tray.update_ui(self.key_hook.is_enabled());
    }

    fn build_title(&self) -> String {
        match self.current_profile_name.borrow().as_ref() {
            Some(name) => format!("{} - {}", rs!(IDS_APP_TITLE), name),
            None => format!("{}", rs!(IDS_APP_TITLE)),
        }
    }

    pub(crate) fn run(&self) {
        self.win_watcher.init(self.window.handle);
        self.window.set_icon(Some(r_icon!(IDI_ICON_APP))); /* bug workaround */

        self.read_settings();
        self.update_controls();

        self.log_view.on_log_enabled(!self.key_hook.is_silent());

        #[cfg(feature = "debug")]
        self.window.set_visible(true);

        nwg::dispatch_thread_events();
    }

    pub(crate) fn on_toggle_processing_enabled(&self) {
        self.key_hook.set_enabled(!self.key_hook.is_enabled());
        self.log_view
            .on_processing_enabled(self.key_hook.is_enabled());
        self.update_controls();
        self.write_settings();
    }

    pub(crate) fn on_toggle_logging_enabled(&self) {
        self.key_hook.set_silent(!self.key_hook.is_silent());

        self.log_view.on_log_enabled(!self.key_hook.is_silent());
        self.update_controls();
        self.write_settings();
    }

    pub(crate) fn on_toggle_auto_switch_profile(&self) {
        self.win_watcher.set_enabled(!self.win_watcher.is_enabled());
        self.log_view
            .on_auto_switch_profile_enabled(self.win_watcher.is_enabled());
        self.update_controls();
        self.write_settings();
    }

    pub(crate) fn on_app_exit(&self) {
        self.write_settings();
        nwg::stop_thread_dispatch();
    }

    pub(crate) fn on_open_window(&self) {
        self.window.set_visible(true);
    }

    pub(crate) fn on_toggle_window_visibility(&self) {
        self.window.set_visible(!self.window.visible());
    }

    pub(crate) fn on_select_profile(&self, profile_name: &Option<String>) {
        self.select_profile(profile_name);
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
