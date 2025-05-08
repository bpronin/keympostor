use crate::key::{VirtualKey, MAX_VK_CODE};
use crate::key_action::KeyAction;
use crate::key_action::KeyTransition;
use std::fmt;
use std::fmt::{Display, Formatter};
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardState;
use KeyTransition::{Down, Up};

pub(crate) const UP_STATE: u8 = 0;
pub(crate) const DOWN_STATE: u8 = 1;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct KeyboardState {
    keys: [u8; MAX_VK_CODE],
}

impl KeyboardState {
    pub(crate) fn new(keys: [u8; MAX_VK_CODE]) -> Self {
        Self { keys }
    }

    pub fn capture() -> Self {
        let mut keys = [UP_STATE; MAX_VK_CODE];
        unsafe { GetKeyboardState(&mut keys) }.unwrap();
        Self { keys }
    }

    fn from_actions(actions: &[KeyAction]) -> Self {
        // todo: optimize!
        let mut keys = [UP_STATE; MAX_VK_CODE];
        actions.iter().for_each(|action| {
            let vk_code = action.key.as_virtual_key().unwrap().value;
            keys[vk_code as usize] = match action.transition {
                Up => UP_STATE,
                Down => DOWN_STATE,
            }
        });
        Self::new(keys)
    }

    // pub(crate) fn get_virtual_keys(&self) -> Vec<&'static VirtualKey> {
    //     let mut keys = vec![];
    //     for vk_code in 0..MAX_VK_CODE {
    //         if &self.keys[vk_code] & DOWN_STATE != 0 {
    //             keys.push(VirtualKey::from_code(vk_code as u8).unwrap())
    //         }
    //     }
    //     keys
    // }

    pub fn has_state(&self, actions: &[KeyAction]) -> bool {
        self == &Self::from_actions(actions)
    }
}

impl Display for KeyboardState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for vk_code in 0..MAX_VK_CODE {
            // if &self.keys[vk_code] & DOWN_STATE != 0 {
            if self.keys[vk_code] == DOWN_STATE {
                let vk = VirtualKey::from_code(vk_code as u8).unwrap();
                write!(f, "{}; ", vk)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::key::{KeyCode, VirtualKey, MAX_VK_CODE};
    use crate::key_action::KeyTransition::{Down, Up};
    use crate::keyboard_state::KeyboardState;
    use crate::keyboard_state::{KeyAction, DOWN_STATE, UP_STATE};
    use crate::{assert_not, key_act};
    use std::array::from_fn;
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_MENU, VK_RETURN, VK_SHIFT};
    use KeyCode::VK;

    #[test]
    fn test_keyboard_default_state() {
        let state = KeyboardState::new([UP_STATE; MAX_VK_CODE]);

        let all_up: [KeyAction; MAX_VK_CODE] = from_fn(|vk_code| KeyAction {
            key: VK(VirtualKey::from_code(vk_code as u8).unwrap()),
            transition: Up,
        });
        assert!(state.has_state(&all_up));

        let all_down: [KeyAction; MAX_VK_CODE] = from_fn(|vk_code| KeyAction {
            key: VK(VirtualKey::from_code(vk_code as u8).unwrap()),
            transition: Down,
        });
        assert_not!(state.has_state(&all_down));

        assert_not!(state.has_state(&[
            key_act!("VK_RETURN↑"),
            key_act!("VK_SHIFT↑"),
            key_act!("VK_MENU↓"),
        ]));

        assert!(state.has_state(&[]));
    }

    #[test]
    fn test_keyboard_state() {
        let mut keys = [UP_STATE; MAX_VK_CODE];
        keys[VK_RETURN.0 as usize] = DOWN_STATE;
        keys[VK_SHIFT.0 as usize] = DOWN_STATE;
        keys[VK_MENU.0 as usize] = UP_STATE;
        let state = KeyboardState::new(keys);

        assert_not!(state.has_state(&[key_act!("VK_RETURN * ")]));
        assert_not!(state.has_state(&[key_act!("VK_MENU ↑ ")]));
        assert_not!(state.has_state(&[]));

        assert!(state.has_state(&[
            key_act!("VK_RETURN ↓ "),
            key_act!("VK_SHIFT ↓ "),
            key_act!("VK_MENU ↑ "),
        ]));

        assert_not!(state.has_state(&[
            key_act!("VK_A ↓ "),
            key_act!("VK_B ↓ "),
            key_act!("VK_C ↑ "),
        ]));
    }

    #[test]
    fn test_keyboard_state_to_actions() {
        let mut keys = [UP_STATE; MAX_VK_CODE];
        keys[VK_RETURN.0 as usize] = DOWN_STATE;
        keys[VK_SHIFT.0 as usize] = DOWN_STATE;
        keys[VK_MENU.0 as usize] = UP_STATE;
        let state = KeyboardState::new(keys);

        println!("{state}");
    }
}
