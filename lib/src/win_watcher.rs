use error::Error;
use crate::profile::ActivationRules;
use log::debug;
use regex::Regex;
use std::error;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self};
use std::time::Duration;
use windows::core::PWSTR;
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

const CHECK_INTERVAL: Duration = Duration::from_millis(100);

pub struct WindowWatcher {
    running: Arc<AtomicBool>,
}

impl WindowWatcher {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    // pub fn apply_profile(&self, settings: Option<ActivationRules>) -> Result<()> {
    //     if let Some(settings) = &settings {
    //         let re = Regex::new(&settings.window_title).context("Invalid regex")?;
    //         self.start(re, |hwnd, active| {
    //             debug!(
    //                 "Auto-activating window `{}` is {}",
    //                 &get_window_title(hwnd).unwrap(),
    //                 if active { "active" } else { "inactive" }
    //             );
    //         });
    //
    //         debug!("Window watcher enabled");
    //     } else {
    //         self.stop();
    //
    //         debug!("Window watcher disabled");
    //     }
    //
    //     Ok(())
    // }

    pub fn start<F>(&self, regex: Regex, callback: F)
    where
        F: Fn(HWND, bool) + Send + 'static,
    {
        if self.running.load(Ordering::SeqCst) {
            return;
        }
        self.running.store(true, Ordering::SeqCst);

        debug!("Window watcher is running");

        let mut last_hwnd = None;
        let running = self.running.clone();
        while running.load(Ordering::SeqCst) {
            if let Some(hwnd) = get_active_window(&regex) {
                if last_hwnd.map_or(true, |prev| prev != hwnd) {
                    debug!("Watch window activated");

                    callback(hwnd, true);
                }
                last_hwnd = Some(hwnd);
            } else if let Some(hwnd) = last_hwnd {
                debug!("Watch window deactivated");

                callback(hwnd, false);
                last_hwnd = None;
            }

            thread::sleep(CHECK_INTERVAL);
        }

        debug!("Window watcher stopped");
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Drop for WindowWatcher {
    fn drop(&mut self) {
        self.stop()
    }
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
