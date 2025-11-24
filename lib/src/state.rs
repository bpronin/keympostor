use crate::key::Key;
use crate::transition::KeyTransition;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyboardState([bool; 256]);

impl KeyboardState {
    pub(crate) const fn new() -> Self {
        Self([false; 256])
    }

    pub(crate) fn set(&mut self, key: &Key, transition: KeyTransition) {
        self.set_vk(key.vk.0, transition);
    }

    pub(crate) fn set_vk(&mut self, vk: u8, transition: KeyTransition) {
        self.0[vk as usize] = transition.into_bool();
    }

    // pub(crate) fn get(&self, vk: u8) -> KeyTransition {
    //     KeyTransition::from_bool(self.is_set(vk))
    // }

    pub(crate) fn is_set(&self, vk: u8) -> bool {
        self.0[vk as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bit256(pub [u128; 2]);

impl Bit256 {
    pub const fn new() -> Self {
        Self([0, 0])
    }

    pub(crate) const fn set(&mut self, index: u8, value: bool) {
        let (chunk, bit) = (index / 128, index % 128);
        if value {
            self.0[chunk as usize] |= 1u128 << bit;
        } else {
            self.0[chunk as usize] &= !(1u128 << bit);
        }
    }

    pub(crate) const fn get(&self, index: u8) -> bool {
        let (chunk, bit) = (index / 128, index % 128);
        (self.0[chunk as usize] >> bit) & 1 != 0
    }
}

// impl Serialize for KeyboardState {
//     serialize_to_string!();
// }
//
// impl<'de> Deserialize<'de> for KeyboardState {
//     deserialize_from_string!();
// }

// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// pub struct KeyboardState(pub [u128; 2]);
//
// impl KeyboardState {
//     pub const fn new() -> Self {
//         Self([0, 0])
//     }
//
//     pub(crate) fn set(&mut self, key: &Key, transition: KeyTransition) {
//         let index = key.vk.0 as usize;
//         let (chunk, bit) = (index / 128, index % 128);
//         if transition.into_bool() {
//             self.0[chunk] |= 1u128 << bit;
//         } else {
//             self.0[chunk] &= !(1u128 << bit);
//         }
//     }
//
//     pub(crate) fn is_down(&self, index: usize) -> bool {
//         let (chunk, bit) = (index / 128, index % 128);
//         (self.0[chunk] >> bit) & 1 != 0
//     }
// }

// impl Into<[bool;256]> for KeyboardState {
//     fn into(self) -> [bool;256] {
//         let mut bools = [false; 256];
//         for i in 0..256 {
//             let (chunk, bit) = (i / 128, i % 128);
//             bools[i] = (self.0[chunk] >> bit) & 1 != 0;
//         }
//         bools
//     }
// }
//
// impl From<[bool;256]> for KeyboardState {
//     fn from(value: [bool;256]) -> Self {
//         let mut inner = [0u128; 2];
//         for i in 0..256 {
//             if value[i] {
//                 let (chunk, bit) = (i / 128, i % 128);
//                 inner[chunk] |= 1u128 << bit;
//             }
//         }
//         Self(inner)
//     }
// }
//
// impl fmt::Display for KeyboardState {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         writeln!(f, "{:0128b}", self.0[1])?;
//         writeln!(f, "{:0128b}", self.0[0])?;
//         Ok(())
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit256_display() {
        // let b = Bit256::from_bools([true; 256]);
        let mut b = KeyboardState::new();
        // b.set(255, true);
        // println!("{}", b);
        // assert_eq!()
    }
}
