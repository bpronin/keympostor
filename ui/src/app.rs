use crate::indicator::notify_layout_changed;
use crate::kb_watch::{KeyboardLayoutState, KeyboardLayoutWatcher};
use crate::layout::{KeyTransformLayout, TransformLayouts};
use crate::profile::{Profile, Profiles, NO_PROFILE};
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
use std::ops::DerefMut;
use std::rc::Rc;
use ui::utils;
use utils::drain_timer_msg_queue;

#[derive(Default)]
pub(crate) struct App {
    pub(crate) window: MainWindow,
    key_hook: KeyboardHook,
    win_watcher: WindowWatcher,
    keyboard_layout_watcher: KeyboardLayoutWatcher,
    settings: RefCell<AppSettings>,
    profiles: Rc<RefCell<Profiles>>,
    layouts: RefCell<TransformLayouts>,
    is_processing_enabled: RelaxedAtomicBool,
    current_profile_name: RefCell<String>,
    current_layout_name: RefCell<String>,
}

impl App {
    fn with_settings<F, R>(&self, action: F) -> R
    where
        F: FnOnce(&mut AppSettings) -> R,
    {
        action(self.settings.borrow_mut().deref_mut())
    }

    fn with_current_profile<F, R>(&self, action: F) -> R
    where
        F: FnOnce(&mut Profile) -> R,
    {
        let mut profiles = self.profiles.borrow_mut();
        let profile_name = self.current_profile_name.borrow();
        let profile = profiles.get_mut(&profile_name);
        action(profile)
    }

    fn with_current_layout<F>(&self, action: F)
    where
        F: FnOnce(&KeyTransformLayout),
    {
        let layouts = self.layouts.borrow();
        let layout_name = self.current_layout_name.borrow();
        let layout = layouts
            .find(&layout_name)
            .expect(&format!("Layout not found: `{}`", layout_name));
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
            self.key_hook.set_rules(layout.rules.as_ref());
            self.window.on_layout_changed(Some(layout));
            notify_layout_changed(layout, &KeyboardLayoutState::capture());
        });

        self.with_current_profile(|profile| profile.transform_layout = layout_name.to_string());

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
        self.with_current_layout(|layout| {
            let settings = self.settings.borrow();
            self.window.update_ui(
                settings.layout_autoswitch_enabled,
                self.is_processing_enabled.load(),
                settings.keys_logging_enabled,
                self.current_profile_name.borrow().as_str(),
                layout,
            );
        });
    }

    fn show_window(&self, show: bool) {
        self.update_window();
        self.window.set_visible(show);
    }

    fn select_next_transform_layout(&self) {
        let layouts = self.layouts.borrow();
        let next_name = {
            let current = self.current_layout_name.borrow(); /* must stay exactly inside the block */
            let next = layouts.cyclic_next(current.as_str());
            next.name.clone()
        };
        self.on_select_layout(next_name.as_str());
    }

    fn on_init(&self) {
        self.settings.replace(AppSettings::load());
        self.layouts.replace(TransformLayouts::load());
        self.profiles.replace(Profiles::load());

        self.with_settings(|settings| {
            if let Some(key) = &settings.toggle_layout_hot_key {
                self.key_hook.suppress_keys(&[key.action.key]);
            }
            self.window.apply_settings(&settings.main_window);
        });
        self.on_select_profile(None);

        let hwnd = self.window.hwnd();
        self.key_hook.setup(hwnd);
        self.key_hook.install();

        let settings = self.settings.borrow();
        self.is_processing_enabled.store(true);
        self.keyboard_layout_watcher.setup(hwnd);
        self.win_watcher.setup(
            hwnd,
            self.profiles.borrow().to_map(),
            settings.layout_autoswitch_enabled,
        );

        self.window.set_layouts(&self.layouts.borrow());
        self.update_window();

        #[cfg(feature = "debug")]
        self.window.set_visible(true);
    }

    fn on_key_hook_notify(&self, notification: &KeyEventNotification) {
        let settings = self.settings.borrow();

        if let Some(key) = &settings.toggle_layout_hot_key {
            if &notification.event.trigger == key {
                self.select_next_transform_layout();
            }
        }

        if settings.keys_logging_enabled {
            self.window.on_key_hook_notify(notification);
        }
    }

    pub(crate) fn on_select_profile(&self, profile_name: Option<&str>) {
        let n = profile_name
            .map(|s| s.to_string())
            .unwrap_or(NO_PROFILE.to_string());
        self.current_profile_name.replace(n);
        debug!("Selected profile: `{}`", self.current_profile_name.borrow());

        let layout_name = self.with_current_profile(|profile| profile.transform_layout.clone());
        self.apply_layout(&layout_name);
    }

    pub(crate) fn on_select_layout(&self, layout_name: &str) {
        self.apply_layout(layout_name);
        self.profiles.borrow().save();
    }

    pub(crate) fn on_keyboard_layout_changed(&self, state: &KeyboardLayoutState) {
        self.with_current_profile(|profile| {
            profile.keyboard_locale = Some(state.locale());
        });

        self.with_current_layout(|layout| {
            notify_layout_changed(layout, state);
        });

        self.profiles.borrow().save();
    }

    pub(crate) fn on_toggle_processing_enabled(&self) {
        self.is_processing_enabled.toggle();
        if self.is_processing_enabled.load() {
            self.key_hook.install();
        } else {
            self.key_hook.uninstall();
        }
        self.update_window();
    }

    pub(crate) fn on_toggle_logging_enabled(&self) {
        self.with_settings(|settings| {
            settings.keys_logging_enabled = !settings.keys_logging_enabled;
            settings.save();
        });

        self.update_window();
    }

    pub(crate) fn on_toggle_auto_switch_layout(&self) {
        self.with_settings(|settings| {
            settings.layout_autoswitch_enabled = !settings.layout_autoswitch_enabled;
            self.win_watcher.enable(settings.layout_autoswitch_enabled);
            settings.save();
        });
        self.update_window();
    }

    pub(crate) fn on_window_close(&self) {
        self.update_window();
        #[cfg(feature = "debug")]
        self.on_app_exit()
    }

    pub(crate) fn on_app_exit(&self) {
        self.with_settings(|settings| {
            self.window.update_settings(&mut settings.main_window);
            settings.save();
        });
        
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
