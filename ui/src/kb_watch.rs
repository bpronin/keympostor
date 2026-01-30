use crate::ui::App;
use crate::{indicator, util};
use indicator::notify_layout_changed;
use log::{debug, warn};
use native_windows_gui::{ControlHandle, Event};
use std::cell::RefCell;
use util::{get_current_keyboard_layout, get_lock_state};
use windows::Win32::Foundation::HWND;
use windows::Win32::Globalization::GetLocaleInfoW;
use windows::Win32::UI::Input::KeyboardAndMouse::{HKL, VK_CAPITAL, VK_NUMLOCK, VK_SCROLL};
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

const TIMER_ID: usize = 19718;
const WATCH_INTERVAL: u32 = 200;

const LOCK_NUM: u8 = 1;
const LOCK_CAPS: u8 = 2;
const LOCK_SCROLL: u8 = 4;

#[derive(Default, Debug, PartialEq)]
pub(crate) struct KeyboardLayoutState {
    pub(crate) layout: HKL,
    pub(crate) locks: u8,
}

impl KeyboardLayoutState {
    pub(crate) fn capture() -> Self {
        let mut locks = 0;
        if get_lock_state(VK_NUMLOCK) {
            locks |= LOCK_NUM
        }
        if get_lock_state(VK_CAPITAL) {
            locks |= LOCK_CAPS
        }
        if get_lock_state(VK_SCROLL) {
            locks |= LOCK_SCROLL
        }

        Self {
            layout: get_current_keyboard_layout(),
            locks,
        }
    }

    pub(crate) fn locale(&self) -> String {
        let lang_id = (self.layout.0 as u32) & 0xFFFF;

        let get_locale_info = |lc_type: u32| -> String {
            unsafe {
                let buffer_size = GetLocaleInfoW(lang_id, lc_type, None) as usize;
                let mut buffer = vec![0u16; buffer_size];
                GetLocaleInfoW(lang_id, lc_type, Some(&mut buffer));
                buffer.set_len(buffer_size - 1); /* remove null terminator */
                String::from_utf16_lossy(&buffer)
            }
        };

        format!("{}_{}", get_locale_info(0x59), get_locale_info(0x5A)).to_lowercase()
    }

    pub(crate) fn locks(&self) -> String {
        let mut locks: Vec<String> = vec![];
        if self.locks & LOCK_NUM != 0 {
            locks.push(String::from("num"));
        }
        if self.locks & LOCK_CAPS != 0 {
            locks.push(String::from("caps"));
        }
        if self.locks & LOCK_SCROLL != 0 {
            locks.push(String::from("scroll"));
        }
        locks.join("_")
    }
}

#[derive(Default)]
pub(crate) struct KeyboardLayoutWatcher {
    owner: RefCell<Option<HWND>>,
    last_state: RefCell<KeyboardLayoutState>,
}

impl KeyboardLayoutWatcher {
    pub(crate) fn start(&self, owner: Option<HWND>) {
        self.owner.replace(owner);
        self.last_state.replace(KeyboardLayoutState::capture());

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
        let state = KeyboardLayoutState::capture();
        if state == *self.last_state.borrow() {
            return;
        }

        debug!("Keyboard layout state: {:?}", state);

        app.with_current_layout(|layout| {
            notify_layout_changed(layout, &state);
        });
        self.last_state.replace(state);
    }
}
