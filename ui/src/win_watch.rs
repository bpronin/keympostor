use crate::app::App;
use crate::profile::LayoutAutoswitchProfile;
use crate::util::{with_process_path, with_window_title};
use log::{debug, warn};
use native_windows_gui::{ControlHandle, Event};
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};
use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::GetForegroundWindow};

const TIMER_ID: usize = 19717;
const WATCH_INTERVAL: u32 = 500;

#[derive(Default)]
pub(crate) struct WindowWatcher {
    owner: RefCell<HWND>,
    profiles: RefCell<Rc<HashMap<String, LayoutAutoswitchProfile>>>,
    last_hwnd: RefCell<Option<HWND>>,
}

impl WindowWatcher {
    pub(crate) fn setup(
        &self,
        owner: HWND,
        profiles: HashMap<String, LayoutAutoswitchProfile>,
        enable: bool,
    ) {
        self.owner.replace(owner);
        self.profiles.replace(Rc::from(profiles));
        self.enable(enable);
    }

    pub(crate) fn enable(&self, enable: bool) {
        if enable {
            unsafe {
                SetTimer(Some(*self.owner.borrow()), TIMER_ID, WATCH_INTERVAL, None);
            }
            debug!("Window watch timer started");
        } else {
            unsafe {
                KillTimer(Some(*self.owner.borrow()), TIMER_ID).unwrap_or_else(|e| {
                    if e.code().is_err() {
                        warn!("Failed to kill window watch timer: {}", e);
                    }
                });
            }
            debug!("Window watch timer stopped");
        }
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        if let Event::OnTimerTick = evt {
            if !is_our_timer_tick(handle) {
                return;
            }

            if let Some(profile_name) = self.detect_profile_change() {
                app.on_select_profile(profile_name.as_deref())
            }
        }
    }

    fn detect_profile_change(&self) -> Option<Option<String>> {
        let profiles = self.profiles.borrow();

        let match_result = profiles.iter().find_map(|(profile_name, profile)| {
            let regex = profile.rule_regex()?;
            active_window_matches(&regex).map(|hwnd| (hwnd, profile_name.clone()))
        });

        if let Some((hwnd, profile_name)) = match_result {
            let is_new_activation = self.last_hwnd.borrow().map_or(true, |prev| prev != hwnd);
            self.last_hwnd.replace(Some(hwnd));

            if is_new_activation {
                debug!("Window detected for profile: `{}`", profile_name);
                return Some(Some(profile_name));
            }

            return None;
        }

        if self.last_hwnd.borrow().is_some() {
            debug!("No active profile windows");
            self.last_hwnd.replace(None);
            return Some(None);
        }

        None
    }
}

fn is_our_timer_tick(handle: ControlHandle) -> bool {
    handle
        .timer()
        .is_some_and(|(_, timer_id)| timer_id == TIMER_ID as u32)
}

fn active_window_matches(regex: &Regex) -> Option<HWND> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return None;
    }

    if with_window_title(hwnd, |t| regex.is_match(t)).unwrap_or(false) {
        return Some(hwnd);
    }

    if with_process_path(hwnd, |n| regex.is_match(n)).unwrap_or(false) {
        return Some(hwnd);
    }

    None
}
