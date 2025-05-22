use crate::write_joined;
use core::ops;
use ops::{BitOr};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN,
};

pub(crate) const KM_NONE: KeyModifiers = KeyModifiers(0);
pub(crate) const KM_LSHIFT: KeyModifiers = KeyModifiers(1);
pub(crate) const KM_RSHIFT: KeyModifiers = KeyModifiers(1 << 1);
pub(crate) const KM_LCTRL: KeyModifiers = KeyModifiers(1 << 2);
pub(crate) const KM_RCTRL: KeyModifiers = KeyModifiers(1 << 3);
pub(crate) const KM_LALT: KeyModifiers = KeyModifiers(1 << 4);
pub(crate) const KM_RALT: KeyModifiers = KeyModifiers(1 << 5);
pub(crate) const KM_LWIN: KeyModifiers = KeyModifiers(1 << 6);
pub(crate) const KM_RWIN: KeyModifiers = KeyModifiers(1 << 7);
pub(crate) const KM_ALL: KeyModifiers = KeyModifiers(u8::MAX);

// #[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash)]
// struct KeyModifiersMatrix(u64);
//
// impl KeyModifiersMatrix {
//     pub(crate) fn new(items: &[KeyModifiers]) -> Self {
//         let mut bytes = [0u8; 8];
//         for (i, item) in items.iter().enumerate() {
//             bytes[i] = item.0;
//         }
//         Self(u64::from_be_bytes(bytes))
//     }
//
//     pub(crate) fn has_item(&self, item: KeyModifiers) -> bool {
//         // self.0.to_be_bytes().iter().any(|b| b & item.0 == item.0)
//         for i in 0..8 {
//             let b = ((self.0 >> (8 * i)) & 0xFF) as u8;
//             if b & item.0 == item.0 {
//                 return true;
//             }
//         }
//         false
//     }
// }
// impl Display for KeyModifiersMatrix {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let items = self
//             .0
//             .to_be_bytes()
//             .iter()
//             .filter_map(|b| {
//                 if *b != 0 {
//                     Some(KeyModifiers(*b))
//                 } else {
//                     None
//                 }
//             })
//             .collect::<Vec<_>>();
//
//         if !items.is_empty() {
//             write_joined!(f, items, ", ")
//         } else {
//             Ok(())
//         }
//     }
// }
//
// impl FromStr for KeyModifiersMatrix {
//     type Err = String;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let ts = s.trim();
//         let value = if ts.is_empty() {
//             KeyModifiersMatrix::default()
//         } else {
//             let items = ts
//                 .split(',')
//                 .map(|s| s.parse().unwrap())
//                 .collect::<Vec<KeyModifiers>>();
//             KeyModifiersMatrix::new(&items)
//         };
//         Ok(value)
//     }
// }

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub(crate) struct KeyModifiersMatrix {
    pub(crate) items: [KeyModifiers; 8],
}

impl KeyModifiersMatrix {
    pub(crate) fn new(modifiers: &[KeyModifiers]) -> Self {
        let mut items = [KeyModifiers::default(); 8];
        for (i, m) in modifiers.iter().enumerate() {
            items[i] = *m;
        }
        Self { items }
    }

    pub(crate) fn has_item(&self, item: KeyModifiers) -> bool {
        for i in self.items {
            if i.contains(item) {
                return true;
            }
        }
        false
    }
}

impl Display for KeyModifiersMatrix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if !self.items.is_empty() {
            write_joined!(f, self.items.iter().filter(|&x| { *x != KM_NONE }), ", ")
        } else {
            Ok(())
        }
    }
}

impl Serialize for KeyModifiersMatrix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Ok(self.to_string().serialize(serializer)?)
    }
}

impl<'de> Deserialize<'de> for KeyModifiersMatrix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        let modifiers = text.parse().map_err(de::Error::custom)?;

        Ok(modifiers)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash)]
pub(crate) struct KeyModifiers(u8);

impl KeyModifiers {
    pub(crate) fn from_keyboard_state(keys: [u8; 256]) -> Self {
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

    pub(crate) const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub(crate) fn to_string_short(&self) -> String {
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

impl Display for KeyModifiers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut names: Vec<&str> = vec![];

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
        let text = String::deserialize(deserializer)?;
        let modifiers = text.parse().map_err(de::Error::custom)?;

        Ok(modifiers)
    }
}

impl BitOr for KeyModifiers {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

// impl BitAnd for KeyModifiers {
//     type Output = Self;
//     fn bitand(self, other: Self) -> Self {
//         Self(self.0 & other.0)
//     }
// }

// impl BitOrAssign for KeyModifiers {
//     fn bitor_assign(&mut self, other: Self) {
//         self.0.bitor_assign(other.0)
//     }
// }

// impl BitAndAssign for KeyModifiers {
//     fn bitand_assign(&mut self, other: Self) {
//         self.0.bitand_assign(other.0)
//     }
// }

// impl Not for KeyModifiers {
//     type Output = Self;
//     fn not(self) -> Self {
//         Self(self.0.not())
//     }
// }

#[cfg(test)]
mod tests {
    use crate::assert_not;
    use crate::keyboard::key_modifiers::{
        KeyModifiers, KeyModifiersMatrix, KM_LALT, KM_LCTRL, KM_LSHIFT, KM_LWIN, KM_NONE, KM_RALT, KM_RCTRL,
        KM_RSHIFT, KM_RWIN,
    };
    use serde::{Deserialize, Serialize};
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LCONTROL, VK_LSHIFT, VK_RSHIFT, VK_RWIN};

    #[macro_export]
    macro_rules! key_mod {
        ($text:literal) => {
            $text.parse::<KeyModifiers>().unwrap()
        };
    }

    #[macro_export]
    macro_rules! key_mod_mx {
        ($text:literal) => {
            $text.parse::<KeyModifiersMatrix>().unwrap()
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
        /* TOML requires wrapper */
        #[derive(Debug, Serialize, Deserialize)]
        struct Wrapper {
            value: KeyModifiers,
        }

        let source = Wrapper {
            value: "LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN".parse().unwrap(),
        };
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str::<Wrapper>(&text).unwrap();
        assert_eq!(source.value, actual.value);
    }

    #[test]
    fn test_key_modifiers_matrix_display() {
        let actual = KeyModifiersMatrix::new(&[
            KM_LALT,
            KM_RSHIFT,
            KM_RCTRL,
            KM_NONE,
            KM_RCTRL | KM_RWIN,
            KM_NONE,
        ])
        .to_string();

        assert_eq!(
            "LEFT_ALT, RIGHT_SHIFT, RIGHT_CTRL, RIGHT_CTRL + RIGHT_WIN",
            actual
        );
    }

    #[test]
    fn test_key_modifiers_matrix_contains() {
        let matrix = KeyModifiersMatrix::new(&[KM_LALT, KM_RSHIFT, KM_RCTRL, KM_RCTRL | KM_RWIN]);

        assert!(matrix.has_item(KM_LALT));
        assert_not!(matrix.has_item(KM_RALT));
        assert!(matrix.has_item(KM_RSHIFT));
        assert_not!(matrix.has_item(KM_LSHIFT));
        assert!(matrix.has_item(KM_RCTRL | KM_RWIN));
        assert_not!(matrix.has_item(KM_RALT | KM_LWIN));

        let matrix = KeyModifiersMatrix::new(&[KM_LALT, KM_RSHIFT, KM_RCTRL, KM_RCTRL | KM_RWIN]);
        assert_not!(matrix.has_item(KM_RALT | KM_RSHIFT));
    }
}
