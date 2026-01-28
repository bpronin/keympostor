use crate::error::KeyError;
use crate::key::{
    KEY_LEFT_ALT, KEY_LEFT_CTRL, KEY_LEFT_SHIFT, KEY_LEFT_WIN, KEY_RIGHT_ALT, KEY_RIGHT_CTRL,
    KEY_RIGHT_SHIFT, KEY_RIGHT_WIN, KEY_SHIFT,
};
use crate::modifiers::KeyModifiers::All;
use crate::state::KeyboardState;
use crate::{deserialize_from_string, key_err, serialize_to_string, write_joined};
use core::ops;
use ops::BitOr;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_RMENU, VK_RSHIFT,
    VK_RWIN,
};

pub const KM_NONE: ModifierKeys = ModifierKeys(0);
pub const KM_LSHIFT: ModifierKeys = ModifierKeys(1);
pub const KM_RSHIFT: ModifierKeys = ModifierKeys(1 << 1);
pub const KM_LCTRL: ModifierKeys = ModifierKeys(1 << 2);
pub const KM_RCTRL: ModifierKeys = ModifierKeys(1 << 3);
pub const KM_LALT: ModifierKeys = ModifierKeys(1 << 4);
pub const KM_RALT: ModifierKeys = ModifierKeys(1 << 5);
pub const KM_LWIN: ModifierKeys = ModifierKeys(1 << 6);
pub const KM_RWIN: ModifierKeys = ModifierKeys(1 << 7);

//todo: replace it with KeyboardState to support any key as modifier

//todo: probably change `[A+B] C^` to 'A+B+C^' and get rid of `[]` prefix for modifiers absence ?
// ok. then what's would be a 'A+B+C' and 'A+B+C*' ?

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct ModifierKeys(u8);

impl ModifierKeys {
    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for ModifierKeys {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl Display for ModifierKeys {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut names: Vec<&str> = Vec::new();

        if self.contains(KM_LSHIFT) {
            names.push(KEY_LEFT_SHIFT.name);
        }
        if self.contains(KM_RSHIFT) {
            names.push(KEY_RIGHT_SHIFT.name);
        }
        if self.contains(KM_LCTRL) {
            names.push(KEY_LEFT_CTRL.name);
        }
        if self.contains(KM_RCTRL) {
            names.push(KEY_RIGHT_CTRL.name);
        }
        if self.contains(KM_LALT) {
            names.push(KEY_LEFT_ALT.name);
        }
        if self.contains(KM_RALT) {
            names.push(KEY_RIGHT_ALT.name);
        }
        if self.contains(KM_LWIN) {
            names.push(KEY_LEFT_WIN.name);
        }
        if self.contains(KM_RWIN) {
            names.push(KEY_RIGHT_WIN.name);
        }

        if !names.is_empty() {
            write_joined!(f, names, " + ")
        } else {
            Ok(())
        }
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

impl From<&KeyboardState> for ModifierKeys {
    fn from(keyboard_state: &KeyboardState) -> Self {
        let value = (0..MODIFIER_KEYS.len())
            .filter(|modifier_index| {
                let vk_code = MODIFIER_KEYS[*modifier_index].0;
                keyboard_state.is_set(vk_code as u8)
            })
            .fold(0, |acc, flag_index| acc | (1 << flag_index));

        Self(value as u8)
    }
}

impl FromStr for ModifierKeys {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Ok(KM_NONE)
        } else {
            let result = s.trim().split('+').fold(KM_NONE, |acc, part| {
                let p = part.trim();
                acc | if KEY_LEFT_SHIFT.name == p {
                    KM_LSHIFT
                } else if KEY_RIGHT_SHIFT.name == p {
                    KM_RSHIFT
                } else if KEY_SHIFT.name == p {
                    KM_LSHIFT | KM_RSHIFT
                } else if KEY_LEFT_CTRL.name == p {
                    KM_LCTRL
                } else if KEY_RIGHT_CTRL.name == p {
                    KM_RCTRL
                } else if KEY_LEFT_ALT.name == p {
                    KM_LALT
                } else if KEY_RIGHT_ALT.name == p {
                    KM_RALT
                } else if KEY_LEFT_WIN.name == p {
                    KM_LWIN
                } else if KEY_RIGHT_WIN.name == p {
                    KM_RWIN
                    // todo: this expands key into LEFT+RIGHT but must be LEFT|RIGHT
                    // } else if KEY_CTRL.name == p {
                    //     KM_LCTRL | KM_RCTRL
                    // } else if "ALT" == p {
                    //     KM_LALT | KM_RALT
                    // } else if "WIN" == p {
                    //     KM_LWIN | KM_RWIN
                } else {
                    KM_NONE
                }
            });

            if result != KM_NONE {
                Ok(result)
            } else {
                key_err!("Error parsing key modifiers: `{s}`")
            }
        }
    }
}

impl Serialize for ModifierKeys {
    serialize_to_string!();
}

impl<'de> Deserialize<'de> for ModifierKeys {
    deserialize_from_string!();
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum KeyModifiers {
    Any,
    All(ModifierKeys),
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

impl FromStr for KeyModifiers {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        /* `Any` is parsed outside from `None` */
        Ok(All(ModifierKeys::from_str(s.trim())?))
    }
}

#[cfg(test)]
mod tests {
    use crate::modifiers::KeyModifiers::{All, Any};
    use crate::modifiers::{
        KeyModifiers, ModifierKeys, KM_LALT, KM_LCTRL, KM_LSHIFT, KM_NONE, KM_RCTRL, KM_RSHIFT,
        KM_RWIN,
    };
    use crate::state::KeyboardState;
    use crate::utils::test::SerdeWrapper;
    use std::str::FromStr;
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LCONTROL, VK_LSHIFT, VK_RSHIFT, VK_RWIN};

    #[macro_export]
    macro_rules! key_mod {
        ($text:literal) => {
            $text.parse::<ModifierKeys>().unwrap()
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
    fn test_key_modifiers_capture() {
        let keys = KeyboardState::new();
        assert_eq!(KM_NONE, ModifierKeys::from(&keys));

        let keys = KeyboardState::from_bits(&[
            VK_LSHIFT.0 as u8,
            VK_RSHIFT.0 as u8,
            VK_LCONTROL.0 as u8,
            VK_RWIN.0 as u8,
        ]);

        assert_eq!(
            KM_LSHIFT | KM_RSHIFT | KM_LCTRL | KM_RWIN,
            ModifierKeys::from(&keys)
        );
    }

    #[test]
    fn test_keyboard_state_display() {
        assert_eq!(
            "[LEFT_SHIFT + RIGHT_WIN]",
            All(KM_LSHIFT | KM_RWIN).to_string()
        );
        assert_eq!("[]", All(KM_NONE).to_string());
        assert_eq!("", Any.to_string());
    }

    #[test]
    fn test_key_modifiers_from_str() {
        assert_eq!(All(KM_NONE), KeyModifiers::from_str("").unwrap());

        assert_eq!(
            All(KM_LSHIFT | KM_RSHIFT | KM_RWIN),
            KeyModifiers::from_str("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN").unwrap()
        );
    }

    #[test]
    fn test_key_modifiers_from_str_fails() {
        assert!(KeyModifiers::from_str("BANANA").is_err());
    }

    #[test]
    fn test_key_modifier_keys_serialize() {
        let source = SerdeWrapper::new(key_mod!("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN"));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_modifiers_serialize() {
        let source = SerdeWrapper::new(All(key_mod!("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN")));
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);

        let source = SerdeWrapper::new(Any);
        let text = toml::to_string_pretty(&source).unwrap();
        let actual = toml::from_str(&text).unwrap();

        assert_eq!(source, actual);
    }
}
