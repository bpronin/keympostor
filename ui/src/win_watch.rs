use crate::profile::Profiles;
use crate::ui::App;
use log::{debug, warn};
use native_windows_gui::{ControlHandle, Event};
use regex::Regex;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use windows::core::PWSTR;
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};
use windows::{
    Win32::Foundation::{CloseHandle, HWND, MAX_PATH},
    Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    },
    Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    },
};

const TIMER_ID: usize = 19717;
const WATCH_INTERVAL: u32 = 500;

#[derive(Default)]
pub(crate) struct WinWatcher {
    owner: RefCell<Option<HWND>>,
    is_enabled: RefCell<bool>,
    detector: RefCell<WindowActivationDetector>,
}

impl WinWatcher {
    pub(crate) fn init(&self, owner: Option<HWND>) {
        self.owner.replace(owner);
    }

    pub(crate) fn set_profiles(&self, profiles: Rc<Profiles>) {
        self.detector.borrow_mut().profiles = profiles;
    }

    pub(crate) fn is_enabled(&self) -> bool {
        *self.is_enabled.borrow()
    }

    pub(crate) fn set_enabled(&self, is_enabled: bool) {
        if is_enabled {
            self.start()
        } else {
            self.stop()
        }
    }

    fn start(&self) {
        if self.is_enabled.replace(true) {
            return;
        }

        unsafe {
            SetTimer(*self.owner.borrow(), TIMER_ID, WATCH_INTERVAL, None);
        }

        debug!("Window watch timer started");
    }

    pub(crate) fn stop(&self) {
        if !self.is_enabled.replace(false) {
            return;
        }

        unsafe {
            KillTimer(*self.owner.borrow(), TIMER_ID).unwrap_or_else(|e| {
                if e.code().is_err() {
                    warn!("Failed to kill window watch timer: {}", e);
                }
            });
        }

        debug!("Window watch timer stopped");
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        match evt {
            Event::OnTimerTick => {
                if let Some((_, timer_id)) = handle.timer() {
                    if timer_id == TIMER_ID as u32 {
                        self.invoke_detector(app);
                    }
                }
            }
            _ => {}
        };
    }

    fn invoke_detector(&self, app: &App) {
        if let Some(profile_name) = self.detector.borrow_mut().detect() {
            // if unsafe { GetForegroundWindow() } == *self.owner.borrow(){
            //     debug!("Self window detected, skipping profile switch");
            //     return;
            // }
            app.select_profile(profile_name);
        }
    }
}

#[derive(Default)]
struct WindowActivationDetector {
    profiles: Rc<Profiles>,
    last_hwnd: Option<HWND>,
}

impl WindowActivationDetector {
    fn detect(&mut self) -> Option<Option<&str>> {
        if let Some((hwnd, profile_name)) = detect_active_window(self.profiles.as_ref()) {
            let activated = self.last_hwnd.map_or(true, |it| it != hwnd);
            self.last_hwnd = Some(hwnd);
            if activated {
                debug!("Window detected for profile: {:?}", profile_name);

                return Some(Some(profile_name));
            }
        } else if self.last_hwnd.is_some() {
            debug!("No active profile windows");

            self.last_hwnd = None;
            return Some(None);
        }
        None
    }
}

fn detect_active_window(profiles: &Profiles) -> Option<(HWND, &String)> {
    profiles
        .iter()
        .find_map(|(name, profile)| match profile.regex() {
            Some(regex) => get_active_window(&regex).map(|hwnd| (hwnd, name)),
            None => None,
        })
}

fn get_process_name(hwnd: HWND) -> Result<String, Box<dyn Error>> {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return Err("Failed to get process ID".into());
        }

        let p_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)?;

        let mut buffer = [0u16; MAX_PATH as usize];
        let mut buffer_size = buffer.len() as u32;
        let success = QueryFullProcessImageNameW(
            p_handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut buffer_size,
        )
        .is_ok();

        CloseHandle(p_handle)?;

        if success {
            Ok(String::from_utf16_lossy(&buffer[..buffer_size as usize]))
        } else {
            Err("Failed to get process name".into())
        }
    }
}

fn get_window_title(hwnd: HWND) -> Result<String, Box<dyn Error>> {
    unsafe {
        if GetWindowTextLengthW(hwnd) == 0 {
            return Err("Invalid window handle".into());
        }

        let mut buffer = vec![0u16; (GetWindowTextLengthW(hwnd) + 1) as usize];
        let bytes_read = GetWindowTextW(hwnd, &mut buffer);
        if bytes_read == 0 {
            return Err("Failed to get window title".into());
        }

        let result = String::from_utf16_lossy(&buffer[..bytes_read as usize]);
        Ok(result)
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

    #[test]
    fn test_get_window_title() {
        let hwnd = unsafe { GetForegroundWindow() };
        let result = get_window_title(hwnd);

        assert!(result.is_ok());
        println!("{:?}", result);
    }

    #[test]
    fn test_get_process_name() {
        let hwnd = unsafe { GetForegroundWindow() };
        let result = get_process_name(hwnd);

        assert!(result.is_ok());
        println!("{:?}", result);
    }
}
