use crate::res::res_ids::{IDI_ICON_APP, IDS_APP_TITLE, IDS_NO_LAYOUT};
use crate::res::RESOURCES;
use crate::settings::{AppSettings, LAYOUTS_PATH};
use crate::ui::layout_view::LayoutView;
use crate::ui::log_view::LogView;
use crate::ui::main_menu::MainMenu;
use crate::ui::tray::Tray;
use crate::ui_warn;
use crate::utils::{get_window_size, layout_path_from_args, raw_hwnd, set_window_size};
use crate::win_watch::WinWatcher;
use crate::{r_icon, rs};
use keympostor::keyboard::hook::KeyboardHook;
use keympostor::layout::Layouts;
use log::{debug, warn};
use native_windows_gui as nwg;
use native_windows_gui::NativeUi;
use std::cell::RefCell;

mod layout_view;
mod layouts_menu;
mod log_view;
mod main_menu;
mod main_window;
mod tray;
mod utils;

#[derive(Default)]
pub(crate) struct App {
    current_layout: RefCell<Option<String>>,
    layouts: RefCell<Layouts>,
    key_hook: KeyboardHook,
    win_watcher: WinWatcher,
    window: nwg::Window,
    layout: nwg::FlexboxLayout,
    tab_log_layout: nwg::FlexboxLayout,
    tab_layouts_layout: nwg::FlexboxLayout,
    text_editor: nwg::TextInput,
    tab_container: nwg::TabsContainer,
    tab_log: nwg::Tab,
    tab_layouts: nwg::Tab,
    log_view: LogView,
    layout_view: LayoutView,
    main_menu: MainMenu,
    tray: Tray,
}

impl App {
    fn read_settings(&self) {
        if let Ok(layouts) = Layouts::load(LAYOUTS_PATH) {
            self.layouts.replace(layouts);
        } else {
            ui_warn!("Unable to load layouts.");
        }

        let settings = AppSettings::load_default();

        self.main_menu.build_layouts_menu(&self.layouts.borrow());

        self.select_layout(&layout_path_from_args().or(settings.layout));

        self.key_hook.set_notify_enabled(settings.logging_enabled);
        self.key_hook.set_enabled(settings.processing_enabled);

        self.win_watcher.set_profiles(settings.profiles);
        self.win_watcher.set_enabled(settings.layouts_enabled);

        self.log_view
            .add_processing_enabled(self.key_hook.is_enabled());

        self.log_view
            .add_auto_switch_layout_enabled(self.win_watcher.is_enabled());

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

        settings.layout = self.current_layout.borrow().to_owned();
        settings.processing_enabled = self.key_hook.is_enabled();
        settings.layouts_enabled = self.win_watcher.is_enabled();
        settings.logging_enabled = self.key_hook.is_notify_enabled();
        settings.main_window.position = Some(self.window.position());
        settings.main_window.size = Some(get_window_size(self.window.handle));
        settings.main_window.selected_page = Some(self.tab_container.selected_tab());

        settings.save_default().unwrap_or_else(|e| {
            ui_warn!("{}", e);
        });
    }

    fn select_layout(&self, layout_name: &Option<String>) {
        let Some(name) = layout_name else {
            warn!("Empty layout name");
            return;
        };

        let layouts = self.layouts.borrow();
        let Some(layout) = layouts.get(name) else {
            warn!("No layouts found");
            return;
        };

        debug!("Selected layout: {:?}", layout.name);

        self.key_hook.apply_rules(&layout.rules);
        self.current_layout.replace(Some(layout.name.clone()));
        self.layout_view.update_ui(layout);
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
            self.key_hook.is_notify_enabled(),
            &self.current_layout.borrow(),
        );

        self.tray.update_ui(self.key_hook.is_enabled());
    }

    fn build_title(&self) -> String {
        match self.current_layout.borrow().as_ref() {
            Some(name) => format!("{} - {}", rs!(IDS_APP_TITLE), name),
            None => format!("{} - {}", rs!(IDS_APP_TITLE), rs!(IDS_NO_LAYOUT)),
        }
    }

    pub(crate) fn run(&self) {
        self.win_watcher.init(self.window.handle);
        self.key_hook.init(raw_hwnd(self.window.handle));
        self.window.set_icon(Some(r_icon!(IDI_ICON_APP))); /* bug workaround */

        self.read_settings();
        self.update_controls();

        self.log_view.add_logging_enabled(self.key_hook.is_notify_enabled());

        #[cfg(feature = "debug")]
        self.window.set_visible(true);

        nwg::dispatch_thread_events();
    }

    pub(crate) fn on_toggle_processing_enabled(&self) {
        self.key_hook.set_enabled(!self.key_hook.is_enabled());

        self.log_view
            .add_processing_enabled(self.key_hook.is_enabled());
        self.update_controls();
        self.write_settings();
    }

    pub(crate) fn on_toggle_logging_enabled(&self) {
        self.key_hook.set_notify_enabled(!self.key_hook.is_notify_enabled());

        self.log_view.add_logging_enabled(self.key_hook.is_notify_enabled());
        self.update_controls();
        self.write_settings();
    }

    pub(crate) fn on_toggle_auto_switch_layout(&self) {
        self.win_watcher.set_enabled(!self.win_watcher.is_enabled());

        self.log_view
            .add_auto_switch_layout_enabled(self.win_watcher.is_enabled());
        self.update_controls();
        self.write_settings();
    }

    pub(crate) fn on_app_exit(&self) {
        self.write_settings();
        nwg::stop_thread_dispatch();
    }

    pub(crate) fn on_show_main_window(&self) {
        self.window.set_visible(true);
    }

    pub(crate) fn on_toggle_window_visibility(&self) {
        self.window.set_visible(!self.window.visible());
    }

    pub(crate) fn on_select_layout(&self, layout_name: &Option<String>) {
        self.select_layout(layout_name);
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
