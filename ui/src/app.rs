use crate::res::RESOURCES;
use std::cell::RefCell;
use std::ops::Not;
use std::rc::Rc;
use log::debug;
use native_windows_gui::{stop_thread_dispatch, ControlHandle, Event};
use keympostor::event::KeyEvent;
use keympostor::hook::KeyboardHook;
use keympostor::notify::WM_KEY_HOOK_NOTIFY;
use keympostor::trigger::KeyTrigger;
use ui::utils;
use utils::drain_timer_msg_queue;
use crate::indicator::notify_layout_changed;
use crate::kb_watch::{KeyboardLayoutState, KeyboardLayoutWatcher};
use crate::layout::{KeyTransformLayout, KeyTransformLayouts};
use crate::profile::{Profile, Profiles};
use crate::res::res_ids::{IDS_APP_TITLE, IDS_FAILED_LOAD_LAYOUTS, IDS_FAILED_LOAD_SETTINGS, IDS_NO_PROFILE};
use crate::{rs, show_warn_message, ui};
use crate::settings::AppSettings;
use crate::ui::main_window::MainWindow;
use crate::win_watch::WinWatcher;

#[derive(Default)]
pub(crate) struct App {
    pub(crate) window: MainWindow,
    key_hook: KeyboardHook,
    win_watcher: WinWatcher,
    keyboard_layout_watcher: KeyboardLayoutWatcher,
    is_log_enabled: RefCell<bool>,
    profiles: RefCell<Rc<Profiles>>,
    layouts: RefCell<KeyTransformLayouts>,
    current_profile_name: RefCell<Option<String>>,
    current_layout_name: RefCell<String>,
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

        let layout_name = settings.transform_layout.unwrap_or_else(|| {
            self.layouts.borrow().first().name.clone()
        });
        self.select_layout(layout_name.as_str());

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
                settings.transform_layout = Some(layout_name.clone());
            }
            Some(name) => {
                settings.profiles.get_or_insert_default().get_or_insert(
                    name,
                    Profile {
                        activation_rule: None,
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
        action(list.find(name.as_str()));
    }

    pub(crate) fn select_profile(&self, name: Option<&str>) {
        self.current_profile_name.replace(name.map(Into::into));

        self.with_current_profile(|profile| {
            if let Some(p) = profile {
                self.select_layout(p.layout.as_str());
            }
        });

        debug!("Selected profile: {:?}", self.current_profile_name.borrow());
    }

    pub(crate) fn select_layout(&self, layout_name: &str) {
        self.current_layout_name.replace(layout_name.to_string());
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
            let next = layouts.cyclic_next(current.as_str());
            next.name.clone()
        };

        debug!("Next layout: {:?}", next_name);

        self.select_layout(next_name.as_str());
    }

    fn update_controls(&self) {
        let layouts = self.layouts.borrow();
        let layout = layouts.find(self.current_layout_name.borrow().as_str()).unwrap();

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

        title = format!("{} - {}", title, self.current_layout_name.borrow());

        #[cfg(not(feature = "debug"))]
        self.window.set_title(title.as_str());

        #[cfg(feature = "debug")]
        self.window.set_title(&format!("{} - DEBUG", title));
    }

    fn show_window(&self, show: bool) {
        self.update_controls();
        self.window.set_visible(show);
    }

    pub(crate) fn handle_event(&self, evt: Event, handle: ControlHandle) {
        match evt {
            Event::OnInit => self.on_init(),
            _ => {}
        }
        self.win_watcher.handle_event(&self, evt, handle);
        self.keyboard_layout_watcher
            .handle_event(&self, evt, handle);
        self.window.handle_event(&self, evt, handle);
    }

    pub(crate) fn handle_raw_event(&self, msg: u32, l_param: isize) {
        if msg == WM_KEY_HOOK_NOTIFY {
            let param = unsafe { &*(l_param as *const KeyEvent) };
            self.on_key_hook_notify(param);
        }
    }

    fn on_init(&self) {
        self.load_layouts();
        self.load_settings();

        let window_hwnd = utils::try_hwnd(self.window.handle());
        self.key_hook.install(window_hwnd);
        self.win_watcher.init(window_hwnd);
        self.keyboard_layout_watcher.start(window_hwnd);

        self.update_controls();

        #[cfg(feature = "debug")]
        self.window.set_visible(true);
    }

    pub(crate) fn on_select_layout(&self, layout_name: &str) {
        self.select_layout(layout_name);
        self.save_settings();
    }

    pub(crate) fn on_toggle_logging_enabled(&self) {
        self.is_log_enabled.replace_with(|v| v.not());
        self.update_controls();
        self.save_settings();
    }

    pub(crate) fn on_toggle_auto_switch_layout(&self) {
        self.win_watcher
            .set_enabled(self.win_watcher.is_enabled().not());
        self.update_controls();
        self.save_settings();
    }

    pub(crate) fn on_window_close(&self) {
        self.update_controls();
        #[cfg(feature = "debug")]
        self.on_app_exit()
    }

    pub(crate) fn on_app_exit(&self) {
        self.save_settings();
        self.keyboard_layout_watcher.stop();
        self.win_watcher.stop();
        drain_timer_msg_queue();
        stop_thread_dispatch();
    }

    pub(crate) fn on_show_main_window(&self) {
        self.show_window(true);
    }

    pub(crate) fn on_toggle_window_visibility(&self) {
        self.show_window(!self.window.is_visible());
    }

    pub(crate) fn on_log_view_clear(&self) {
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