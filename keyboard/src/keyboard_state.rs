use crate::key::{KeyCode, VirtualKey, MAX_VK_CODE};
use crate::key_action::KeyAction;
use crate::key_action::KeyTransition;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardState;
use KeyCode::{SC, VK};

pub(crate) const UP_STATE: u8 = 0x80;
pub(crate) const DOWN_STATE: u8 = 0;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct KeyboardState {
    keys: [u8; MAX_VK_CODE],
}

impl KeyboardState {
    pub(crate) fn new(keys: [u8; MAX_VK_CODE]) -> Self {
        Self { keys }
    }

    pub fn capture() -> Self {
        let mut keys = [DOWN_STATE; MAX_VK_CODE];
        unsafe { GetKeyboardState(&mut keys) }.unwrap();
        Self::new(keys)
    }

    pub fn has_state(&self, actions: &[KeyAction]) -> bool {
        for key in actions {
            if !self.has_key_state(key) {
                return false;
            }
        }
        true
    }

    fn has_key_state(&self, action: &KeyAction) -> bool {
        let actual = self.key_transition(&action.key);
        action.transition == actual
    }

    fn key_transition(&self, key: &KeyCode) -> KeyTransition {
        match key {
            VK(vk) => self.vk_transition(vk),
            SC(sc) => self.vk_transition(sc.to_virtual_key().unwrap()),
        }
    }

    fn vk_transition(&self, virtual_key: &VirtualKey) -> KeyTransition {
        let up = self.keys[virtual_key.value as usize] & UP_STATE != 0;
        KeyTransition::from_bool(up)
    }

    fn is_vk_code_down(&self, vk_code: u16) -> bool {
        self.keys[vk_code as usize] & UP_STATE != 0
    }
}

#[cfg(test)]
mod tests {
    use crate::key::MAX_VK_CODE;
    use crate::key_action;
    use crate::keyboard_state::KeyboardState;
    use crate::keyboard_state::{KeyAction, DOWN_STATE, UP_STATE};
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_MENU, VK_RETURN, VK_SHIFT};

    #[test]
    fn test_keyboard_state() {
        let mut keys = [UP_STATE; MAX_VK_CODE];
        keys[VK_RETURN.0 as usize] = DOWN_STATE;
        keys[VK_SHIFT.0 as usize] = DOWN_STATE;
        keys[VK_MENU.0 as usize] = UP_STATE;
        let state = KeyboardState::new(keys);

        assert!(state.has_key_state(&key_action!("VK_RETURN*")));
        assert!(state.has_key_state(&key_action!("VK_SHIFT*")));
        assert!(state.has_key_state(&key_action!("VK_MENU^")));

        let expected = [
            key_action!("VK_RETURN*"),
            key_action!("VK_SHIFT*"),
            key_action!("VK_MENU^"),
        ];
        assert!(state.has_state(&expected));

        let expected = [key_action!("VK_RETURN*")];
        assert!(state.has_state(&expected));

        let unexpected = [key_action!("VK_RETURN^")];
        assert!(!state.has_state(&unexpected));

        let unexpected = [
            key_action!("VK_A*"),
            key_action!("VK_B*"),
            key_action!("VK_C^"),
        ];
        assert!(!state.has_state(&unexpected));
    }
}
