use crate::write_joined;
use core::ops;
use ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardState, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_RCONTROL,
    VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT,
};

pub const KM_NONE: KeyModifiers = KeyModifiers(0u16);
pub const KM_LSHIFT: KeyModifiers = KeyModifiers(1u16);
pub const KM_RSHIFT: KeyModifiers = KeyModifiers(1u16 << 1);
pub const KM_SHIFT: KeyModifiers = KeyModifiers(1u16 << 2);
pub const KM_LCTRL: KeyModifiers = KeyModifiers(1u16 << 3);
pub const KM_RCTRL: KeyModifiers = KeyModifiers(1u16 << 4);
pub const KM_CTRL: KeyModifiers = KeyModifiers(1u16 << 5);
pub const KM_LALT: KeyModifiers = KeyModifiers(1u16 << 6);
pub const KM_RALT: KeyModifiers = KeyModifiers(1u16 << 7);
pub const KM_ALT: KeyModifiers = KeyModifiers(1u16 << 8);
pub const KM_LWIN: KeyModifiers = KeyModifiers(1u16 << 9);
pub const KM_RWIN: KeyModifiers = KeyModifiers(1u16 << 10);

const FLAGS_COUNT: usize = 11;
const FLAG_NAMES: [&str; FLAGS_COUNT] = [
    "LSHIFT", "RSHIFT", "SHIFT", "LCONTROL", "RCONTROL", "CONTROL", "LALT", "RALT", "ALT", "LWIN",
    "RWIN",
];

#[derive(Debug, Eq, PartialEq, Default, Clone)]
pub struct KeyModifiers(u16);

impl KeyModifiers {
    pub(crate) fn capture() -> Self {
        let mut keys = [0u8; 256];
        unsafe { GetKeyboardState(&mut keys) }.unwrap();
        Self::from_keys(keys)
    }

    fn from_keys(keys: [u8; 256]) -> Self {
        let flag_keys = [
            VK_LSHIFT,
            VK_RSHIFT,
            VK_SHIFT,
            VK_LCONTROL,
            VK_RCONTROL,
            VK_CONTROL,
            VK_LMENU,
            VK_RMENU,
            VK_MENU,
            VK_LWIN,
            VK_RWIN,
        ];

        // for i in 0..FLAGS_COUNT {
        //     let key = flag_keys[i];
        //     let up = keys[key.0 as usize];
        //     let name = FLAG_NAMES[i];
        //     println!("{:?} {:0X} {name}", key, up)
        // }

        let value = (0..FLAGS_COUNT)
            .filter(|f_ix| {
                let vk_code = flag_keys[*f_ix].0;
                keys[vk_code as usize] & 0x80 != 0
            })
            .fold(0u16, |acc, f_ix| acc | (1u16 << f_ix));

        Self(value)
    }
}

impl Display for KeyModifiers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&str> = (0..FLAGS_COUNT)
            .filter(|f_ix| (self.0 >> f_ix) & 1 == 1)
            .map(|f_ix| FLAG_NAMES[f_ix])
            .collect();
        if !names.is_empty() {
            write_joined!(f, names, " + ")
        } else {
            write!(f, "NONE")
        }
    }
}

impl FromStr for KeyModifiers {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ts = s.trim();
        if ts.is_empty() || ts == "NONE" {
            Ok(KM_NONE)
        } else {
            let value = ts.split('+').fold(0u16, |acc, s| {
                let f_ix = FLAG_NAMES
                    .iter()
                    .position(|&name| name == s.trim())
                    .expect(&format!("Error parsing modifiers `{s}`"));
                acc | (1u16 << f_ix)
            });

            Ok(Self(value))
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
        KeyModifiers, KM_CTRL, KM_LSHIFT, KM_NONE, KM_RSHIFT, KM_RWIN, KM_SHIFT,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_CONTROL, VK_LSHIFT, VK_RSHIFT, VK_RWIN};

    #[test]
    fn test_key_modifiers_display() {
        assert_eq!("NONE", KM_NONE.to_string());

        assert_eq!(
            "LSHIFT + RSHIFT + SHIFT + RWIN",
            (KM_LSHIFT | KM_RSHIFT | KM_SHIFT | KM_RWIN).to_string()
        );
    }

    #[test]
    fn test_key_modifiers_parse() {
        assert_eq!(KM_NONE, "".parse().unwrap());
        assert_eq!(KM_NONE, "NONE".parse().unwrap());
        assert_eq!(
            KM_LSHIFT | KM_RSHIFT | KM_SHIFT | KM_RWIN,
            "LSHIFT + RSHIFT + SHIFT + RWIN".parse().unwrap()
        );
    }

    #[test]
    fn test_key_modifiers_capture() {
        let mut keys = [0u8; 256];
        assert_eq!(KM_NONE, KeyModifiers::from_keys(keys));

        keys[VK_LSHIFT.0 as usize] = 0x80;
        keys[VK_RSHIFT.0 as usize] = 0x80;
        keys[VK_CONTROL.0 as usize] = 0x80;
        keys[VK_RWIN.0 as usize] = 0x80;

        assert_eq!(
            KM_LSHIFT | KM_RSHIFT | KM_CTRL | KM_RWIN,
            KeyModifiers::from_keys(keys)
        );
    }

    #[test]
    fn test_key_modifiers_serialize() {
        let source: KeyModifiers = "LSHIFT + RSHIFT + SHIFT + RWIN".parse().unwrap();
        let json = serde_json::to_string_pretty(&source).unwrap();

        dbg!(&json);

        let actual = serde_json::from_str::<KeyModifiers>(&json).unwrap();
        assert_eq!(source, actual);
    }
}
