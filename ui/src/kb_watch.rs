use crate::kb_light::{get_current_keyboard_layout, update_keyboard_lighting};
use crate::ui::App;
use log::{debug, warn};
use native_windows_gui::{ControlHandle, Event};
use std::cell::RefCell;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::HKL;
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

const TIMER_ID: usize = 19718;
const WATCH_INTERVAL: u32 = 500;

#[derive(Default)]
pub(crate) struct KeyboardLayoutWatcher {
    owner: RefCell<HWND>,
    last_layout: RefCell<HKL>,
}

impl KeyboardLayoutWatcher {
    pub(crate) fn init(&self, owner: HWND) {
        self.owner.replace(owner);
        self.last_layout.replace(get_current_keyboard_layout());
    }

    pub(crate) fn start(&self) {
        unsafe {
            SetTimer(Some(*self.owner.borrow()), TIMER_ID, WATCH_INTERVAL, None);
        }

        debug!("Keyboard layout watch started");
    }

    pub(crate) fn stop(&self) {
        unsafe {
            KillTimer(Some(*self.owner.borrow()), TIMER_ID).unwrap_or_else(|e| {
                warn!("Failed to kill timer: {}", e);
            });
        }

        debug!("Keyboard layout watch stopped");
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        match evt {
            Event::OnTimerTick => {
                if let Some((_, timer_id)) = handle.timer() {
                    if timer_id == TIMER_ID as u32 {
                        self.on_keyboard_layout_change(app);
                    }
                }
            }
            _ => {}
        };
    }

    fn on_keyboard_layout_change(&self, app: &App) {
        let current_layout = get_current_keyboard_layout();
        if current_layout == *self.last_layout.borrow() {
            return;
        }

        self.last_layout.replace(current_layout);
        debug!("Keyboard layout changed to {:?}", current_layout);

        let layout_name = app.current_layout_name.borrow();
        update_keyboard_lighting(layout_name.as_deref(), current_layout);
    }
}
