use crate::action::{KeyAction, KeyActionSequence};
use crate::event::SELF_EVENT_MARKER;
use crate::key::{
    KEY_LEFT_BUTTON, KEY_MIDDLE_BUTTON, KEY_RIGHT_BUTTON, KEY_WHEEL_X, KEY_WHEEL_Y, KEY_XBUTTON1,
    KEY_XBUTTON2,
};
use crate::transition::KeyTransition::{Down, Up};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP,
    MOUSEINPUT, MOUSE_EVENT_FLAGS,
};
use windows::Win32::UI::WindowsAndMessaging::{XBUTTON1, XBUTTON2};

pub fn build_input(seq: &KeyActionSequence) -> Vec<INPUT> {
    seq.iter().map(|a| build_action_input(a)).collect()
}

fn build_action_input(action: &KeyAction) -> INPUT {
    build_mouse_button_input(action).unwrap_or(
        build_mouse_x_button_input(action)
            .unwrap_or(build_mouse_wheel_input(action).unwrap_or(build_key_input(action))),
    )
}

fn build_mouse_wheel_input(action: &KeyAction) -> Option<INPUT> {
    if action.key == &KEY_WHEEL_X {
        build_mouse_input(MOUSEEVENTF_WHEEL, 0)
    } else if action.key == &KEY_WHEEL_Y {
        build_mouse_input(MOUSEEVENTF_WHEEL, 0)
    } else {
        return None;
    }
}

fn build_mouse_button_input(action: &KeyAction) -> Option<INPUT> {
    let flags = if action.key == &KEY_LEFT_BUTTON {
        if action.transition == Down {
            MOUSEEVENTF_LEFTDOWN
        } else {
            MOUSEEVENTF_LEFTUP
        }
    } else if action.key == &KEY_RIGHT_BUTTON {
        if action.transition == Down {
            MOUSEEVENTF_RIGHTDOWN
        } else {
            MOUSEEVENTF_RIGHTUP
        }
    } else if action.key == &KEY_MIDDLE_BUTTON {
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
    let data = if action.key == &KEY_XBUTTON1 {
        XBUTTON1
    } else if action.key == &KEY_XBUTTON2 {
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
                dwExtraInfo: SELF_EVENT_MARKER,
                ..Default::default()
            },
        },
    })
}

fn build_key_input(action: &KeyAction) -> INPUT {
    let mut flags = KEYEVENTF_SCANCODE;
    if action.key.sc.1 {
        flags |= KEYEVENTF_EXTENDEDKEY
    }
    if action.transition == Up {
        flags |= KEYEVENTF_KEYUP;
    }

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: action.key.vk.into(),
                wScan: action.key.sc.into_ext(),
                dwFlags: flags,
                dwExtraInfo: SELF_EVENT_MARKER,
                ..Default::default()
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::action::KeyAction;
    use crate::event::SELF_EVENT_MARKER;
    use crate::input::{build_action_input, build_key_input};
    use crate::key_action;
    use crate::sc::ScanCode;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
        KEYEVENTF_SCANCODE, MOUSEEVENTF_WHEEL, VK_RETURN,
    };

    #[test]
    fn test_build_key_input() {
        let actual: INPUT = build_action_input(&key_action!("ENTER*"));
        unsafe {
            assert_eq!(INPUT_KEYBOARD, actual.r#type);
            assert_eq!(VK_RETURN, actual.Anonymous.ki.wVk);
            assert_eq!(ScanCode(0x1C, false).into_ext(), actual.Anonymous.ki.wScan);
            assert_eq!(KEYEVENTF_SCANCODE, actual.Anonymous.ki.dwFlags);
            assert_eq!(SELF_EVENT_MARKER, actual.Anonymous.ki.dwExtraInfo);
        };

        let actual: INPUT = build_key_input(&key_action!("NUM_ENTER^"));
        unsafe {
            assert_eq!(INPUT_KEYBOARD, actual.r#type);
            assert_eq!(VK_RETURN, actual.Anonymous.ki.wVk);
            assert_eq!(ScanCode(0x1C, true).into_ext(), actual.Anonymous.ki.wScan);
            assert_eq!(
                KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
                actual.Anonymous.ki.dwFlags
            );
            assert_eq!(SELF_EVENT_MARKER, actual.Anonymous.ki.dwExtraInfo);
        };
    }

    #[test]
    fn test_build_mouse_wheel_input() {
        let actual: INPUT = build_action_input(&key_action!("WHEEL_Y*"));
        unsafe {
            assert_eq!(INPUT_MOUSE, actual.r#type);
            assert_eq!(MOUSEEVENTF_WHEEL, actual.Anonymous.mi.dwFlags);
            assert_eq!(120, actual.Anonymous.mi.dy);
            assert_eq!(0, actual.Anonymous.mi.dx);
            assert_eq!(0, actual.Anonymous.mi.mouseData);
            assert_eq!(SELF_EVENT_MARKER, actual.Anonymous.mi.dwExtraInfo);
        };

        let actual: INPUT = build_action_input(&key_action!("WHEEL_X^"));
        unsafe {
            assert_eq!(INPUT_MOUSE, actual.r#type);
            assert_eq!(MOUSEEVENTF_WHEEL, actual.Anonymous.mi.dwFlags);
            assert_eq!(0, actual.Anonymous.mi.dy);
            assert_eq!(-480, actual.Anonymous.mi.dx);
            assert_eq!(0, actual.Anonymous.mi.mouseData);
            assert_eq!(SELF_EVENT_MARKER, actual.Anonymous.mi.dwExtraInfo);
        };
    }
}
