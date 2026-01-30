use crate::indicator;
use crate::ui::App;
use crate::util::get_current_keyboard_layout;
use indicator::notify_layout_changed;
use log::{debug, warn};
use native_windows_gui::{ControlHandle, Event};
use std::cell::RefCell;
use windows::Win32::Foundation::HWND;
use windows::Win32::Globalization::GetLocaleInfoW;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, GetKeyboardLayout, HKL, VK_NUMLOCK};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId, KillTimer, SetTimer};

const TIMER_ID: usize = 19718;
const WATCH_INTERVAL: u32 = 200;

#[derive(Default)]
pub(crate) struct KeyboardLayoutWatcher {
    owner: RefCell<Option<HWND>>,
    last_layout: RefCell<HKL>,
}

impl KeyboardLayoutWatcher {
    pub(crate) fn start(&self, owner: Option<HWND>) {
        self.owner.replace(owner);
        self.last_layout.replace(get_current_keyboard_layout());
        unsafe {
            SetTimer(*self.owner.borrow(), TIMER_ID, WATCH_INTERVAL, None);
        }

        debug!("Keyboard layout watch started");
    }

    pub(crate) fn stop(&self) {
        unsafe {
            KillTimer(*self.owner.borrow(), TIMER_ID).unwrap_or_else(|e| {
                if e.code().is_err() {
                    warn!("Failed to kill keyboard layout watch timer: {}", e);
                }
            });
        }

        debug!("Keyboard layout watch stopped");
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        match evt {
            Event::OnTimerTick => {
                if let Some((_, timer_id)) = handle.timer() {
                    if timer_id == TIMER_ID as u32 {
                        self.check_keyboard_layout_state(app);
                    }
                }
            }
            _ => {}
        };
    }

    fn check_keyboard_layout_state(&self, app: &App) {
        let keyboard_layout = get_current_keyboard_layout();
        if keyboard_layout == *self.last_layout.borrow() {
            return;
        }

        self.last_layout.replace(keyboard_layout);

        debug!("Keyboard layout changed to {:?}", keyboard_layout);

        app.with_current_layout(|layout| {
            notify_layout_changed(layout, keyboard_layout);
        })
    }
}

