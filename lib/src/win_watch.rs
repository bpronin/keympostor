use error::Error;
use regex::Regex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{error, thread};
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

const CHECK_INTERVAL: Duration = Duration::from_millis(1000);

pub fn detect_window_activation<F>(
    rules: Vec<Regex>,
    on_window_active: Box<F>,
    run_handle: Arc<AtomicBool>,
) where
    F: Fn(Option<&Regex>) + Send + 'static,
{
    let mut last_hwnd = None;
    while run_handle.load(Ordering::SeqCst) {
        if let Some((hwnd, rule)) = detect_active_window(rules.as_ref()) {
            if last_hwnd.map_or(true, |it| it != hwnd) {
                on_window_active(Some(rule));
            }
            last_hwnd = Some(hwnd);
        } else if last_hwnd.is_some() {
            on_window_active(None);
            last_hwnd = None;
        }

        thread::sleep(CHECK_INTERVAL);
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

fn get_active_window(rule: &Regex) -> Option<HWND> {
    let hwnd = unsafe { GetForegroundWindow() };
    if !hwnd.is_invalid() {
        if let Ok(window_title) = get_window_title(hwnd) {
            if rule.is_match(&window_title) {
                return Some(hwnd);
            }
        }

        if let Ok(process_name) = get_process_name(hwnd) {
            if rule.is_match(&process_name) {
                return Some(hwnd);
            }
        }
    }
    None
}

fn detect_active_window(rules: &[Regex]) -> Option<(HWND, &Regex)> {
    rules
        .iter()
        .find_map(|rule| get_active_window(rule).map(|hwnd| (hwnd, rule)))
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
