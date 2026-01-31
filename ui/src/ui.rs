use crate::indicator::notify_layout_changed;
use crate::kb_watch::{KeyboardLayoutState, KeyboardLayoutWatcher};
use crate::layout::{KeyTransformLayout, KeyTransformLayouts};
use crate::profile::{Profile, Profiles};
use crate::res::res_ids::{
    IDS_APP_ALREADY_RUNNING, IDS_APP_TITLE, IDS_FAILED_LOAD_LAYOUTS, IDS_FAILED_LOAD_SETTINGS,
    IDS_NO_LAYOUT, IDS_NO_PROFILE,
};
use crate::res::RESOURCES;
use crate::settings::AppSettings;
use crate::ui::layout_view::LayoutView;
use crate::ui::log_view::LogView;
use crate::ui::main_menu::MainMenu;
use crate::ui::main_window::MainWindow;
use crate::ui::style::display_font;
use crate::ui::test_editor::TypeTestEditor;
use crate::ui::tray::Tray;
use crate::ui::utils::show_warn_message;
use crate::util::is_app_running;
use crate::win_watch::WinWatcher;
use crate::{rs, show_warn_message};
use keympostor::event::KeyEvent;
use keympostor::hook::KeyboardHook;
use keympostor::notify::WM_KEY_HOOK_NOTIFY;
use keympostor::trigger::KeyTrigger;
use log::{debug, error};
use native_windows_gui as nwg;
use std::cell::RefCell;
use std::ops::Not;
use std::rc::Rc;
use utils::{get_window_size, set_window_size, try_hwnd};

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
    keyboard_layout_watcher: KeyboardLayoutWatcher,
    is_log_enabled: RefCell<bool>,
    profiles: RefCell<Rc<Profiles>>,
    layouts: RefCell<KeyTransformLayouts>,
    current_profile_name: RefCell<Option<String>>,
    current_layout_name: RefCell<Option<String>>,
    toggle_layout_hot_key: RefCell<Option<KeyTrigger>>,
}

impl App {
    fn load_settings(&self) {
        let settings = AppSettings::load_default().unwrap_or_else(|e| {
            show_warn_message!("{}:\n{}", rs!(IDS_FAILED_LOAD_SETTINGS), e);
            AppSettings::default()
        });

        self.window.apply_settings(&settings.main_window);
        let profiles = Rc::new(settings.profiles.unwrap_or_default());

        self.profiles.replace(Rc::clone(&profiles));

        self.select_layout(settings.transform_layout.as_deref());
        self.win_watcher.set_profiles(profiles);

        self.win_watcher.set_enabled(settings.profiles_enabled);
        self.is_log_enabled.replace(settings.logging_enabled);
        self.toggle_layout_hot_key
            .replace(settings.toggle_layout_hot_key);

        if let Some(key) = self.toggle_layout_hot_key.borrow().as_ref() {
            self.key_hook.suppress_keys(&[key.action.key]);
        }
    }

    fn save_settings(&self) {
        let mut settings = AppSettings::load_default().unwrap_or_default();
        let layout_name = self.current_layout_name.borrow();
        let profile_name = self.current_profile_name.borrow();

        match profile_name.as_deref() {
            None => {
                settings.transform_layout = layout_name.clone();
            }
            Some(name) => {
                settings.profiles.get_or_insert_default().get_or_insert(
                    name,
                    Profile {
                        rule: None,
                        layout: layout_name.clone(),
                    },
                );
            }
        }

        settings.profiles_enabled = self.win_watcher.is_enabled();
        settings.logging_enabled = *self.is_log_enabled.borrow();

        self.window.update_settings(&mut settings.main_window);

        settings.save_default();
    }

    fn load_layouts(&self) {
        let layouts = KeyTransformLayouts::load_default().unwrap_or_else(|e| {
            show_warn_message!("{}:\n{}", rs!(IDS_FAILED_LOAD_LAYOUTS), e);
            KeyTransformLayouts::default()
        });
        self.layouts.replace(layouts);
        self.window.set_layouts(&self.layouts.borrow());
    }

    pub(crate) fn with_current_profile<F>(&self, action: F)
    where
        F: FnOnce(Option<&Profile>),
    {
        let list = self.profiles.borrow();
        let name = self.current_profile_name.borrow();
        action(list.get(name.as_deref()));
    }

    pub(crate) fn with_current_layout<F>(&self, action: F)
    where
        F: FnOnce(Option<&KeyTransformLayout>),
    {
        let list = self.layouts.borrow();
        let name = self.current_layout_name.borrow();
        action(list.find(name.as_deref()));
    }

    pub(crate) fn select_profile(&self, name: Option<&str>) {
        self.current_profile_name.replace(name.map(Into::into));

        self.with_current_profile(|profile| {
            let layout_name = profile.and_then(|p| p.layout.as_deref());
            self.select_layout(layout_name)
        });

        debug!("Selected profile: {:?}", self.current_profile_name.borrow());
    }

    pub(crate) fn select_layout(&self, name: Option<&str>) {
        self.current_layout_name.replace(name.map(Into::into));
        self.with_current_layout(|layout| {
            self.key_hook.set_rules(layout.map(|l| &l.rules));
            self.window.on_layout_changed(layout);
            notify_layout_changed(layout, &KeyboardLayoutState::capture());
        });
        self.update_controls();

        debug!("Selected layout: {:?}", self.current_layout_name.borrow(),);
    }

    pub(crate) fn select_next_layout(&self) {
        let layouts = self.layouts.borrow();
        let next_name = {
            let current = self.current_layout_name.borrow(); /* must stay exactly inside the block */
            let next = layouts.cyclic_next(current.as_deref());
            next.and_then(|l| Some(l.name.as_str()))
        };

        debug!("Next layout: {:?}", next_name);

        self.select_layout(next_name);
    }

    fn update_controls(&self) {
        let layouts = self.layouts.borrow();
        let layout = layouts.find(self.current_layout_name.borrow().as_deref());

        self.update_title();
        self.window.update_ui(
            self.win_watcher.is_enabled(),
            *self.is_log_enabled.borrow(),
            layout,
        );
    }

    fn update_title(&self) {
        let mut title = rs!(IDS_APP_TITLE).to_string();

        match self.current_profile_name.borrow().as_deref() {
            Some(name) => title = format!("{} - {}", title, name),
            None => title = format!("{} - {}", title, rs!(IDS_NO_PROFILE)),
        };

        match self.current_layout_name.borrow().as_deref() {
            Some(name) => title = format!("{} - {}", title, name),
            None => title = format!("{} - {}", title, rs!(IDS_NO_LAYOUT)),
        };

        #[cfg(not(feature = "debug"))]
        self.window.set_title(title.as_str());

        #[cfg(feature = "debug")]
        self.window.set_title(&format!("{} - DEBUG", title));
    }

    fn show_window(&self, show: bool) {
        self.update_controls();
        self.window.set_visible(show);
    }

    fn handle_event(&self, evt: nwg::Event, handle: nwg::ControlHandle) {
        match evt {
            nwg::Event::OnInit => self.on_init(),
            _ => {}
        }
        self.win_watcher.handle_event(&self, evt, handle);
        self.keyboard_layout_watcher
            .handle_event(&self, evt, handle);
        self.window.handle_event(&self, evt, handle);
    }

    fn handle_raw_event(&self, msg: u32, l_param: isize) {
        if msg == WM_KEY_HOOK_NOTIFY {
            let param = unsafe { &*(l_param as *const KeyEvent) };
            self.on_key_hook_notify(param);
        }
    }

    fn on_init(&self) {
        self.load_layouts();
        self.load_settings();

        let window_hwnd = try_hwnd(self.window.handle());
        self.key_hook.install(window_hwnd);
        self.win_watcher.init(window_hwnd);
        self.keyboard_layout_watcher.start(window_hwnd);

        self.update_controls();

        #[cfg(feature = "debug")]
        self.window.set_visible(true);
    }

    fn on_select_layout(&self, layout_name: Option<&str>) {
        self.select_layout(layout_name);
        self.save_settings();
    }

    fn on_toggle_logging_enabled(&self) {
        self.is_log_enabled.replace_with(|v| v.not());
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
        self.update_controls();
        #[cfg(feature = "debug")]
        self.on_app_exit()
    }

    fn on_app_exit(&self) {
        self.save_settings();
        self.keyboard_layout_watcher.stop();
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
        if let Some(key) = self.toggle_layout_hot_key.borrow().as_ref() {
            if &event.as_trigger() == key {
                self.select_next_layout();
            }
        }

        if *self.is_log_enabled.borrow() {
            self.window.on_key_hook_notify(event);
        }
    }
}

impl nwg::NativeUi<AppUi> for App {
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
        if is_app_running() {
            show_warn_message(rs!(IDS_APP_ALREADY_RUNNING));
            return;
        }
        self.setup_event_handlers();
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
