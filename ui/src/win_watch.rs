use crate::app::App;
use crate::profile::LayoutAutoswitchProfile;
use crate::util::{get_process_name, get_window_title};
use log::{debug, warn};
use native_windows_gui::{ControlHandle, Event};
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};
use windows::{
    Win32::Foundation::HWND,
    Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    },
};

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
        profiles: Rc<HashMap<String, LayoutAutoswitchProfile>>,
        enable: bool,
    ) {
        self.owner.replace(owner);
        self.profiles.replace(profiles);
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
        match evt {
            Event::OnTimerTick => {
                if let Some((_, timer_id)) = handle.timer() {
                    if timer_id == TIMER_ID as u32 {
                        if let Some(profile_name) = self.detect_profile() {
                            app.on_select_profile(profile_name.as_deref());
                        }
                    }
                }
            }
            _ => {}
        };
    }

    fn detect_profile(&self) -> Option<Option<String>> {
        let profiles = self.profiles.borrow();
        let find_result = profiles.iter().find_map(|(profile_name, profile)| {
            profile
                .rule_regex()
                .and_then(|regex| get_active_window(&regex).map(|hwnd| (hwnd, profile_name)))
        });

        if let Some((hwnd, profile_name)) = find_result {
            let activated = self.last_hwnd.borrow().map_or(true, |it| it != hwnd);
            self.last_hwnd.replace(Some(hwnd));
            if activated {
                debug!("Window detected for profile: `{}`", profile_name);

                return Some(Some(profile_name.clone()));
            }
        } else if self.last_hwnd.borrow().is_some() {
            debug!("No active profile windows");

            self.last_hwnd.replace(None);
            return Some(None);
        }
        None
    }
}

fn get_active_window(regex: &Regex) -> Option<HWND> {
    let hwnd = unsafe { GetForegroundWindow() };
    if !hwnd.is_invalid() {
        if let Ok(window_title) = get_window_title(hwnd) {
            if regex.is_match(&window_title) {
                return Some(hwnd);
            }
        }

        if let Ok(process_name) = get_process_name(hwnd) {
            if regex.is_match(&process_name) {
                return Some(hwnd);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{get_process_name, get_window_title};

    #[test]
    fn test_get_window_title() {
        let hwnd = unsafe { GetForegroundWindow() };
        let result = get_window_title(hwnd);

        assert!(result.is_ok());
        // println!("{:?}", result);
    }

    #[test]
    fn test_get_process_name() {
        let hwnd = unsafe { GetForegroundWindow() };
        let result = get_process_name(hwnd);

        assert!(result.is_ok());
        // println!("{:?}", result);
    }
}
