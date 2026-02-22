use crate::action::KeyAction;
use crate::event::KeyEvent;
use crate::input::PRIVATE_EVENT_MARKER;
use crate::key::Key;
use crate::key::Key::{LeftButton, MiddleButton, RightButton, WheelX, WheelY};
use crate::modifiers::KeyModifiers::All;
use crate::notify::install_notify_listener;
use crate::rule::{KeyTransformRule, KeyTransformRules};
use crate::state::KeyboardState;
use crate::transform::KeyTransformMap;
use crate::transition::KeyTransition;
use crate::transition::KeyTransition::{Down, Up};
use crate::trigger::KeyTrigger;
use crate::utils::if_else;
use crate::{input, notify};
use fxhash::FxHashSet;
use input::build_input;
use log::{debug, trace, warn};
use notify::notify_key_event;
use std::cell::{Cell, RefCell};
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT};
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Debug, Default)]
pub struct KeyboardHook {}

impl KeyboardHook {
    pub fn setup(&self, owner: HWND) {
        install_notify_listener(owner);
    }

    pub fn install(&self) {
        KEYBOARD_STATE.replace(KeyboardState::default());
        trace!("Keyboard state cleared");

        install_keyboard_hook();

        #[cfg(feature = "no_mouse")]
        warn!("Mouse hook is disabled by feature flag");
        #[cfg(not(feature = "no_mouse"))]
        install_mouse_hook();
    }

    pub fn uninstall(&self) {
        uninstall_key_hook();
        #[cfg(not(feature = "no_mouse"))]
        uninstall_mouse_hook();
    }

    pub fn set_rules(&self, rules: Option<&KeyTransformRules>) {
        let map = rules.and_then(|r| Some(KeyTransformMap::new(r.iter())));
        TRANSFOFM_MAP.replace(map);
    }

    pub fn suppress_keys(&self, keys: &[Key]) {
        SUPPRESSED_KEYS.replace(FxHashSet::from_iter(keys.iter().cloned()));
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        self.uninstall();
    }
}

thread_local! {
    static KEY_HOOK: Cell<Option<HHOOK>> = Cell::new(None);
    static MOUSE_HOOK: Cell<Option<HHOOK>> = Cell::new(None);
    static KEYBOARD_STATE: Cell<KeyboardState> = Cell::new(KeyboardState::default());
    static TRANSFOFM_MAP: RefCell<Option<KeyTransformMap>> = RefCell::new(None);
    static SUPPRESSED_KEYS: RefCell<FxHashSet<Key>> = RefCell::new(FxHashSet::default());
}

fn install_keyboard_hook() {
    if KEY_HOOK.get().is_some() {
        warn!("Keyboard hook already installed");
        return;
    }

    match unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_hook_proc), None, 0) } {
        Ok(handle) => {
            KEY_HOOK.replace(Some(handle));
            debug!("Keyboard hook installed");
        }
        Err(e) => {
            KEY_HOOK.replace(None);
            warn!("Failed to install keyboard hook: {}", e);
        }
    }
}

fn uninstall_key_hook() {
    if let Some(handle) = KEY_HOOK.take() {
        match unsafe { UnhookWindowsHookEx(handle) } {
            Ok(_) => debug!("Keyboard hook uninstalled"),
            Err(e) => warn!("Failed to uninstall keyboard hook: {}", e),
        }
    } else {
        warn!("Keyboard hook already uninstalled");
    }
}

fn install_mouse_hook() {
    if MOUSE_HOOK.get().is_some() {
        warn!("Mouse hook already installed");
        return;
    }

    match unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0) } {
        Ok(handle) => {
            MOUSE_HOOK.replace(Some(handle));
            debug!("Mouse hook installed");
        }
        Err(e) => {
            MOUSE_HOOK.replace(None);
            warn!("Failed to install mouse hook: {}", e);
        }
    }
}

fn uninstall_mouse_hook() {
    if let Some(handle) = MOUSE_HOOK.take() {
        match unsafe { UnhookWindowsHookEx(handle) } {
            Ok(_) => debug!("Mouse hook uninstalled"),
            Err(e) => warn!("Failed to uninstall mouse hook: {}", e),
        }
    } else {
        warn!("Mouse hook already uninstalled");
    }
}

extern "system" fn key_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let input = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
        let event = build_key_event(input);
        if handle_event(&event) {
            return LRESULT(1);
        }
    }

    unsafe { CallNextHookEx(KEY_HOOK.get(), code, w_param, l_param) }
}

extern "system" fn mouse_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let msg = w_param.0 as u32;
    if msg != WM_MOUSEMOVE {
        let input = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };
        let event = build_mouse_event(msg, input);
        if handle_event(&event) {
            return LRESULT(1);
        }
    }

    unsafe { CallNextHookEx(MOUSE_HOOK.get(), code, w_param, l_param) }
}

#[inline(always)]
fn handle_event(event: &KeyEvent) -> bool {
    trace!("Processing event: {event}");

    if event.is_private {
        trace!("Event ignored");
        notify_key_event(event.clone(), None);
        return false;
    }

    if SUPPRESSED_KEYS.with_borrow(|set| set.contains(&event.trigger.action.key)) {
        trace!("Event suppressed");
        update_kbd_state(&event.trigger.action);
        notify_key_event(event.clone(), None);
        return true;
    }

    match get_rule(&event) {
        Some(rule) => {
            debug!("Applying rule: {}", rule);
            notify_key_event(event.clone(), Some(rule.clone()));
            apply_rule(&rule);
            true
        }
        None => {
            trace!("No matching rules");
            notify_key_event(event.clone(), None);
            update_kbd_state(&event.trigger.action);
            false
        }
    }
}

#[inline(always)]
fn get_rule(event: &KeyEvent) -> Option<KeyTransformRule> {
    TRANSFOFM_MAP.with_borrow(|transform_map| {
        transform_map
            .as_ref()
            .and_then(|map| map.get(&event.trigger).cloned())
    })
}

#[inline(always)]
fn apply_rule(rule: &KeyTransformRule) {
    unsafe {
        if SendInput(&build_input(&rule.actions), size_of::<INPUT>() as i32) == 0 {
            warn!("Failed to send input: {:?}", GetLastError());
        }
    }
}

#[inline(always)]
fn build_key_event(input: KBDLLHOOKSTRUCT) -> KeyEvent {
    let action = build_action_from_kbd_input(input);
    KeyEvent {
        trigger: KeyTrigger {
            action,
            modifiers: All(prepare_kbd_state(&action)),
        },
        is_injected: input.flags.contains(LLKHF_INJECTED),
        is_private: input.dwExtraInfo == PRIVATE_EVENT_MARKER,
        time: input.time,
    }
}

#[inline(always)]
fn build_mouse_event(msg: u32, input: MSLLHOOKSTRUCT) -> KeyEvent {
    let action = build_action_from_mouse_input(msg, input);
    KeyEvent {
        trigger: KeyTrigger {
            action,
            modifiers: All(prepare_kbd_state(&action)),
        },
        is_injected: (input.flags & (LLMHF_INJECTED | LLMHF_LOWER_IL_INJECTED)) != 0,
        is_private: input.dwExtraInfo == PRIVATE_EVENT_MARKER,
        time: input.time,
    }
}

#[inline(always)]
fn build_action_from_kbd_input(input: KBDLLHOOKSTRUCT) -> KeyAction {
    KeyAction {
        key: Key::from_code(
            input.vkCode as u8,
            input.scanCode as u8,
            input.flags.contains(LLKHF_EXTENDED),
        ),
        transition: if_else(input.flags.contains(LLKHF_UP), Up, Down),
    }
}

#[inline(always)]
fn build_action_from_mouse_input(msg: u32, input: MSLLHOOKSTRUCT) -> KeyAction {
    match msg {
        WM_LBUTTONDOWN => KeyAction::new(LeftButton, Down),
        WM_LBUTTONUP => KeyAction::new(LeftButton, Up),
        WM_RBUTTONDOWN => KeyAction::new(RightButton, Down),
        WM_RBUTTONUP => KeyAction::new(RightButton, Up),
        WM_MBUTTONDOWN => KeyAction::new(MiddleButton, Down),
        WM_MBUTTONUP => KeyAction::new(MiddleButton, Up),
        WM_XBUTTONDOWN => KeyAction::new(build_mouse_x_button_key(input), Down),
        WM_XBUTTONUP => KeyAction::new(build_mouse_x_button_key(input), Up),
        WM_MOUSEWHEEL => KeyAction::new(WheelY, build_mouse_wheel_transition(input)),
        WM_MOUSEHWHEEL => KeyAction::new(WheelX, build_mouse_wheel_transition(input)),
        _ => panic!("Illegal mouse message: `{}`", msg),
    }
}

#[inline(always)]
fn build_mouse_wheel_transition(input: MSLLHOOKSTRUCT) -> KeyTransition {
    let delta = (input.mouseData >> 16) as i16;
    if_else(delta < 0, Up, Down)
}

#[inline(always)]
fn build_mouse_x_button_key(input: MSLLHOOKSTRUCT) -> Key {
    match (input.mouseData >> 16) as u16 {
        1 => Key::Xbutton1,
        2 => Key::Xbutton2,
        b => {
            warn!("Unsupported mouse x-button: `{b}`");
            Key::Xbutton1
        }
    }
}

#[inline(always)]
fn prepare_kbd_state(action: &KeyAction) -> KeyboardState {
    let mut state = KEYBOARD_STATE.get();
    state.remove(&action);
    KEYBOARD_STATE.set(state);
    state
}

#[inline(always)]
fn update_kbd_state(action: &KeyAction) {
    let mut state = KEYBOARD_STATE.get();
    state.update(action);
    KEYBOARD_STATE.set(state);
}
