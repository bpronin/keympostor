use crate::write_joined;
use core::ops;
use ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardState, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_RMENU, VK_RSHIFT,
    VK_RWIN,
};

pub const KM_NONE: KeyModifiers = KeyModifiers(0);
pub const KM_LSHIFT: KeyModifiers = KeyModifiers(1);
pub const KM_RSHIFT: KeyModifiers = KeyModifiers(1 << 1);
pub const KM_LCONTROL: KeyModifiers = KeyModifiers(1 << 2);
pub const KM_RCONTROL: KeyModifiers = KeyModifiers(1 << 3);
pub const KM_LALT: KeyModifiers = KeyModifiers(1 << 4);
pub const KM_RALT: KeyModifiers = KeyModifiers(1 << 5);
pub const KM_LWIN: KeyModifiers = KeyModifiers(1 << 6);
pub const KM_RWIN: KeyModifiers = KeyModifiers(1 << 7);

const FLAGS_COUNT: usize = 8;
// const FLAG_NAMES: [&str; FLAGS_COUNT] = [
//     "LSHIFT", "RSHIFT", "LCONTROL", "RCONTROL", "LALT", "RALT", "LWIN", "RWIN",
// ];

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct KeyModifiers(u8);

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
            VK_LCONTROL,
            VK_RCONTROL,
            VK_LMENU,
            VK_RMENU,
            VK_LWIN,
            VK_RWIN,
        ];

        let value = (0..FLAGS_COUNT)
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
        // let names: Vec<&str> = (0..FLAGS_COUNT)
        //     .filter(|f_ix| (self.0 >> f_ix) & 1 == 1)
        //     .map(|f_ix| {
        //         match f_ix {
        //             0 => "LSHIFT",
        //             1 => "RSHIFT",
        //             2 => "LCONTROL",
        //             3 => "RCONTROL",
        //             4 => "LALT",
        //             5 => "RALT",
        //             6 => "LWIN",
        //             7 => "RWIN",
        //             _ => panic!("Illegal key modifier index: {}", f_ix),
        //         }
        //     })
        //     .collect();
        //

        let mut names: Vec<&str> = vec![];

        if self.contains(KM_LSHIFT | KM_RSHIFT) {
            names.push("SHIFT");
        } else if self.contains(KM_LSHIFT) {
            names.push("LSHIFT");
        } else if self.contains(KM_RSHIFT) {
            names.push("RSHIFT");
        }

        if self.contains(KM_LCONTROL | KM_RCONTROL) {
            names.push("CONTROL");
        } else if self.contains(KM_LCONTROL) {
            names.push("LCONTROL");
        } else if self.contains(KM_RCONTROL) {
            names.push("RCONTROL");
        }

        if self.contains(KM_LALT | KM_RALT) {
            names.push("ALT");
        } else if self.contains(KM_LALT) {
            names.push("LALT");
        } else if self.contains(KM_RALT) {
            names.push("RALT");
        }

        if self.contains(KM_LWIN | KM_RWIN) {
            names.push("WIN");
        } else if self.contains(KM_LWIN) {
            names.push("LWIN");
        } else if self.contains(KM_RWIN) {
            names.push("RWIN");
        }

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
            let this = ts.split('+').fold(KM_NONE, |acc, part| {
                let tp = part.trim();
                match tp {
                    "LSHIFT" => acc | KM_LSHIFT,
                    "RSHIFT" => acc | KM_RSHIFT,
                    "SHIFT" => acc | KM_LSHIFT | KM_RSHIFT,
                    "LCONTROL" => acc | KM_LCONTROL,
                    "RCONTROL" => acc | KM_RCONTROL,
                    "CONTROL" => acc | KM_LCONTROL | KM_RCONTROL,
                    "LALT" => acc | KM_LALT,
                    "RALT" => acc | KM_RALT,
                    "ALT" => acc | KM_LALT | KM_RALT,
                    "LWIN" => acc | KM_LWIN,
                    "RWIN" => acc | KM_RWIN,
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
        KeyModifiers, KM_LALT, KM_LCONTROL, KM_LSHIFT, KM_LWIN, KM_NONE, KM_RALT, KM_RCONTROL,
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
        assert_eq!("NONE", KM_NONE.to_string());

        assert_eq!("LSHIFT + RWIN", (KM_LSHIFT | KM_RWIN).to_string());
        assert_eq!("RCONTROL + LALT", (KM_LALT | KM_RCONTROL).to_string());
        
        assert_eq!("SHIFT + CONTROL + ALT + WIN", (KM_LSHIFT
            | KM_RSHIFT
            | KM_LWIN
            | KM_RWIN
            | KM_LALT
            | KM_RALT
            | KM_LCONTROL
            | KM_RCONTROL).to_string());
        
    }

    #[test]
    fn test_key_modifiers_parse() {
        assert_eq!(KM_NONE, "".parse().unwrap());
        assert_eq!(KM_NONE, "NONE".parse().unwrap());

        assert_eq!(
            KM_LSHIFT | KM_RSHIFT | KM_RWIN,
            "LSHIFT + RSHIFT + RWIN".parse().unwrap()
        );

        assert_eq!(
            KM_LSHIFT
                | KM_RSHIFT
                | KM_LWIN
                | KM_RWIN
                | KM_LALT
                | KM_RALT
                | KM_LCONTROL
                | KM_RCONTROL,
            "SHIFT + WIN + ALT + CONTROL".parse().unwrap()
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
        assert_eq!(KM_NONE, KeyModifiers::from_keys(keys));

        keys[VK_LSHIFT.0 as usize] = 0x80;
        keys[VK_RSHIFT.0 as usize] = 0x80;
        keys[VK_LCONTROL.0 as usize] = 0x80;
        keys[VK_RWIN.0 as usize] = 0x80;

        assert_eq!(
            KM_LSHIFT | KM_RSHIFT | KM_LCONTROL | KM_RWIN,
            KeyModifiers::from_keys(keys)
        );
    }

    #[test]
    fn test_key_modifiers_serialize() {
        let source: KeyModifiers = "LSHIFT + RSHIFT + RWIN".parse().unwrap();
        let json = serde_json::to_string_pretty(&source).unwrap();

        dbg!(&json);

        let actual = serde_json::from_str::<KeyModifiers>(&json).unwrap();
        assert_eq!(source, actual);
    }
}
