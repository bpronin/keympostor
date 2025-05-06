use crate::key::{KeyCode, VirtualKey};
use std::fmt::Display;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardState;
use KeyCode::{SC, VK};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct KeyboardState {
    keys: [u8; 256],
}

impl KeyboardState {
    pub fn capture() -> Self {
        let mut keys = [0; 256];
        unsafe { GetKeyboardState(&mut keys) }.unwrap();
        Self { keys }
    }

    pub fn is_key_down(&self, key: &KeyCode) -> bool {
        match key {
            VK(vk) => self.is_vk_down(vk),
            SC(sc) => {
                if let Some(vk) = sc.to_virtual_key() {
                    self.is_vk_down(vk)
                } else {
                    false
                }
            }
        }
    }

    fn is_vk_down(&self, virtual_key: &VirtualKey) -> bool {
        self.keys[virtual_key.value as usize] & 0x80 != 0
    }

    pub fn are_keys_down(&self, keys: &[&KeyCode]) -> bool {
        for key in keys {
            if !self.is_key_down(key) {
                return false;
            }
        }
        true
    }
}
