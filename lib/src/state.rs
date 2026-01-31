use crate::action::KeyAction;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyboardState([u64; 4]);

/* [u64; 4] is faster than [u128; 2] on most systems */

impl KeyboardState {
    pub(crate) fn new() -> Self {
        Self([0u64; 4])
    }

    pub(crate) fn update(&mut self, action: KeyAction) {
        let index = action.key.vk.0;
        if action.transition.into_bool() {
            self.set_bit(index);
        } else {
            self.clear_bit(index);
        }
    }

    #[inline]
    pub(crate) fn is_set(&self, index: u8) -> bool {
        let (part_index, bit_index) = self.bit_pos(index);
        unsafe {
            let part = self.0.get_unchecked(part_index);
            (*part >> bit_index) & 1 == 1
        }
        // (self.0[part_index] >> bit_index) & 1 == 1 //slower
    }

    #[inline]
    fn set_bit(&mut self, index: u8) {
        let (part_index, bit_index) = self.bit_pos(index);
        unsafe {
            let part = self.0.get_unchecked_mut(part_index);
            *part |= 1u64 << bit_index;
        }
        // self.0[part_index] |= 1u64 << bit_index;  //slower
    }

    #[inline]
    fn clear_bit(&mut self, index: u8) {
        let (part_index, bit_index) = self.bit_pos(index);
        unsafe {
            let part = self.0.get_unchecked_mut(part_index);
            *part &= !(1u64 << bit_index);
        }
        // self.0[part_index] &= !(1u64 << bit_index); //slower
    }

    #[inline]
    fn bit_pos(&self, index: u8) -> (usize, u8) {
        ((index / 64) as usize, index % 64)
    }
}

impl fmt::Binary for KeyboardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, &part) in self.0.iter().enumerate().rev() {
            if i < 3 {
                write!(f, "_{:064b}", part)?;
            } else {
                write!(f, "{:064b}", part)?;
            }
        }
        Ok(())
    }
}

impl fmt::UpperHex for KeyboardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, &part) in self.0.iter().enumerate().rev() {
            if i < 3 {
                write!(f, "_{:016X}", part)?;
            } else {
                write!(f, "{:016X}", part)?;
            }
        }
        Ok(())
    }
}


#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::key::{KEY_0, KEY_END, KEY_F1};
    use crate::vk::VirtualKey;

    pub fn state_from_keys(keys: &[VirtualKey]) -> KeyboardState {
        let mut this = KeyboardState::new();
        keys.iter().for_each(|key| this.set_bit(key.0));
        this
    }

    pub fn state_to_keys(state: &KeyboardState) -> Vec<VirtualKey> {
        let mut result = vec![];
        for index in 0..255 {
            if state.is_set(index) {
                result.push(VirtualKey(index))
            }
        }
        result
    }

    #[test]
    fn test_keyboard_state_get_set_bit() {
        let mut state = KeyboardState::new();
        state.set_bit(1);
        state.clear_bit(41);

        assert!(state.is_set(1));
        assert!(!state.is_set(41));

        state.clear_bit(1);
        state.set_bit(41);

        assert!(!state.is_set(1));
        assert!(state.is_set(41));
    }

    #[test]
    fn test_keyboard_state_to_keys() {
        let state = state_from_keys(&[KEY_F1.vk, KEY_END.vk, KEY_0.vk]);

        // order is not guaranteed
        assert_eq!(vec![KEY_END.vk, KEY_0.vk, KEY_F1.vk], state_to_keys(&state));
    }

    #[test]
    fn test_keyboard_state_hex_format() {
        let state = state_from_keys(&[KEY_F1.vk, KEY_END.vk, KEY_0.vk]);

        // println!("{:X}", state);
        assert_eq!(
            "0000000000000000_0000000000000000_0001000000000000_0001000800000000",
            format!("{:X}", state)
        );
    }

    #[test]
    fn test_keyboard_state_bin_format() {
        let state = state_from_keys(&[KEY_F1.vk, KEY_END.vk, KEY_0.vk]);

        // println!("{:b}", state);
        assert_eq!(
            "0000000000000000000000000000000000000000000000000000000000000000_0000000000000000000000000000000000000000000000000000000000000000_0000000000000001000000000000000000000000000000000000000000000000_0000000000000001000000000000100000000000000000000000000000000000",
            format!("{:b}", state)
        );
    }
}
