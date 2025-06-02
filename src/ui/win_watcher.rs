use keympostor::profile::ActivationRules;
use log::debug;
use regex::Regex;
use std::cell::RefCell;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use windows::{
    Win32::Foundation::HWND,
    Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW},
};
use anyhow::{Context, Result};

const CHECK_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Default)]
pub struct WindowWatcher {
    running: Arc<AtomicBool>,
    handle: RefCell<Option<JoinHandle<()>>>,
}

impl WindowWatcher {
    pub(crate) fn apply_profile(&self, settings: Option<ActivationRules>) -> Result<()> {
        if let Some(settings) = &settings {
            let re = Regex::new(&settings.window_title).context("Invalid regex")?;
            self.start(re, |hwnd, active| {
                debug!(
                    "Auto-activating window `{}` is {}",
                    &get_window_title(hwnd).unwrap(),
                    if active { "active" } else { "inactive" }
                );
            });

            debug!("Window watcher enabled");
        } else {
            self.stop();

            debug!("Window watcher disabled");
        }

        Ok(())
    }

    fn start<F>(&self, regex: Regex, callback: F)
    where
        F: Fn(HWND, bool) + Send + 'static,
    {
        if self.running.load(Ordering::SeqCst) {
            return; 
        }
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        self.handle.replace(Some(thread::spawn(move || {
            debug!("Starting window watcher");

            let mut last_hwnd = None;
            while running.load(Ordering::SeqCst) {
                if let Some(hwnd) = get_active_window(&regex) {
                    if last_hwnd.map_or(true, |prev| prev != hwnd) {
                        callback(hwnd, true);
                    }
                    last_hwnd = Some(hwnd);
                } else if let Some(hwnd) = last_hwnd {
                    callback(hwnd, false);
                    last_hwnd = None;
                }
                thread::sleep(CHECK_INTERVAL);
            }

            debug!("Window watcher stopped");
        })));
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for WindowWatcher {
    fn drop(&mut self) {
        self.stop()
    }
}

fn get_window_title(hwnd: HWND) -> Option<String> {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return None;
        }

        let mut buffer = vec![0u16; (len + 1) as usize];
        let read_len = GetWindowTextW(hwnd, &mut buffer);
        if read_len == 0 {
            return None;
        }

        Some(String::from_utf16_lossy(&buffer[..read_len as usize]))
    }
}

fn get_active_window(re: &Regex) -> Option<HWND> {
    let hwnd = unsafe { GetForegroundWindow() };
    if !hwnd.is_invalid() {
        if let Some(window_title) = get_window_title(hwnd) {
            if re.is_match(&window_title) {
                return Some(hwnd);
            }
        }
    }
    None
}