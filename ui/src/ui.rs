use crate::res::res_ids::{IDS_APP_TITLE, IDS_NO_LAYOUT};
use crate::res::RESOURCES;
use crate::rs;
use crate::settings::{AppSettings, LAYOUTS_PATH};
use crate::ui::layout_view::LayoutView;
use crate::ui::log_view::LogView;
use crate::ui::main_menu::MainMenu;
use crate::ui::main_window::MainWindow;
use crate::ui::style::display_font;
use crate::ui::test_editor::TypeTestEditor;
use crate::ui::tray::Tray;
use crate::ui_warn;
use crate::utils::{get_window_size, layout_path_from_args, raw_hwnd, set_window_size};
use crate::win_watch::WinWatcher;
use event::KeyEvent;
use keympostor::keyboard::event;
use keympostor::keyboard::hook::{KeyboardHook, WM_KEY_HOOK_NOTIFY};
use keympostor::keyboard::trigger::KeyTrigger;
use keympostor::layout::Layouts;
use log::{debug, error, warn};
use native_windows_gui as nwg;
use native_windows_gui::NativeUi;
use std::cell::RefCell;
use std::ops::Not;
use std::rc::Rc;

mod app_ui;
mod layout_view;
mod layouts_menu;
mod log_view;
mod main_menu;
mod main_window;
mod style;
mod test_editor;
mod tray;
mod utils;

#[derive(Default)]
pub(crate) struct App {
    /* Components */
    key_hook: KeyboardHook,
    win_watcher: WinWatcher,
    /* UI controls */
    window: MainWindow,
    tray: Tray,
    /* State */
    is_log_enabled: RefCell<bool>,
    layouts: RefCell<Layouts>,
    current_layout: RefCell<Option<String>>,
}

impl App {
    fn load_settings(&self) {
        let settings = AppSettings::load_default();

        self.window
            .main_menu
            .build_layouts_menu(&self.layouts.borrow());

        self.select_layout(&layout_path_from_args().or(settings.layout));

        self.is_log_enabled.replace(settings.logging_enabled);
        self.apply_log_enabled();

        self.key_hook.set_enabled(settings.processing_enabled);

        self.win_watcher.set_profiles(settings.profiles);
        self.win_watcher.set_enabled(settings.layouts_enabled);

        if let Some(position) = settings.main_window.position {
            self.window.window.set_position(position.0, position.1);
        }
        if let Some(size) = settings.main_window.size {
            set_window_size(self.window.window.handle, size);
        }
        if let Some(page) = settings.main_window.selected_page {
            self.window.tab_container.set_selected_tab(page);
        }

        debug!("Loaded settings");
    }

    fn load_layouts(&self) {
        if let Ok(layouts) = Layouts::load(LAYOUTS_PATH) {
            self.layouts.replace(layouts);
        } else {
            ui_warn!("Unable to load layouts.");
        }
    }

    fn save_settings(&self) {
        let mut settings = AppSettings::load_default();

        settings.layout = self.current_layout.borrow().to_owned();
        settings.processing_enabled = self.key_hook.is_enabled();
        settings.layouts_enabled = self.win_watcher.is_enabled();
        settings.logging_enabled = self.is_log_enabled.borrow().to_owned();
        settings.main_window.position = Some(self.window.window.position());
        settings.main_window.size = Some(get_window_size(self.window.window.handle));
        settings.main_window.selected_page = Some(self.window.tab_container.selected_tab());

        settings.save_default().unwrap_or_else(|e| {
            ui_warn!("{}", e);
        });

        debug!("Saved settings");
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
        self.window.layout_view.update_ui(layout);
        self.update_controls();
        self.save_settings();
    }

    fn update_controls(&self) {
        #[cfg(feature = "debug")]
        self.window
            .window
            .set_text(format!("{} - DEBUG", self.build_title()).as_str());

        #[cfg(not(feature = "debug"))]
        self.window.set_text(self.build_title().as_str());

        self.window.main_menu.update_ui(
            self.key_hook.is_enabled(),
            self.win_watcher.is_enabled(),
            self.is_log_enabled.borrow().to_owned(),
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

    fn show_window(&self, show: bool) {
        self.apply_log_enabled();
        self.update_controls();
        self.window.window.set_visible(show);
    }

    fn apply_log_enabled(&self) {
        self.key_hook
            .set_notify_enabled(self.is_log_enabled.borrow().to_owned());
    }

    fn run(&self) {
        self.win_watcher.init(self.window.window.handle);
        self.key_hook.init(raw_hwnd(self.window.window.handle));

        self.load_layouts();
        self.load_settings();
        self.update_controls();

        #[cfg(feature = "debug")]
        self.window.window.set_visible(true);

        nwg::dispatch_thread_events();
    }

    fn handle_event(&self, evt: nwg::Event, handle: nwg::ControlHandle) {
        self.tray.handle_event(&self, evt, handle);
        self.window.main_menu.handle_event(&self, evt, handle);
        self.win_watcher.handle_event(&self, evt, handle);
        self.window.test_editor.handle_event(&self, evt, handle);
        match evt {
            nwg::Event::OnWindowClose => {
                if &handle == &self.window.window.handle {
                    self.on_window_close()
                }
            }
            _ => {}
        }
    }

    fn handle_raw_event(&self, msg: u32, l_param: isize) {
        if msg == WM_KEY_HOOK_NOTIFY {
            self.on_key_hook_notify(KeyEvent::from_l_param(l_param));
        }
        // app.log_view.handle_raw_event(msg, l_param);
    }

    fn on_toggle_processing_enabled(&self) {
        self.key_hook.set_enabled(!self.key_hook.is_enabled());
        self.update_controls();
        self.save_settings();
    }

    fn on_toggle_logging_enabled(&self) {
        self.is_log_enabled.replace_with(|v| v.not());
        self.apply_log_enabled();
        self.update_controls();
        self.save_settings();
    }

    fn on_toggle_auto_switch_layout(&self) {
        self.win_watcher.set_enabled(!self.win_watcher.is_enabled());
        self.update_controls();
        self.save_settings();
    }

    fn on_window_close(&self) {
        self.key_hook.set_notify_enabled(false); /* temporarily disable logging while closed */
        self.update_controls();
        #[cfg(feature = "debug")]
        self.on_app_exit()
    }

    fn on_app_exit(&self) {
        self.save_settings();
        nwg::stop_thread_dispatch();
    }

    fn on_show_main_window(&self) {
        self.show_window(true);
    }

    fn on_toggle_window_visibility(&self) {
        self.show_window(!self.window.window.visible());
    }

    pub(crate) fn on_select_layout(&self, layout_name: &Option<String>) {
        self.select_layout(layout_name);
    }

    fn on_log_view_clear(&self) {
        self.window.log_view.clear();
    }

    fn on_key_hook_notify(&self, event: &KeyEvent) {
        self.window.log_view.append(&event);
        self.window
            .key_event_label
            .set_text(KeyTrigger::from(event).to_string().as_str());
    }
}

impl NativeUi<AppUi> for App {
    fn build_ui(app: App) -> Result<AppUi, nwg::NwgError> {
        AppUi::build(app)
    }
}

#[derive(Default)]
struct AppUi {
    app: Rc<App>,
    event_handler: RefCell<Option<nwg::EventHandler>>,
    raw_event_handler: RefCell<Option<nwg::RawEventHandler>>,
}

impl AppUi {
    pub(crate) fn build(mut app: App) -> Result<Self, nwg::NwgError> {
        app.window.build()?;
        app.tray.build(&app.window.window)?;

        Ok(Self {
            app: Rc::new(app),
            event_handler: Default::default(),
            raw_event_handler: Default::default(),
        })
    }

    fn setup_event_handlers(&self) {
        let app_rc = Rc::downgrade(&self.app);
        self.event_handler
            .replace(Some(nwg::full_bind_event_handler(
                &self.app.window.window.handle,
                move |evt, _evt_data, handle| {
                    // debug!("NWG: {:?} {:?} {:?}", evt, _evt_data, handle);
                    if let Some(app) = app_rc.upgrade() {
                        app.handle_event(evt, handle);
                    }
                },
            )));

        let app_rc = Rc::downgrade(&self.app);
        self.raw_event_handler.replace(Some(
            nwg::bind_raw_event_handler(
                &self.app.window.window.handle,
                0x10000,
                move |_hwnd, msg, _w_param, l_param| {
                    // debug!("NWG RAW: {:?} {:?} {:?} {:?}", _hwnd, msg, _w_param, l_param);
                    if let Some(app) = app_rc.upgrade() {
                        app.handle_raw_event(msg, l_param);
                    }
                    None
                },
            )
            .expect("Failed to bind raw event handler"),
        ));
    }

    pub(crate) fn run(&self) {
        self.setup_event_handlers();
        self.app.run()
    }
}

impl Drop for AppUi {
    fn drop(&mut self) {
        if let Some(handler) = self.event_handler.borrow().as_ref() {
            nwg::unbind_event_handler(handler);
        }
        if let Some(handler) = self.raw_event_handler.borrow().as_ref() {
            nwg::unbind_raw_event_handler(handler)
                .unwrap_or_else(|e| error!("Failed to unbind raw event handler: {}", e));
        }
    }
}

pub(crate) fn run_app() {
    nwg::init().expect("Failed to init Native Windows GUI.");
    nwg::Font::set_global_default(Some(display_font(17)));
    App::build_ui(Default::default())
        .expect("Failed to build application UI.")
        .run();
}
