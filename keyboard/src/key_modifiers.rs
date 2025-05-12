use crate::write_joined;
use core::ops;
use ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardState, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_RMENU, VK_RSHIFT,
    VK_RWIN,
};

pub const KM_NONE: KeyModifiers = KeyModifiers(0);
pub const KM_LSHIFT: KeyModifiers = KeyModifiers(1);
pub const KM_RSHIFT: KeyModifiers = KeyModifiers(1 << 1);
pub const KM_LCTRL: KeyModifiers = KeyModifiers(1 << 2);
pub const KM_RCTRL: KeyModifiers = KeyModifiers(1 << 3);
pub const KM_LALT: KeyModifiers = KeyModifiers(1 << 4);
pub const KM_RALT: KeyModifiers = KeyModifiers(1 << 5);
pub const KM_LWIN: KeyModifiers = KeyModifiers(1 << 6);
pub const KM_RWIN: KeyModifiers = KeyModifiers(1 << 7);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct KeyModifiers(u8);

impl KeyModifiers {
    pub(crate) fn capture() -> Self {
        let mut keys = [0u8; 256];
        unsafe { GetKeyboardState(&mut keys) }.unwrap();
        Self::from_keyboard_state(keys)
    }

    fn from_keyboard_state(keys: [u8; 256]) -> Self {
        let flag_keys = [
            VK_LSHIFT,
            VK_RSHIFT,
            VK_LCONTROL,
            VK_RCONTROL,
            VK_LMENU,
            VK_RMENU,
            VK_LWIN,
            VK_RWIN,
        ];

        let value = (0..flag_keys.len())
            .filter(|flag_index| {
                let vk_code = flag_keys[*flag_index].0;
                keys[vk_code as usize] & 0x80 != 0
            })
            .fold(0, |acc, flag_index| acc | (1 << flag_index));

        Self(value as u8)
    }

    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl Display for KeyModifiers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut names: Vec<&str> = vec![];

        if self.contains(KM_LSHIFT | KM_RSHIFT) {
            names.push("SHIFT");
        } else if self.contains(KM_LSHIFT) {
            names.push("LEFT_SHIFT");
        } else if self.contains(KM_RSHIFT) {
            names.push("RIGHT_SHIFT");
        }

        if self.contains(KM_LCTRL | KM_RCTRL) {
            names.push("CTRL");
        } else if self.contains(KM_LCTRL) {
            names.push("LEFT_CTRL");
        } else if self.contains(KM_RCTRL) {
            names.push("RIGHT_CTRL");
        }

        if self.contains(KM_LALT | KM_RALT) {
            names.push("ALT");
        } else if self.contains(KM_LALT) {
            names.push("LEFT_ALT");
        } else if self.contains(KM_RALT) {
            names.push("RIGHT_ALT");
        }

        if self.contains(KM_LWIN | KM_RWIN) {
            names.push("WIN");
        } else if self.contains(KM_LWIN) {
            names.push("LEFT_WIN");
        } else if self.contains(KM_RWIN) {
            names.push("RIGHT_WIN");
        }

        if !names.is_empty() {
            write_joined!(f, names, " + ")
        } else {
            write!(f, "{}", "UNASSIGNED")
        }
    }
}

impl FromStr for KeyModifiers {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ts = s.trim();
        if ts.is_empty() || ts == "UNASSIGNED" {
            Ok(KM_NONE)
        } else {
            let this = ts.split('+').fold(KM_NONE, |acc, part| {
                let tp = part.trim();
                match tp {
                    "LEFT_SHIFT" => acc | KM_LSHIFT,
                    "RIGHT_SHIFT" => acc | KM_RSHIFT,
                    "SHIFT" => acc | KM_LSHIFT | KM_RSHIFT,
                    "LEFT_CTRL" => acc | KM_LCTRL,
                    "RIGHT_CTRL" => acc | KM_RCTRL,
                    "CTRL" => acc | KM_LCTRL | KM_RCTRL,
                    "LEFT_ALT" => acc | KM_LALT,
                    "RIGHT_ALT" => acc | KM_RALT,
                    "ALT" => acc | KM_LALT | KM_RALT,
                    "LEFT_WIN" => acc | KM_LWIN,
                    "RIGHT_WIN" => acc | KM_RWIN,
                    "WIN" => acc | KM_LWIN | KM_RWIN,
                    &_ => panic!("Error parsing key modifier: `{tp}`"),
                }
            });

            Ok(this)
        }
    }
}

impl Serialize for KeyModifiers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Ok(self.to_string().serialize(serializer)?)
    }
}

impl<'de> Deserialize<'de> for KeyModifiers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = &String::deserialize(deserializer)?;
        Ok(Self::from_str(s).unwrap())
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

#[cfg(test)]
mod tests {
    use crate::key_modifiers::{
        KeyModifiers, KM_LALT, KM_LCTRL, KM_LSHIFT, KM_LWIN, KM_NONE, KM_RALT, KM_RCTRL,
        KM_RSHIFT, KM_RWIN,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LCONTROL, VK_LSHIFT, VK_RSHIFT, VK_RWIN};

    #[macro_export]
    macro_rules! key_mod {
        ($text:literal) => {
            $text.parse::<KeyModifiers>().unwrap()
        };
    }

    #[test]
    fn test_key_modifiers_display() {
        assert_eq!("UNASSIGNED", KM_NONE.to_string());

        assert_eq!("LEFT_SHIFT + RIGHT_WIN", (KM_LSHIFT | KM_RWIN).to_string());
        assert_eq!(
            "RIGHT_CTRL + LEFT_ALT",
            (KM_LALT | KM_RCTRL).to_string()
        );

        assert_eq!(
            "SHIFT + CTRL + ALT + WIN",
            (KM_LSHIFT
                | KM_RSHIFT
                | KM_LWIN
                | KM_RWIN
                | KM_LALT
                | KM_RALT
                | KM_LCTRL
                | KM_RCTRL)
                .to_string()
        );
    }

    #[test]
    fn test_key_modifiers_parse() {
        assert_eq!(KM_NONE, "".parse().unwrap());
        assert_eq!(KM_NONE, "UNASSIGNED".parse().unwrap());

        assert_eq!(
            KM_LSHIFT | KM_RSHIFT | KM_RWIN,
            "LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN".parse().unwrap()
        );

        assert_eq!(
            KM_LSHIFT
                | KM_RSHIFT
                | KM_LWIN
                | KM_RWIN
                | KM_LALT
                | KM_RALT
                | KM_LCTRL
                | KM_RCTRL,
            "SHIFT + WIN + ALT + CTRL".parse().unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_key_modifiers_parse_fails() {
        "BANANA".parse::<KeyModifiers>().unwrap();
    }

    #[test]
    fn test_key_modifiers_capture() {
        let mut keys = [0u8; 256];
        assert_eq!(KM_NONE, KeyModifiers::from_keyboard_state(keys));

        keys[VK_LSHIFT.0 as usize] = 0x80;
        keys[VK_RSHIFT.0 as usize] = 0x80;
        keys[VK_LCONTROL.0 as usize] = 0x80;
        keys[VK_RWIN.0 as usize] = 0x80;

        assert_eq!(
            KM_LSHIFT | KM_RSHIFT | KM_LCTRL | KM_RWIN,
            KeyModifiers::from_keyboard_state(keys)
        );
    }

    #[test]
    fn test_key_modifiers_serialize() {
        let source: KeyModifiers = "LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN".parse().unwrap();
        let json = serde_json::to_string_pretty(&source).unwrap();

        dbg!(&json);

        let actual = serde_json::from_str::<KeyModifiers>(&json).unwrap();
        assert_eq!(source, actual);
    }
}
