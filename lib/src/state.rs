use crate::action::KeyAction;
use fmt::Display;
use serde::Serializer;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyboardState([u64; 4]);  /* [u64; 4] is faster than [u128; 2] on most systems */

impl KeyboardState {
    pub(crate) fn new() -> Self {
        Self([0u64; 4])
    }

    pub(crate) fn from_bits(bits: &[u8]) -> Self {
        let mut this = Self::new();
        bits.iter().for_each(|b| this.set(*b));
        this
    }

    pub(crate) fn update(&mut self, action: KeyAction) {
        let index = action.key.vk.0;
        if action.transition.into_bool() {
            self.set(index);
        } else {
            self.unset(index);
        }
    }

    #[inline]
    pub(crate) fn is_set(&self, index: u8) -> bool {
        let (part_index, bit_index) = self.pos(index);
        unsafe {
            let part = self.0.get_unchecked(part_index);
            (*part >> bit_index) & 1 == 1
        }
        // (self.0[part_index] >> bit_index) & 1 == 1 //slower
    }

    #[inline]
    fn set(&mut self, index: u8) {
        let (part_index, bit_index) = self.pos(index);
        unsafe {
            let part = self.0.get_unchecked_mut(part_index);
            *part |= 1u64 << bit_index;
        }
        // self.0[part_index] |= 1u64 << bit_index;  //slower
    }

    #[inline]
    fn unset(&mut self, index: u8) {
        let (part_index, bit_index) = self.pos(index);
        unsafe {
            let part = self.0.get_unchecked_mut(part_index);
            *part &= !(1u64 << bit_index);
        }
        // self.0[part_index] &= !(1u64 << bit_index); //slower
    }

    #[inline]
    fn pos(&self, index: u8) -> (usize, u8) {
        ((index / 64) as usize, index % 64)
    }
}

impl Display for KeyboardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:016x} {:016x} {:016x} {:016x}",
            self.0[3], self.0[2], self.0[1], self.0[0]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_state_display() {
        let state = KeyboardState::from_bits(&[1, 255]);
        assert_eq!(
            "8000000000000000 0000000000000000 0000000000000000 0000000000000002",
            state.to_string()
        );
    }

    #[test]
    fn test_keyboard_state_get_set_bit() {
        let mut state = KeyboardState::new();
        state.set(1);
        state.unset(41);

        assert!(state.is_set(1));
        assert!(!state.is_set(41));

        state.unset(1);
        state.set(41);

        assert!(!state.is_set(1));
        assert!(state.is_set(41));
    }
}
