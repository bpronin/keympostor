use crate::action::KeyAction;
use crate::error::KeyError;
use crate::key::Key;
use crate::transition::KeyTransition;
use crate::{deserialize_from_string, key_error, serialize_to_string};
use serde::Deserializer;
use serde::Serializer;
use serde::{de, Deserialize, Serialize};
use std::fmt::{Binary, Display, Formatter, UpperHex};
use std::hash::Hash;
use std::str::FromStr;
use KeyTransition::{Down, Up};

/* Using [u64; 4] because it is faster than [u128; 2] on most systems */
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct KeyboardState([u64; 4]);

impl KeyboardState {
    pub(crate) fn exclude(&mut self, key: Key) {
        self.clear_bit(key as u8);
    }

    pub(crate) fn update(&mut self, action: &KeyAction) {
        match action.transition {
            Down => self.set_bit(action.key as u8),
            Up => self.clear_bit(action.key as u8),
        };
    }

    #[inline]
    fn is_bit_set(&self, index: u8) -> bool {
        let (part_index, bit_index) = self.bit_pos(index);
        unsafe {
            let part = self.0.get_unchecked(part_index);
            (*part >> bit_index) & 1 == 1
        }
    }

    #[inline]
    fn set_bit(&mut self, index: u8) {
        let (part_index, bit_index) = self.bit_pos(index);
        unsafe {
            let part = self.0.get_unchecked_mut(part_index);
            *part |= 1u64 << bit_index;
        }
    }

    #[inline]
    fn clear_bit(&mut self, index: u8) {
        let (part_index, bit_index) = self.bit_pos(index);
        unsafe {
            let part = self.0.get_unchecked_mut(part_index);
            *part &= !(1u64 << bit_index);
        }
    }

    #[inline]
    fn bit_pos(&self, index: u8) -> (usize, u8) {
        ((index / 64) as usize, index % 64)
    }
}

impl FromStr for KeyboardState {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self::default());
        }

        let mut this = Self::default();
        for part in s.split('+') {
            let name = part.trim();
            let key = Key::from_str(name).ok_or(key_error!("Invalid key name: `{}`", name))?;
            this.set_bit(key as u8);
        }
        Ok(this)
    }
}

impl Display for KeyboardState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = (0..=255)
            .filter(|&index| self.is_bit_set(index))
            .map(|k| Key::from_index(k).expect("Invalid key index").as_str())
            .collect::<Vec<&str>>()
            .join(" + ");

        write!(f, "{}", s)
    }
}

impl Binary for KeyboardState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

impl UpperHex for KeyboardState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
    use crate::key::Key::{Digit0, End, F1};

    pub fn kb_state_from_keys(keys: &[Key]) -> KeyboardState {
        let mut this = KeyboardState::default();
        for key in keys {
            this.set_bit(*key as u8);
        }
        this
    }

    #[test]
    fn test_keyboard_state_get_set_bit() {
        let mut state = KeyboardState::default();
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
        assert_eq!(
            "END + 0 + F1",
            kb_state_from_keys(&[F1, End, Digit0]).to_string()
        );

        assert_eq!("", KeyboardState::default().to_string());
    }

    #[test]
    fn test_keyboard_state_from_string() {
        assert_eq!(
            Ok(kb_state_from_keys(&[F1, End, Digit0])),
            KeyboardState::from_str("END + 0 + F1")
        );

        assert_eq!(Ok(KeyboardState::default()), KeyboardState::from_str(""));
    }

    #[test]
    fn test_keyboard_state_hex_format() {
        assert_eq!(
            "0000000000000000_0000000000000000_0001000000000000_0001000800000000",
            format!("{:X}", kb_state_from_keys(&[F1, End, Digit0]))
        );
    }

    #[test]
    fn test_keyboard_state_bin_format() {
        assert_eq!(
            "0000000000000000000000000000000000000000000000000000000000000000_\
            0000000000000000000000000000000000000000000000000000000000000000_\
            0000000000000001000000000000000000000000000000000000000000000000_\
            0000000000000001000000000000100000000000000000000000000000000000",
            format!("{:b}", kb_state_from_keys(&[F1, End, Digit0]))
        );
    }
}
