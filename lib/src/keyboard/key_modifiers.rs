use crate::keyboard::key_modifiers::KeyModifiers::All;
use crate::write_joined;
use core::ops;
use ops::BitOr;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_RMENU, VK_RSHIFT,
    VK_RWIN,
};

pub const KM_NONE: KeyModifiersState = KeyModifiersState(0);
pub const KM_LSHIFT: KeyModifiersState = KeyModifiersState(1);
pub const KM_RSHIFT: KeyModifiersState = KeyModifiersState(1 << 1);
pub const KM_LCTRL: KeyModifiersState = KeyModifiersState(1 << 2);
pub const KM_RCTRL: KeyModifiersState = KeyModifiersState(1 << 3);
pub const KM_LALT: KeyModifiersState = KeyModifiersState(1 << 4);
pub const KM_RALT: KeyModifiersState = KeyModifiersState(1 << 5);
pub const KM_LWIN: KeyModifiersState = KeyModifiersState(1 << 6);
pub const KM_RWIN: KeyModifiersState = KeyModifiersState(1 << 7);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct KeyModifiersState(u8);

impl KeyModifiersState {
    pub(crate) const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn to_string_short(&self) -> String {
        let mut text: [char; 8] = ['.'; 8];

        if self.contains(KM_LSHIFT) {
            text[0] = 'S';
        }
        if self.contains(KM_RSHIFT) {
            text[7] = 'S';
        }
        if self.contains(KM_LCTRL) {
            text[1] = 'C';
        }
        if self.contains(KM_RCTRL) {
            text[6] = 'C';
        }
        if self.contains(KM_LALT) {
            text[2] = 'A';
        }
        if self.contains(KM_RALT) {
            text[5] = 'A';
        }
        if self.contains(KM_LWIN) {
            text[3] = 'W';
        }
        if self.contains(KM_RWIN) {
            text[4] = 'W';
        }

        text.iter().collect()
    }
}

static MODIFIER_KEYS: [VIRTUAL_KEY; 8] = [
    VK_LSHIFT,
    VK_RSHIFT,
    VK_LCONTROL,
    VK_RCONTROL,
    VK_LMENU,
    VK_RMENU,
    VK_LWIN,
    VK_RWIN,
];

impl From<&[bool; 256]> for KeyModifiersState {
    fn from(keyboard_state: &[bool; 256]) -> Self {
        let value = (0..MODIFIER_KEYS.len())
            .filter(|modifier_index| {
                let vk_code = MODIFIER_KEYS[*modifier_index].0;
                keyboard_state[vk_code as usize]
            })
            .fold(0, |acc, flag_index| acc | (1 << flag_index));

        Self(value as u8)
    }
}

impl Display for KeyModifiersState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut names: Vec<&str> = Vec::new();

        if self.contains(KM_LSHIFT) {
            names.push("LEFT_SHIFT");
        }
        if self.contains(KM_RSHIFT) {
            names.push("RIGHT_SHIFT");
        }
        if self.contains(KM_LCTRL) {
            names.push("LEFT_CTRL");
        }
        if self.contains(KM_RCTRL) {
            names.push("RIGHT_CTRL");
        }
        if self.contains(KM_LALT) {
            names.push("LEFT_ALT");
        }
        if self.contains(KM_RALT) {
            names.push("RIGHT_ALT");
        }
        if self.contains(KM_LWIN) {
            names.push("LEFT_WIN");
        }
        if self.contains(KM_RWIN) {
            names.push("RIGHT_WIN");
        }

        if !names.is_empty() {
            write_joined!(f, names, " + ")
        } else {
            Ok(())
        }
    }
}

impl BitOr for KeyModifiersState {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum KeyModifiers {
    Any,
    All(KeyModifiersState),
}

impl Display for KeyModifiers {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let All(modifiers) = self {
            write!(f, "[{}]", modifiers)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key_modifiers::{
        KeyModifiers, KeyModifiersState, KM_LALT, KM_LCTRL, KM_LSHIFT, KM_LWIN, KM_NONE, KM_RALT, KM_RCTRL,
        KM_RSHIFT, KM_RWIN,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LCONTROL, VK_LSHIFT, VK_RSHIFT, VK_RWIN};

    #[macro_export]
    macro_rules! key_mod {
        ($text:literal) => {
            $text.parse::<KeyModifiersState>().unwrap()
        };
    }

    #[test]
    fn test_key_modifiers_display() {
        assert_eq!("", KM_NONE.to_string());

        assert_eq!("LEFT_SHIFT + RIGHT_WIN", (KM_LSHIFT | KM_RWIN).to_string());
        assert_eq!("RIGHT_CTRL + LEFT_ALT", (KM_LALT | KM_RCTRL).to_string());

        // assert_eq!(
        //     "SHIFT + CTRL + ALT + WIN",
        //     (KM_LSHIFT | KM_RSHIFT | KM_LWIN | KM_RWIN | KM_LALT | KM_RALT | KM_LCTRL | KM_RCTRL)
        //         .to_string()
        // );
    }

    #[test]
    fn test_key_modifiers_display_short() {
        assert_eq!("........", KM_NONE.to_string_short());
        assert_eq!("S...W...", (KM_LSHIFT | KM_RWIN).to_string_short());
        assert_eq!("..A...C.", (KM_LALT | KM_RCTRL).to_string_short());

        assert_eq!(
            "SCAWWACS",
            (KM_LSHIFT | KM_RSHIFT | KM_LWIN | KM_RWIN | KM_LALT | KM_RALT | KM_LCTRL | KM_RCTRL)
                .to_string_short()
        );
    }

    #[test]
    fn test_key_modifiers_capture() {
        let mut keys = [false; 256];
        assert_eq!(KM_NONE, KeyModifiersState::from(&keys));

        keys[VK_LSHIFT.0 as usize] = true;
        keys[VK_RSHIFT.0 as usize] = true;
        keys[VK_LCONTROL.0 as usize] = true;
        keys[VK_RWIN.0 as usize] = true;

        assert_eq!(
            KM_LSHIFT | KM_RSHIFT | KM_LCTRL | KM_RWIN,
            KeyModifiersState::from(&keys)
        );
    }

    #[test]
    fn test_keyboard_state_display() {
        assert_eq!(
            "[LEFT_SHIFT + RIGHT_WIN]",
            KeyModifiers::All(KM_LSHIFT | KM_RWIN).to_string()
        );
        assert_eq!("[]", KeyModifiers::All(KM_NONE).to_string());
        assert_eq!("", KeyModifiers::Any.to_string());
    }
}
