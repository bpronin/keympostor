use crate::keyboard::action::{KeyAction, KeyActionSequence};
use crate::keyboard::event::SELF_EVENT_MARKER;
use crate::keyboard::key::{
    KEY_LEFT_BUTTON, KEY_MIDDLE_BUTTON, KEY_MOUSE_X, KEY_MOUSE_Y, KEY_RIGHT_BUTTON, KEY_WHEEL_X,
    KEY_WHEEL_Y, KEY_XBUTTON1, KEY_XBUTTON2,
};
use crate::keyboard::sc::ScanCode;
use crate::keyboard::transition::KeyTransition::{Down, Up};
use crate::keyboard::vk::VirtualKey;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP,
    MOUSEINPUT, MOUSE_EVENT_FLAGS,
};
use windows::Win32::UI::WindowsAndMessaging::{XBUTTON1, XBUTTON2};

pub fn build_input(seq: &KeyActionSequence, delta: i32) -> Vec<INPUT> {
    seq.actions
        .iter()
        .map(|a| build_action_input(a, delta))
        .collect()
}

fn build_action_input(action: &KeyAction, delta: i32) -> INPUT {
    build_mouse_button_input(action).unwrap_or(
        build_mouse_x_button_input(action).unwrap_or(
            build_mouse_move_input(action, delta).unwrap_or(
                build_mouse_wheel_input(action, delta).unwrap_or(build_key_input(action)),
            ),
        ),
    )
}

fn build_key_input(action: &KeyAction) -> INPUT {
    let virtual_key = VirtualKey::from(action.key);
    let scan_code = ScanCode::from(action.key);

    let mut flags = KEYEVENTF_SCANCODE;
    if scan_code.is_extended {
        flags |= KEYEVENTF_EXTENDEDKEY
    }
    if action.transition == Up {
        flags |= KEYEVENTF_KEYUP;
    }

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: virtual_key.into(),
                wScan: scan_code.into(),
                dwFlags: flags,
                dwExtraInfo: SELF_EVENT_MARKER,
                ..Default::default()
            },
        },
    }
}

fn build_mouse_move_input(action: &KeyAction, delta: i32) -> Option<INPUT> {
    if action.key == &KEY_MOUSE_X {
        build_mouse_input(delta, 0, MOUSEEVENTF_MOVE, 0)
    } else if action.key == &KEY_MOUSE_Y {
        build_mouse_input(0, delta, MOUSEEVENTF_MOVE, 0)
    } else {
        return None;
    }
}

fn build_mouse_wheel_input(action: &KeyAction, delta: i32) -> Option<INPUT> {
    if action.key == &KEY_WHEEL_X {
        build_mouse_input(delta, 0, MOUSEEVENTF_WHEEL, 0)
    } else if action.key == &KEY_WHEEL_Y {
        build_mouse_input(0, delta, MOUSEEVENTF_WHEEL, 0)
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

    build_mouse_input(0, 0, flags, 0)
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

    build_mouse_input(0, 0, flags, data as u32)
}

fn build_mouse_input(x: i32, y: i32, flags: MOUSE_EVENT_FLAGS, data: u32) -> Option<INPUT> {
    Some(INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: x,
                dy: y,
                dwFlags: flags,
                mouseData: data,
                dwExtraInfo: SELF_EVENT_MARKER,
                ..Default::default()
            },
        },
    })
}

#[cfg(test)]
mod tests {
    use crate::keyboard::action::KeyAction;
    use crate::keyboard::event::SELF_EVENT_MARKER;
    use crate::keyboard::input::build_key_input;
    use crate::keyboard::sc::ScanCode;
    use crate::{key_action, sc_key};
    use std::str::FromStr;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_KEYBOARD, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
        VK_RETURN,
    };

    #[test]
    fn test_key_action_create_input() {
        let actual: INPUT = build_key_input(&key_action!("ENTER*"));
        unsafe {
            assert_eq!(INPUT_KEYBOARD, actual.r#type);
            assert_eq!(VK_RETURN, actual.Anonymous.ki.wVk);
            assert_eq!(actual.Anonymous.ki.wScan, sc_key!("SC_ENTER").into());
            assert_eq!(KEYEVENTF_SCANCODE, actual.Anonymous.ki.dwFlags);
            assert_eq!(SELF_EVENT_MARKER, actual.Anonymous.ki.dwExtraInfo);
        };

        let actual: INPUT = build_key_input(&key_action!("NUM_ENTER^"));
        unsafe {
            assert_eq!(INPUT_KEYBOARD, actual.r#type);
            assert_eq!(VK_RETURN, actual.Anonymous.ki.wVk);
            assert_eq!(actual.Anonymous.ki.wScan, sc_key!("SC_NUM_ENTER").into(),);
            assert_eq!(
                KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
                actual.Anonymous.ki.dwFlags
            );
            assert_eq!(SELF_EVENT_MARKER, actual.Anonymous.ki.dwExtraInfo);
        };

        // let actual: INPUT = build_mouse_input(&key_action!("MOUSE_Y^"), 10);
        // unsafe {
        //     assert_eq!(INPUT_MOUSE, actual.r#type);
        // assert_eq!(VK_RETURN, actual.Anonymous.ki.wVk);
        // assert_eq!(
        //     sc_key!("SC_NUM_ENTER").ext_value(),
        //     actual.Anonymous.ki.wScan
        // );
        // assert_eq!(
        //     KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
        //     actual.Anonymous.ki.dwFlags
        // );
        // assert_eq!(
        //     SELF_EVENT_MARKER.as_ptr(),
        //     actual.Anonymous.mi.dwExtraInfo as *const u8
        // );
        // };
    }
}
