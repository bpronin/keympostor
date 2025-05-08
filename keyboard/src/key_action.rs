use crate::key::{KeyCode, ScanCode, VirtualKey};
use crate::key_action::KeyTransition::{Down, Up};
use crate::key_event::SELF_EVENT_MARKER;
use crate::write_joined;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VIRTUAL_KEY,
};
use KeyCode::{SC, VK};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum KeyTransition {
    #[serde(alias = "UP", alias = "up")]
    Up,
    #[serde(alias = "DOWN", alias = "down")]
    Down,
}

impl KeyTransition {
    pub(crate) fn from_bool(up: bool) -> KeyTransition {
        if up { Up } else { Down }
    }

    pub(crate) fn is_up(&self) -> bool {
        matches!(*self, Up)
    }

    // pub fn is_down(&self) -> bool {
    //     matches!(*self, Down)
    // }
}

impl Default for KeyTransition {
    fn default() -> Self {
        Up
    }
}

impl Display for KeyTransition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Up => Display::fmt(&'↑', f),
            Down => Display::fmt(&'↓', f),
        }
    }
}

impl FromStr for KeyTransition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.trim().chars();
        let symbol = chars.next().expect("Key transition symbol is empty.");
        if chars.next().is_none() {
            match symbol {
                '↑' | '^' => Ok(Up),
                '↓' | '*' => Ok(Down),
                _ => Err(format!("Illegal key transition symbol `{}`.", s)),
            }
        } else {
            Err(format!("Key transition symbols `{}` is too long.", s))
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyAction {
    pub key: KeyCode,
    pub transition: KeyTransition,
}

impl KeyAction {
    fn create_input(&self) -> INPUT {
        match self.key {
            VK(vk) => self.create_virtual_key_input(*vk),
            SC(sc) => self.create_scancode_input(*sc),
        }
    }

    fn create_virtual_key_input(&self, virtual_key: VirtualKey) -> INPUT {
        let mut flags = KEYBD_EVENT_FLAGS::default();
        if self.transition.is_up() {
            flags |= KEYEVENTF_KEYUP
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(virtual_key.value as u16),
                    dwFlags: flags,
                    dwExtraInfo: SELF_EVENT_MARKER.as_ptr() as usize,
                    ..Default::default()
                },
            },
        }
    }

    fn create_scancode_input(&self, scan_code: ScanCode) -> INPUT {
        let mut flags = KEYEVENTF_SCANCODE;
        if scan_code.is_extended {
            flags |= KEYEVENTF_EXTENDEDKEY
        }
        if self.transition.is_up() {
            flags |= KEYEVENTF_KEYUP;
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wScan: scan_code.ext_value(),
                    dwFlags: flags,
                    dwExtraInfo: SELF_EVENT_MARKER.as_ptr() as usize,
                    ..Default::default()
                },
            },
        }
    }
}

impl Display for KeyAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.key, self.transition)
    }
}

impl FromStr for KeyAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let st = s.trim();
        let suf = st
            .chars()
            .last()
            .expect(&format!("Error parsing key action. String is empty. `{s}`"));
        let key = st
            .strip_suffix(suf)
            .expect(&format!("Invalid key action suffix: `{suf}`."));
        Ok(Self {
            key: key.parse()?,
            transition: suf.to_string().parse()?,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyActionSequence {
    pub(crate) actions: Vec<KeyAction>,
}

impl KeyActionSequence {
    pub(crate) fn create_input(&self) -> Vec<INPUT> {
        self.actions.iter().map(|a| a.create_input()).collect()
    }
}

impl Display for KeyActionSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_joined!(f, &self.actions, " → ")
    }
}

impl FromStr for KeyActionSequence {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let actions = s
            .split(|c| "→>".contains(c))
            .map(str::parse)
            .collect::<Result<_, _>>()?;

        Ok(Self { actions })
    }
}

// #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
// pub struct KeyActionPattern {
//     sequence: Vec<KeyActionSequence>,
// }

#[cfg(test)]
mod tests {
    use crate::assert_not;
    use crate::key::KeyCode::SC;
    use crate::key::{KeyCode, ScanCode, VirtualKey};
    use crate::key_action::KeyTransition::{Down, Up};
    use crate::key_action::{KeyAction, KeyActionSequence, KeyTransition};
    use windows::Win32::UI::Input::KeyboardAndMouse::{KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP};
    use KeyCode::VK;

    #[macro_export]
    macro_rules! key_act {
        ($text:literal) => {
            $text.parse::<KeyAction>().unwrap()
        };
    }

    #[macro_export]
    macro_rules! key_act_seq {
        ($text:literal) => {
            $text.parse::<KeyActionSequence>().unwrap()
        };
    }

    #[test]
    fn test_key_transition_display() {
        assert_eq!("↓", format!("{}", Down));
        assert_eq!("↑", format!("{}", Up));
    }

    #[test]
    fn test_key_transition_basics() {
        assert_eq!(Up, KeyTransition::default());
        assert_eq!(Up, KeyTransition::from_bool(true));
        assert!(Up.is_up());
        assert_not!(Down.is_up());
    }

    #[test]
    fn test_key_transition_parse() {
        assert_eq!(Down, "↓".parse().unwrap());
        assert_eq!(Up, "↑".parse().unwrap());
        assert_eq!(Down, "*".parse().unwrap());
        assert_eq!(Up, "^".parse().unwrap());
    }

    #[test]
    #[should_panic]
    fn test_key_transition_parse_fails_illegal() {
        assert_eq!(Down, "BANANA".parse().unwrap());
    }
    
    #[test]
    #[should_panic]
    fn test_key_transition_parse_fails_empty() {
        assert_eq!(Down, "".parse().unwrap());
    }

    #[test]
    #[should_panic]
    fn test_key_transition_parse_fails_to_long() {
        assert_eq!(Down, "↑↑↑".parse().unwrap());
    }

    #[test]
    fn test_key_transition_serialize() {
        let source = Down;
        
        let json = serde_json::to_string_pretty(&source).unwrap();
        let actual = serde_json::from_str::<KeyTransition>(&json).unwrap();
        
        assert_eq!(source, actual);

        let source = Up;
        let json = serde_json::to_string_pretty(&source).unwrap();

        // dbg!(&json);

        let actual = serde_json::from_str::<KeyTransition>(&json).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_action_display() {
        let actual = KeyAction {
            key: VK(VirtualKey::from_name("VK_RETURN").unwrap()),
            transition: Down,
        };
        assert_eq!("VK_RETURN↓", format!("{}", actual));

        let actual = KeyAction {
            key: SC(ScanCode::from_name("SC_ENTER").unwrap()),
            transition: Up,
        };
        assert_eq!("SC_ENTER↑", format!("{}", actual));
    }

    #[test]
    fn test_key_action_serialize() {
        let source = KeyAction {
            key: VK(VirtualKey::from_name("VK_RETURN").unwrap()),
            transition: Down,
        };
        let json = serde_json::to_string_pretty(&source).unwrap();

        // dbg!(&json);

        let actual = serde_json::from_str::<KeyAction>(&json).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_action_sequence_display() {
        let actual = key_act_seq!("VK_RETURN↓ → VK_SHIFT↑");

        assert_eq!("VK_RETURN↓ → VK_SHIFT↑", format!("{}", actual));
    }

    #[test]
    fn test_key_action_sequence_serialize() {
        let source = key_act_seq!("VK_RETURN↓ → VK_SHIFT↓");

        let json = serde_json::to_string_pretty(&source).unwrap();

        let actual = serde_json::from_str::<KeyActionSequence>(&json).unwrap();
        assert_eq!(source, actual);
    }

    #[test]
    fn test_key_action_sequence_create_input() {
        let source = key_act_seq!("VK_RETURN↓ → SC_NUM_ENTER↑");
        
        let input = source.create_input();

        assert_eq!(2, input.len());

        let VK(vk) = source.actions[0].key else {
            panic!("Not a VK")
        };
        assert_eq!(vk.value, unsafe { input[0].Anonymous.ki.wVk.0 } as u8);
        assert_not!(unsafe {
            input[0]
                .Anonymous
                .ki
                .dwFlags
                .contains(KEYEVENTF_EXTENDEDKEY)
        });
        assert!(!unsafe { input[0].Anonymous.ki.dwFlags.contains(KEYEVENTF_KEYUP) });

        let SC(sc) = source.actions[1].key else {
            panic!("Not a SC")
        };
        assert_eq!(sc.value, unsafe { input[1].Anonymous.ki.wScan } as u8);
        assert!(unsafe {
            input[1]
                .Anonymous
                .ki
                .dwFlags
                .contains(KEYEVENTF_EXTENDEDKEY)
        });
        assert!(unsafe { input[1].Anonymous.ki.dwFlags.contains(KEYEVENTF_KEYUP) });
    }

    // #[test]
    // fn test_key_action_pattern_serialize() {
    //     let source = KeyActionPattern {
    //         sequence: vec![
    //             KeyActionSequence {
    //                 actions: vec![
    //                     KeyAction {
    //                         key: VK(VirtualKey::from_name("VK_RETURN").unwrap()),
    //                         transition: Down,
    //                     },
    //                     KeyAction {
    //                         key: VK(VirtualKey::from_name("VK_SHIFT").unwrap()),
    //                         transition: Down,
    //                     },
    //                 ],
    //             },
    //             KeyActionSequence {
    //                 actions: vec![KeyAction {
    //                     key: SC(ScanCode::from_name("SC_ENTER").unwrap()),
    //                     transition: Down,
    //                 }],
    //             },
    //         ],
    //     };
    //
    //     let json = serde_json::to_string_pretty(&source).unwrap();
    //
    //     // dbg!(&json);
    //
    //     let actual = serde_json::from_str::<KeyActionPattern>(&json).unwrap();
    //     assert_eq!(source, actual);
    // }
}
