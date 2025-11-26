use crate::action::KeyAction;
use fmt::Display;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyboardState([u128; 2]);

impl KeyboardState {
    pub(crate) const fn new() -> Self {
        Self([0u128; 2])
    }
    
    pub(crate) fn update(&mut self, action: KeyAction) {
        self.set(action.key.vk.0, action.transition.into_bool())
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

impl Display for KeyboardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:0128b}", self.0[1])?;
        writeln!(f, "{:0128b}", self.0[0])?;
        Ok(())
    }
}
