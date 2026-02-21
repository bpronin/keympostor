use crate::indicator::notify_layout_changed;
use crate::kb_watch::{KeyboardLayoutState, KeyboardLayoutWatcher};
use crate::layout::{KeyTransformLayout, KeyTransformLayoutList};
use crate::profile::LayoutAutoswitchProfile;
use crate::settings::AppSettings;
use crate::ui::main_window::MainWindow;
use crate::ui::res::RESOURCES;
use crate::ui::res_ids::{IDS_FAILED_LOAD_LAYOUTS, IDS_FAILED_LOAD_SETTINGS};
use crate::ui::utils::RelaxedAtomicBool;
use crate::win_watch::WindowWatcher;
use crate::{rs, show_warn_message, ui};
use keympostor::hook::KeyboardHook;
use keympostor::notify::{KeyEventNotification, WM_KEY_HOOK_NOTIFY};
use keympostor::trigger::KeyTrigger;
use log::{debug, warn};
use native_windows_gui::{stop_thread_dispatch, ControlHandle, Event};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ui::utils;
use utils::drain_timer_msg_queue;

#[derive(Default)]
pub(crate) struct App {
    pub(crate) window: MainWindow,
    key_hook: KeyboardHook,
    win_watcher: WindowWatcher,
    keyboard_layout_watcher: KeyboardLayoutWatcher,
    is_log_enabled: RelaxedAtomicBool,
    is_autoswitch_enabled: RelaxedAtomicBool,
    autoswitch_profiles: Rc<RefCell<HashMap<String, LayoutAutoswitchProfile>>>,
    layouts: RefCell<KeyTransformLayoutList>,
    current_profile_name: RefCell<Option<String>>,
    current_layout_name: RefCell<String>,
    no_profile_layout_name: RefCell<String>,
    toggle_layout_hot_key: RefCell<Option<KeyTrigger>>,
}

impl App {
    fn load_settings(&self) {
        let settings = AppSettings::load().unwrap_or_else(|e| {
            show_warn_message!("{}:\n{}", rs!(IDS_FAILED_LOAD_SETTINGS), e);
            AppSettings::default()
        });

        let layout_name = settings
            .last_transform_layout
            .unwrap_or_else(|| self.layouts.borrow().first().name.clone());
        self.apply_layout(layout_name.as_str());
        self.no_profile_layout_name.replace(layout_name);

        if let Some(la_settings) = settings.layout_autoswitch {
            *self.autoswitch_profiles.borrow_mut() = la_settings.profiles.unwrap_or_default();
            self.is_autoswitch_enabled.store(la_settings.enabled);
        };

        self.is_log_enabled.store(settings.keys_logging_enabled);

        let hot_key = settings.toggle_layout_hot_key;
        if let Some(key) = &hot_key {
            self.key_hook.suppress_keys(&[key.action.key]);
        }
        self.toggle_layout_hot_key.replace(hot_key);

        self.window.apply_settings(&settings.main_window);
    }

    fn save_settings(&self) {
        let mut settings = AppSettings::default();

        self.window.update_settings(&mut settings.main_window);
        settings.toggle_layout_hot_key = self.toggle_layout_hot_key.borrow().clone();
        settings.keys_logging_enabled = self.is_log_enabled.load();
        settings.last_transform_layout = Some(self.current_layout_name.borrow().clone());

        let autoswitch_settings = settings.layout_autoswitch.get_or_insert_default();
        autoswitch_settings.enabled = self.is_autoswitch_enabled.load();
        autoswitch_settings.profiles = Some(self.autoswitch_profiles.borrow().clone());

        settings.save();
    }

    fn load_layouts(&self) {
        let layouts = KeyTransformLayoutList::load().unwrap_or_else(|e| {
            show_warn_message!("{}:\n{}", rs!(IDS_FAILED_LOAD_LAYOUTS), e);
            KeyTransformLayoutList::default()
        });

        self.window.set_layouts(&layouts);
        self.layouts.replace(layouts);
    }

    pub(crate) fn with_current_profile<F, R>(&self, action: F) -> R
    where
        F: FnOnce(Option<&mut LayoutAutoswitchProfile>) -> R,
    {
        let profile_name = self.current_profile_name.borrow().clone();
        let mut profiles = self.autoswitch_profiles.borrow_mut();
        let profile = profile_name.and_then(|n| profiles.get_mut(&n));
        action(profile)
    }

    pub(crate) fn with_current_layout<F>(&self, action: F)
    where
        F: FnOnce(&KeyTransformLayout),
    {
        let layouts = self.layouts.borrow();
        let layout_name = self.current_layout_name.borrow();
        let layout = layouts
            .find(layout_name.as_str())
            .expect("Layout not found.");
        action(layout);
    }

    pub(crate) fn apply_layout(&self, layout_name: &str) {
        if self.layouts.borrow().find(layout_name).is_some() {
            self.current_layout_name.replace(layout_name.into());
            debug!("Selected layout: `{}`", layout_name);
        } else {
            warn!("Layout not found: `{}`", layout_name);
            return;
        }

        self.with_current_layout(|layout| {
            self.key_hook.set_rules(Some(&layout.rules));
            self.window.on_layout_changed(Some(layout));
            notify_layout_changed(layout, &KeyboardLayoutState::capture());
        });

        self.with_current_profile(|profile| match profile {
            None => {}
            Some(p) => p.transform_layout = layout_name.to_string(),
        });

        self.update_window();
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
            let param = unsafe { &*(l_param as *const KeyEventNotification) };
            self.on_key_hook_notify(param);
        }
    }

    fn update_window(&self) {
        let profile_name = self.current_profile_name.borrow();

        self.with_current_layout(|layout| {
            self.window.update_ui(
                self.is_autoswitch_enabled.load(),
                self.is_log_enabled.load(),
                profile_name.as_deref(),
                layout,
            );
        });
    }

    fn show_window(&self, show: bool) {
        self.update_window();
        self.window.set_visible(show);
    }

    fn on_init(&self) {
        self.load_layouts();
        self.load_settings();

        let hwnd = self.window.hwnd();
        self.key_hook.install(hwnd);
        self.keyboard_layout_watcher.setup(hwnd);
        self.win_watcher.setup(
            hwnd,
            self.autoswitch_profiles.borrow().clone(),
            self.is_autoswitch_enabled.load(),
        );

        self.update_window();

        #[cfg(feature = "debug")]
        self.window.set_visible(true);
    }

    pub(crate) fn on_select_profile(&self, profile_name: Option<&str>) {
        match profile_name {
            None => {
                self.current_profile_name.replace(None);
                debug!("Selected no profile");
            }
            Some(n) => {
                if self.autoswitch_profiles.borrow().get(n).is_some() {
                    self.current_profile_name.replace(Some(n.into()));
                    debug!("Selected profile: `{}`", n);
                } else {
                    warn!("Profile not found: `{}`", n);
                    return;
                }
            }
        }

        let layout_name = self.with_current_profile(|profile| match profile {
            Some(p) => p.transform_layout.clone(),
            None => self.no_profile_layout_name.borrow().clone(),
        });
        self.apply_layout(layout_name.as_str());
    }

    pub(crate) fn on_select_layout(&self, layout_name: &str) {
        self.apply_layout(layout_name);

        if self.current_profile_name.borrow().is_none() {
            self.no_profile_layout_name.replace(layout_name.to_string());
        };

        self.save_settings();
    }

    fn on_select_next_layout(&self) {
        let layouts = self.layouts.borrow();
        let next_name = {
            let current = self.current_layout_name.borrow(); /* must stay exactly inside the block */
            let next = layouts.cyclic_next(current.as_str());
            next.name.clone()
        };
        self.on_select_layout(next_name.as_str());
    }

    pub(crate) fn on_toggle_logging_enabled(&self) {
        self.is_log_enabled.toggle();
        self.update_window();
        self.save_settings();
    }

    fn on_key_hook_notify(&self, notification: &KeyEventNotification) {
        if let Some(key) = self.toggle_layout_hot_key.borrow().as_ref() {
            if &notification.event.trigger == key {
                self.on_select_next_layout();
            }
        }

        if self.is_log_enabled.load() {
            self.window.on_key_hook_notify(notification);
        }
    }

    pub(crate) fn on_toggle_auto_switch_layout(&self) {
        self.is_autoswitch_enabled.toggle();
        self.win_watcher.enable(self.is_autoswitch_enabled.load());
        self.update_window();
        self.save_settings();
    }

    pub(crate) fn on_window_close(&self) {
        self.update_window();
        #[cfg(feature = "debug")]
        self.on_app_exit()
    }

    pub(crate) fn on_app_exit(&self) {
        self.save_settings();
        self.keyboard_layout_watcher.stop();
        self.win_watcher.enable(false);
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
}
