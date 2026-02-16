use crate::action::KeyAction;
use crate::error::KeyError;
use crate::key_code::{virtual_key_as_str, virtual_key_from_str};
use crate::{deserialize_from_string, key_error, serialize_to_string};
use serde::Deserializer;
use serde::Serializer;
use serde::{de, Deserialize, Serialize};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

/* Using [u64; 4] because it is faster than [u128; 2] on most systems */
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyboardState([u64; 4]);

impl KeyboardState {
    pub(crate) fn new() -> Self {
        Self([0u64; 4])
    }

    pub(crate) fn update(&mut self, action: KeyAction) {
        let index = action.key.vk();
        if action.transition.into_bool() {
            self.set_bit(index);
        } else {
            self.clear_bit(index);
        }
    }

    #[inline]
    pub(crate) fn is_bit_set(&self, index: u8) -> bool {
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

impl FromStr for KeyboardState {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut this = Self::new();

        for part in s.trim().split('+') {
            let vk = virtual_key_from_str(part.trim())
                .ok_or(key_error!("Invalid virtual key name: {}", part))?;
            this.set_bit(vk);
        }

        Ok(this)
    }
}

impl Display for KeyboardState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = (0..255)
            .filter(|&index| self.is_bit_set(index))
            .map(|k| virtual_key_as_str(k))
            .collect::<Vec<&str>>()
            .join(" + ");

        write!(f, "{}", s)
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

impl Serialize for KeyboardState {
    serialize_to_string!();
}

impl<'de> Deserialize<'de> for KeyboardState {
    deserialize_from_string!();
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::key::Key;
    use crate::key::Key::{Digit0, End, F1};

    pub fn state_from_keys(keys: &[Key]) -> KeyboardState {
        let mut this = KeyboardState::new();
        keys.iter().for_each(|key| this.set_bit(key.vk()));
        this
    }

    #[test]
    fn test_keyboard_state_get_set_bit() {
        let mut state = KeyboardState::new();
        state.set_bit(1);
        state.clear_bit(41);

        assert!(state.is_bit_set(1));
        assert!(!state.is_bit_set(41));

        state.clear_bit(1);
        state.set_bit(41);

        assert!(!state.is_bit_set(1));
        assert!(state.is_bit_set(41));
    }

    #[test]
    fn test_keyboard_state_to_string() {
        let state = state_from_keys(&[F1, End, Digit0]);

        assert_eq!("VK_END + VK_0 + VK_F1", state.to_string());
        // println!("{}", state);
    }

    #[test]
    fn test_keyboard_state_from_string() {
        let state = KeyboardState::from_str("VK_END + VK_0 + VK_F1").unwrap();
        assert_eq!(state_from_keys(&[F1, End, Digit0]), state);
    }

    #[test]
    fn test_keyboard_state_hex_format() {
        let state = state_from_keys(&[F1, End, Digit0]);

        // println!("{:X}", state);
        assert_eq!(
            "0000000000000000_0000000000000000_0001000000000000_0001000800000000",
            format!("{:X}", state)
        );
    }

    #[test]
    fn test_keyboard_state_bin_format() {
        let state = state_from_keys(&[F1, End, Digit0]);

        // println!("{:b}", state);
        assert_eq!(
            "0000000000000000000000000000000000000000000000000000000000000000_0000000000000000000000000000000000000000000000000000000000000000_0000000000000001000000000000000000000000000000000000000000000000_0000000000000001000000000000100000000000000000000000000000000000",
            format!("{:b}", state)
        );
    }
}
