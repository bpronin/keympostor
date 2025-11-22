use crate::profile::{Profile, Profiles};
use crate::res::RESOURCES;
use crate::res::res_ids::{IDR_SWITCH_LAYOUT, IDS_APP_TITLE, IDS_NO_LAYOUT, IDS_NO_PROFILE};
use crate::settings::{AppSettings, LAYOUTS_PATH};
use crate::ui::layout_view::LayoutView;
use crate::ui::log_view::LogView;
use crate::ui::main_menu::MainMenu;
use crate::ui::main_window::MainWindow;
use crate::ui::style::display_font;
use crate::ui::test_editor::TypeTestEditor;
use crate::ui::tray::Tray;
use crate::utils::{get_window_size, hwnd, set_window_size, try_hwnd};
use crate::win_watch::WinWatcher;
use crate::{r_snd, rs, ui_warn};
use event::KeyEvent;
use keympostor::keyboard::hook::WM_KEY_HOOK_NOTIFY;
use keympostor::keyboard::trigger::KeyTrigger;
use keympostor::keyboard::{KeyboardHook, event};
use keympostor::layout::Layouts;
use log::{debug, error};
use native_windows_gui as nwg;
use native_windows_gui::{Event, NativeUi};
use std::cell::RefCell;
use std::ops::Not;
use std::rc::Rc;

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
    window: MainWindow,
    key_hook: KeyboardHook,
    win_watcher: WinWatcher,
    is_log_enabled: RefCell<bool>,
    profiles: RefCell<Rc<Profiles>>,
    current_profile: RefCell<Option<String>>,
    layouts: RefCell<Layouts>,
    current_layout: RefCell<Option<String>>,
    default_layout: RefCell<Option<String>>,
}

impl App {
    fn load_settings(&self) {
        let settings = AppSettings::load_default();

        self.default_layout.replace(settings.layout);
        self.select_layout(&None);

        self.is_log_enabled.replace(settings.logging_enabled);
        self.apply_log_enabled();

        self.key_hook.set_enabled(settings.processing_enabled);

        self.profiles
            .replace(Rc::new(settings.profiles.unwrap_or_default()));
        self.win_watcher.set_profiles(&self.profiles.borrow());
        self.win_watcher.set_enabled(settings.layouts_enabled);

        self.window.set_position(settings.main_window.position);
        self.window.set_size(settings.main_window.size);
        self.window
            .set_selected_page(settings.main_window.selected_page);

        debug!("Loaded settings");
    }

    fn save_settings(&self) {
        let mut settings = AppSettings::load_default();

        let current_layout = self.current_layout.borrow().to_owned();
        if let Some(profile_name) = self.current_profile.borrow().as_ref() {
            settings.profiles.get_or_insert_default().get_or_insert(
                profile_name,
                Profile {
                    rule: None,
                    layout: current_layout.clone(),
                },
            );
        } else {
            settings.layout = current_layout;
        }

        settings.processing_enabled = self.key_hook.is_enabled();
        settings.layouts_enabled = self.win_watcher.is_enabled();
        settings.logging_enabled = self.is_log_enabled.borrow().to_owned();
        settings.main_window.position = Some(self.window.get_position());
        settings.main_window.size = Some(self.window.get_size());
        settings.main_window.selected_page = Some(self.window.get_selected_page());

        settings.save_default().unwrap_or_else(|e| {
            ui_warn!("{}", e);
        });

        debug!("Saved settings");
    }

    fn load_layouts(&self) {
        match Layouts::load(LAYOUTS_PATH) {
            Ok(layouts) => {
                self.layouts.replace(layouts);
            }
            Err(e) => {
                ui_warn!("Unable to load layouts. {}", e);
            }
        }
        self.window.on_load_layouts(&self.layouts.borrow());
    }

    pub(crate) fn select_profile(&self, profile_name: Option<&String>) {
        self.current_profile.replace(profile_name.cloned());

        debug!("Selected profile: {:?}", self.current_profile.borrow());

        let profiles = self.profiles.borrow();
        let profile = match self.current_profile.borrow().as_ref() {
            Some(n) => profiles.get(n),
            None => None,
        };

        match profile {
            Some(p) => self.select_layout(&p.layout),
            None => self.select_layout(&None),
        }
    }

    fn select_layout(&self, layout_name: &Option<String>) {
        let name = layout_name
            .to_owned()
            .or(self.default_layout.borrow().to_owned());

        let layouts = self.layouts.borrow();
        let layout = match name {
            Some(n) => layouts.get(&n).unwrap_or(layouts.first()),
            None => layouts.first(),
        };

        self.current_layout.replace(Some(layout.name.clone()));

        if self.current_profile.borrow().is_none() {
            self.default_layout
                .replace(self.current_layout.borrow().to_owned());
        }

        debug!(
            "Selected layout: {:?} for: {:?}. Default: {:?}",
            self.current_layout.borrow(),
            self.current_profile.borrow(),
            self.default_layout.borrow()
        );

        self.key_hook.apply_rules(&layout.rules);
        self.window.on_select_layout(layout);
        self.update_controls();
        self.save_settings();

        r_snd!(IDR_SWITCH_LAYOUT);
    }

    fn update_controls(&self) {
        self.update_title();
        self.window.update_ui(
            self.key_hook.is_enabled(),
            self.win_watcher.is_enabled(),
            self.is_log_enabled.borrow().to_owned(),
            &self.current_layout.borrow(),
        );
    }

    fn update_title(&self) {
        let mut title = rs!(IDS_APP_TITLE).to_string();
        match self.current_profile.borrow().as_ref() {
            Some(profile) => title = format!("{} - {}", title, profile),
            None => title = format!("{} - {}", title, rs!(IDS_NO_PROFILE)),
        };
        match self.current_layout.borrow().as_ref() {
            Some(layout) => title = format!("{} - {}", title, layout),
            None => title = format!("{} - {}", title, rs!(IDS_NO_LAYOUT)),
        };

        #[cfg(not(feature = "debug"))]
        self.window.set_title(title.as_str());

        #[cfg(feature = "debug")]
        self.window.set_title(&format!("{} - DEBUG", title));
    }

    fn show_window(&self, show: bool) {
        self.apply_log_enabled();
        self.update_controls();
        self.window.set_visible(show);
    }

    fn apply_log_enabled(&self) {
        self.key_hook
            .set_notify_enabled(self.is_log_enabled.borrow().to_owned());
    }

    fn handle_event(&self, evt: nwg::Event, handle: nwg::ControlHandle) {
        match evt {
            Event::OnInit => self.on_init(),
            _ => {}
        }
        self.win_watcher.handle_event(&self, evt, handle);
        self.window.handle_event(&self, evt, handle);
    }

    fn handle_raw_event(&self, msg: u32, l_param: isize) {
        if msg == WM_KEY_HOOK_NOTIFY {
            self.on_key_hook_notify(KeyEvent::from_l_param(l_param));
        }
        // app.log_view.handle_raw_event(msg, l_param);
    }

    fn on_init(&self) {
        self.win_watcher.init(hwnd(self.window.handle()));
        self.key_hook.init(try_hwnd(self.window.handle()));

        self.load_layouts();
        self.load_settings();
        self.update_controls();

        #[cfg(feature = "debug")]
        self.window.set_visible(true);
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
        self.win_watcher
            .set_enabled(self.win_watcher.is_enabled().not());
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
        self.win_watcher.stop();
        nwg::stop_thread_dispatch();
    }

    fn on_show_main_window(&self) {
        self.show_window(true);
    }

    fn on_toggle_window_visibility(&self) {
        self.show_window(!self.window.is_visible());
    }

    fn on_log_view_clear(&self) {
        self.window.clear_log();
    }

    fn on_key_hook_notify(&self, event: &KeyEvent) {
        self.window.on_key_hook_notify(event);
    }
}

impl NativeUi<AppUi> for App {
    fn build_ui(app: App) -> Result<AppUi, nwg::NwgError> {
        nwg::init().expect("Failed to init Native Windows GUI.");
        nwg::Font::set_global_default(Some(display_font(17)));
        AppUi::build(app)
    }
}

#[derive(Default)]
pub(crate) struct AppUi {
    app: Rc<App>,
    event_handler: RefCell<Option<nwg::EventHandler>>,
    raw_event_handler: RefCell<Option<nwg::RawEventHandler>>,
}

impl AppUi {
    pub(crate) fn build(mut app: App) -> Result<Self, nwg::NwgError> {
        app.window.build()?;
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
                &self.app.window.handle(),
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
                &self.app.window.handle(),
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
        // self.app.run();
        nwg::dispatch_thread_events();
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
