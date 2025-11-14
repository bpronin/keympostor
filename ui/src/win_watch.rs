use crate::profile::Profile;
use crate::ui::App;
use crate::utils::raw_hwnd;
use error::Error;
use keympostor::ife;
use log::{debug, warn};
use native_windows_gui::{ControlHandle, Event};
use regex::Regex;
use std::cell::RefCell;
use std::error;
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};
use windows::core::PWSTR;
use windows::{
    Win32::Foundation::{CloseHandle, HWND, MAX_PATH},
    Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    },
    Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    },
};

const DETECTOR_TIMER: u32 = 19717;
const WIN_WATCH_INTERVAL: u32 = 500;

#[derive(Default)]
pub(crate) struct WinWatcher {
    handle: RefCell<ControlHandle>,
    is_enabled: RefCell<bool>,
    detector: RefCell<WindowActivationDetector>,
}

impl WinWatcher {
    pub(crate) fn init(&self, handle: ControlHandle) {
        self.handle.replace(handle);
    }

    pub(crate) fn set_profiles(&self, profiles: Option<Vec<Profile>>) {
        self.detector.borrow_mut().profiles = profiles;
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.is_enabled.borrow().to_owned()
    }

    pub(crate) fn set_enabled(&self, is_enabled: bool) {
        if is_enabled == self.is_enabled.replace(is_enabled) {
            return;
        }

        let hwnd = raw_hwnd(self.handle.borrow().to_owned());
        if is_enabled {
            unsafe {
                SetTimer(hwnd, DETECTOR_TIMER as usize, WIN_WATCH_INTERVAL, None);
            }
        } else {
            unsafe {
                KillTimer(hwnd, DETECTOR_TIMER as usize).unwrap_or_else(|e| {
                    warn!("Failed to kill timer: {}", e);
                });
            }
        };

        debug!(
            "Profile auto-switch {}",
            ife!(self.is_enabled(), "enabled", "disabled")
        );
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        match evt {
            Event::OnTimerTick => {
                if let Some((_, timer_id)) = handle.timer() {
                    if timer_id == DETECTOR_TIMER {
                        self.invoke_detector(app);
                    }
                }
            }
            _ => {}
        };
    }

    fn invoke_detector(&self, app: &App) {
        if let Some(result) = self.detector.borrow_mut().detect() {
            match result {
                Some(profile) => app.on_select_layout(&profile.layout),
                None => app.on_select_layout(&None),
            }
        }
    }
}

impl Drop for WinWatcher {
    fn drop(&mut self) {
        self.set_enabled(false)
    }
}

#[derive(Default)]
struct WindowActivationDetector {
    profiles: Option<Vec<Profile>>,
    last_hwnd: Option<HWND>,
}

impl WindowActivationDetector {
    fn detect(&mut self) -> Option<Option<&Profile>> {
        if let Some(profile) = &self.profiles {
            if let Some((hwnd, profile)) = detect_active_window(profile) {
                let activated = self.last_hwnd.map_or(true, |it| it != hwnd);
                self.last_hwnd = Some(hwnd);
                if activated {
                    debug!("Window detected for profile: {:?}", profile);

                    return Some(Some(profile));
                }
            } else if self.last_hwnd.is_some() {
                debug!("No active profile windows");

                self.last_hwnd = None;
                return Some(None);
            }
        }
        None
    }
}

fn detect_active_window(profiles: &Vec<Profile>) -> Option<(HWND, &Profile)> {
    profiles
        .iter()
        .find_map(|profile| get_active_window(&profile.regex()).map(|hwnd| (hwnd, profile)))
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

        Ok(String::from_utf16_lossy(&buffer[..bytes_read as usize]))
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
