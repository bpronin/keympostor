use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardState, VIRTUAL_KEY, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL,
    VK_RMENU, VK_RSHIFT, VK_RWIN,
};

pub const KM_NONE: KeyModifiers = KeyModifiers(0);
pub const KM_LEFT_SHIFT: KeyModifiers = KeyModifiers(1);
pub const KM_RIGHT_SHIFT: KeyModifiers = KeyModifiers(1 << 1);
pub const KM_LEFT_CONTROL: KeyModifiers = KeyModifiers(1 << 2);
pub const KM_RIGHT_CONTROL: KeyModifiers = KeyModifiers(1 << 3);
pub const KM_LEFT_ALT: KeyModifiers = KeyModifiers(1 << 4);
pub const KM_RIGHT_ALT: KeyModifiers = KeyModifiers(1 << 5);
pub const KM_LEFT_WIN: KeyModifiers = KeyModifiers(1 << 6);
pub const KM_RIGHT_WIN: KeyModifiers = KeyModifiers(1 << 7);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct KeyModifiers(u8);

impl KeyModifiers {
    pub(crate) fn capture_state() -> Self {
        let mut state = [0u8; 256];
        unsafe { GetKeyboardState(&mut state) }.unwrap();

        let mut this = KM_NONE;
        this.capture_key_state(&state, VK_RSHIFT, KM_RIGHT_SHIFT);
        this.capture_key_state(&state, VK_LSHIFT, KM_LEFT_SHIFT);
        this.capture_key_state(&state, VK_RCONTROL, KM_RIGHT_CONTROL);
        this.capture_key_state(&state, VK_LCONTROL, KM_LEFT_CONTROL);
        this.capture_key_state(&state, VK_RMENU, KM_RIGHT_ALT);
        this.capture_key_state(&state, VK_LMENU, KM_LEFT_ALT);
        this.capture_key_state(&state, VK_RWIN, KM_RIGHT_WIN);
        this.capture_key_state(&state, VK_LWIN, KM_LEFT_WIN);
        this
    }

    #[inline]
    fn capture_key_state(&mut self, state: &[u8; 256], key: VIRTUAL_KEY, flag: KeyModifiers) {
        if (&state[key.0 as usize] & 0x80) != 0 {
            self.0 |= flag.0
        }
    }

    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for KeyModifiers {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitAnd for KeyModifiers {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl BitOrAssign for KeyModifiers {
    fn bitor_assign(&mut self, other: Self) {
        self.0.bitor_assign(other.0)
    }
}

impl BitAndAssign for KeyModifiers {
    fn bitand_assign(&mut self, other: Self) {
        self.0.bitand_assign(other.0)
    }
}

impl Not for KeyModifiers {
    type Output = Self;
    fn not(self) -> Self {
        Self(self.0.not())
    }
}

impl Serialize for KeyModifiers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut values: Vec<&str> = Vec::new();

        if self.contains(KM_LEFT_SHIFT) {
            values.push("LEFT SHIFT")
        }
        if self.contains(KM_RIGHT_SHIFT) {
            values.push("RIGHT SHIFT")
        }
        if self.contains(KM_LEFT_CONTROL) {
            values.push("LEFT CONTROL")
        }
        if self.contains(KM_RIGHT_CONTROL) {
            values.push("RIGHT CONTROL")
        }
        if self.contains(KM_LEFT_ALT) {
            values.push("LEFT ALT")
        }
        if self.contains(KM_RIGHT_ALT) {
            values.push("RIGHT ALT")
        }
        if self.contains(KM_LEFT_WIN) {
            values.push("LEFT WIN")
        }
        if self.contains(KM_RIGHT_WIN) {
            values.push("RIGHT WIN")
        }

        values.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for KeyModifiers {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut this = KM_NONE;
        let values: Vec<String> = Vec::deserialize(deserializer)?;

        for value in values {
            match value.to_uppercase().as_str() {
                "LEFT SHIFT" => this |= KM_LEFT_SHIFT,
                "RIGHT SHIFT" => this |= KM_RIGHT_SHIFT,
                "SHIFT" => this |= KM_LEFT_SHIFT | KM_RIGHT_SHIFT,
                "LEFT CONTROL" => this |= KM_LEFT_CONTROL,
                "RIGHT CONTROL" => this |= KM_RIGHT_CONTROL,
                "CONTROL" => this |= KM_LEFT_CONTROL | KM_RIGHT_CONTROL,
                "LEFT ALT" => this |= KM_LEFT_ALT,
                "RIGHT ALT" => this |= KM_RIGHT_ALT,
                "ALT" => this |= KM_LEFT_ALT | KM_RIGHT_ALT,
                "LEFT WIN" => this |= KM_LEFT_WIN,
                "RIGHT WIN" => this |= KM_RIGHT_WIN,
                "WIN" => this |= KM_LEFT_WIN | KM_RIGHT_WIN,
                &_ => Err(Error::invalid_value(
                    Unexpected::Str(&value),
                    &"Keyboard modifiers",
                ))?,
            }
        }
        Ok(this)
    }
}

impl Display for KeyModifiers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:->10b}", &self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::key_modifier::{KM_LEFT_ALT, KM_LEFT_CONTROL, KM_LEFT_SHIFT, KM_NONE};

    #[test]
    fn key_modifiers() {
        let mut modifiers = KM_NONE;

        modifiers |= KM_LEFT_SHIFT;
        assert!(modifiers.contains(KM_LEFT_SHIFT));
        assert!(!modifiers.contains(KM_LEFT_ALT));

        modifiers |= KM_LEFT_ALT;
        assert!(modifiers.contains(KM_LEFT_SHIFT));
        assert!(modifiers.contains(KM_LEFT_ALT));
        assert!(modifiers.contains(KM_LEFT_ALT | KM_LEFT_SHIFT));
        assert!(!modifiers.contains(KM_LEFT_CONTROL | KM_LEFT_SHIFT));

        // println!("{:010b}", modifiers.0);
    }
}
