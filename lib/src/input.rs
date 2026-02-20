use crate::action::{KeyAction, KeyActionSequence};
use crate::key::Key;
use crate::transition::KeyTransition::{Down, Up};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT, MOUSE_EVENT_FLAGS,
    VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{XBUTTON1, XBUTTON2};

pub(crate) static PRIVATE_EVENT_MARKER: usize = 497298395;

pub(crate) fn build_input(seq: &KeyActionSequence) -> Vec<INPUT> {
    seq.iter().filter_map(build_action_input).collect()
}

fn build_action_input(action: &KeyAction) -> Option<INPUT> {
    build_mouse_button_input(action)
        .or_else(|| build_mouse_x_button_input(action))
        .or_else(|| build_mouse_wheel_input(action))
        .or_else(|| build_key_input(action))
}

fn build_mouse_wheel_input(action: &KeyAction) -> Option<INPUT> {
    if action.key == Key::WheelX {
        build_mouse_input(MOUSEEVENTF_WHEEL, 0)
    } else if action.key == Key::WheelY {
        build_mouse_input(MOUSEEVENTF_WHEEL, 0)
    } else {
        return None;
    }
}

fn build_mouse_button_input(action: &KeyAction) -> Option<INPUT> {
    let flags = if action.key == Key::LeftButton {
        if action.transition == Down {
            MOUSEEVENTF_LEFTDOWN
        } else {
            MOUSEEVENTF_LEFTUP
        }
    } else if action.key == Key::RightButton {
        if action.transition == Down {
            MOUSEEVENTF_RIGHTDOWN
        } else {
            MOUSEEVENTF_RIGHTUP
        }
    } else if action.key == Key::MiddleButton {
        if action.transition == Down {
            MOUSEEVENTF_MIDDLEDOWN
        } else {
            MOUSEEVENTF_MIDDLEUP
        }
    } else {
        return None;
    };

    build_mouse_input(flags, 0)
}

fn build_mouse_x_button_input(action: &KeyAction) -> Option<INPUT> {
    let data = if action.key == Key::Xbutton1 {
        XBUTTON1
    } else if action.key == Key::Xbutton2 {
        XBUTTON2
    } else {
        return None;
    };

    let flags = if action.transition == Down {
        MOUSEEVENTF_XDOWN
    } else {
        MOUSEEVENTF_XUP
    };

    build_mouse_input(flags, data as u32)
}

fn build_mouse_input(flags: MOUSE_EVENT_FLAGS, data: u32) -> Option<INPUT> {
    Some(INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dwFlags: flags,
                mouseData: data,
                dwExtraInfo: PRIVATE_EVENT_MARKER,
                ..Default::default()
            },
        },
    })
}

fn build_key_input(action: &KeyAction) -> Option<INPUT> {
    if action.key == Key::Unassigned {
        return None;
    }

    let mut flags = KEYEVENTF_SCANCODE;
    if action.key.is_ext_sc() {
        flags |= KEYEVENTF_EXTENDEDKEY
    }
    if action.transition == Up {
        flags |= KEYEVENTF_KEYUP;
    }

    Some(INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(action.key.vk() as u16),
                wScan: action.key.ext_sc(),
                dwFlags: flags,
                dwExtraInfo: PRIVATE_EVENT_MARKER,
                ..Default::default()
            },
        },
    })
}

#[cfg(test)]
mod tests {
    use crate::action::KeyAction;
    use crate::input::{build_action_input, build_key_input, PRIVATE_EVENT_MARKER};
    use crate::key_action;
    use crate::key_code::ext_scan_code;
    use std::str::FromStr;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
        KEYEVENTF_SCANCODE, MOUSEEVENTF_WHEEL, VK_RETURN,
    };

    #[test]
    fn test_build_key_input() {
        let actual: INPUT = build_action_input(&key_action!("ENTER*")).unwrap();
        unsafe {
            assert_eq!(INPUT_KEYBOARD, actual.r#type);
            assert_eq!(VK_RETURN, actual.Anonymous.ki.wVk);
            assert_eq!(ext_scan_code(0x1C, false), actual.Anonymous.ki.wScan);
            assert_eq!(KEYEVENTF_SCANCODE, actual.Anonymous.ki.dwFlags);
            assert_eq!(PRIVATE_EVENT_MARKER, actual.Anonymous.ki.dwExtraInfo);
        };

        let actual: INPUT = build_key_input(&key_action!("NUM_ENTER^")).unwrap();
        unsafe {
            assert_eq!(INPUT_KEYBOARD, actual.r#type);
            assert_eq!(VK_RETURN, actual.Anonymous.ki.wVk);
            assert_eq!(ext_scan_code(0x1C, true), actual.Anonymous.ki.wScan);
            assert_eq!(
                KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
                actual.Anonymous.ki.dwFlags
            );
            assert_eq!(PRIVATE_EVENT_MARKER, actual.Anonymous.ki.dwExtraInfo);
        };
    }

    #[test]
    fn test_build_mouse_wheel_input() {
        let actual: INPUT = build_action_input(&key_action!("WHEEL_Y*")).unwrap();
        unsafe {
            assert_eq!(INPUT_MOUSE, actual.r#type);
            assert_eq!(MOUSEEVENTF_WHEEL, actual.Anonymous.mi.dwFlags);
            // assert_eq!(120, actual.Anonymous.mi.dy);
            // assert_eq!(0, actual.Anonymous.mi.dx);
            assert_eq!(0, actual.Anonymous.mi.mouseData);
            assert_eq!(PRIVATE_EVENT_MARKER, actual.Anonymous.mi.dwExtraInfo);
        };

        let actual: INPUT = build_action_input(&key_action!("WHEEL_X^")).unwrap();
        unsafe {
            assert_eq!(INPUT_MOUSE, actual.r#type);
            assert_eq!(MOUSEEVENTF_WHEEL, actual.Anonymous.mi.dwFlags);
            // assert_eq!(0, actual.Anonymous.mi.dy);
            // assert_eq!(-480, actual.Anonymous.mi.dx);
            assert_eq!(0, actual.Anonymous.mi.mouseData);
            assert_eq!(PRIVATE_EVENT_MARKER, actual.Anonymous.mi.dwExtraInfo);
        };
    }
}
