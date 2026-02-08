use crate::indicator::notify_layout_changed;
use crate::kb_watch::{KeyboardLayoutState, KeyboardLayoutWatcher};
use crate::layout::{KeyTransformLayout, KeyTransformLayouts};
use crate::profile::LayoutAutoswitchProfile;
use crate::settings::AppSettings;
use crate::ui::main_window::MainWindow;
use crate::ui::res::RESOURCES;
use crate::ui::res_ids::{
    IDS_APP_TITLE, IDS_FAILED_LOAD_LAYOUTS, IDS_FAILED_LOAD_SETTINGS, IDS_NO_PROFILE,
};
use crate::win_watch::WinWatcher;
use crate::{rs, show_warn_message, ui};
use keympostor::event::KeyEvent;
use keympostor::hook::KeyboardHook;
use keympostor::notify::WM_KEY_HOOK_NOTIFY;
use keympostor::trigger::KeyTrigger;
use log::debug;
use native_windows_gui::{stop_thread_dispatch, ControlHandle, Event};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Not;
use std::rc::Rc;
use ui::utils;
use utils::drain_timer_msg_queue;

#[derive(Default)]
pub(crate) struct App {
    pub(crate) window: MainWindow,
    key_hook: KeyboardHook,
    win_watcher: WinWatcher,
    keyboard_layout_watcher: KeyboardLayoutWatcher,
    is_log_enabled: RefCell<bool>,
    profiles: RefCell<Rc<HashMap<String, LayoutAutoswitchProfile>>>,
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
        let profiles = Rc::new(settings.autoswitch_profiles.unwrap_or_default());

        self.profiles.replace(Rc::clone(&profiles));
        self.win_watcher.set_profiles(profiles);

        self.select_layout(
            settings
                .transform_layout
                .unwrap_or_else(|| self.layouts.borrow().first().name.clone())
                .as_str(),
        );

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
        let layout_name = self.current_layout_name.borrow().clone();
        let profile_name = self.current_profile_name.borrow();

        match profile_name.as_deref() {
            None => {
                settings.transform_layout = Some(layout_name);
            }
            Some(name) => {
                settings
                    .autoswitch_profiles
                    .get_or_insert_default()
                    .entry(name.to_string())
                    .or_insert_with(|| LayoutAutoswitchProfile::new(layout_name));
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
        F: FnOnce(Option<&LayoutAutoswitchProfile>),
    {
        let profiles = self.profiles.borrow();
        let profile_name = self.current_profile_name.borrow();
        let profile = profile_name.as_deref().and_then(|n| profiles.get(n));
        action(profile);
    }

    pub(crate) fn with_current_layout<F>(&self, action: F)
    where
        F: FnOnce(&KeyTransformLayout),
    {
        let layouts = self.layouts.borrow();
        let layout_name = self.current_layout_name.borrow();
        let layout = layouts.find(layout_name.as_str()).expect("Layout not found.");
        action(layout);
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
            self.key_hook.set_rules(Some(&layout.rules));
            self.window.on_layout_changed(Some(layout));
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
        let profile_name = self.current_profile_name.borrow();

        self.with_current_layout(|layout| {
            self.window.update_ui(
                self.win_watcher.is_enabled(),
                *self.is_log_enabled.borrow(),
                profile_name.as_deref(),
                layout,
            );
        });
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

        let hwnd = self.window.hwnd();
        self.key_hook.install(hwnd);
        self.win_watcher.init(hwnd);
        self.keyboard_layout_watcher.start(hwnd);

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
