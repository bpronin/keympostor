use crate::key_hook::SELF_MARKER;
use crate::key_code::{Key, ScanCode, VirtualKey};
use crate::key_modifier::KeyModifiers;
use crate::util::slices_equal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VIRTUAL_KEY,
};
use crate::key_transition::KeyTransition;

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq)]
pub struct KeyAction {
    pub key: Key,
    pub transition: KeyTransition,
    pub modifiers: Option<KeyModifiers>,
}

impl KeyAction {
    fn create_input(&self) -> INPUT {
        if let Some(scancode) = self.key.scancode {
            Self::create_scancode_input(scancode, self.transition)
        } else {
            let virtual_key = self.key.virtual_key.unwrap();
            Self::create_virtual_key_input(virtual_key, self.transition)
        }
    }

    fn create_virtual_key_input(virtual_key: &VirtualKey, transition: KeyTransition) -> INPUT {
        let mut flags = KEYBD_EVENT_FLAGS::default();
        if transition.is_up() {
            flags |= KEYEVENTF_KEYUP
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(virtual_key.value as u16),
                    dwFlags: flags,
                    dwExtraInfo: SELF_MARKER.as_ptr() as usize,
                    ..Default::default()
                },
            },
        }
    }

    fn create_scancode_input(scancode: &ScanCode, transition: KeyTransition) -> INPUT {
        let mut flags = KEYEVENTF_SCANCODE;
        if scancode.is_extended {
            flags |= KEYEVENTF_EXTENDEDKEY
        }
        if transition.is_up() {
            flags |= KEYEVENTF_KEYUP;
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wScan: scancode.ext_value(),
                    dwFlags: flags,
                    dwExtraInfo: SELF_MARKER.as_ptr() as usize,
                    ..Default::default()
                },
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeyActionSequence {
    pub(crate) actions: Vec<KeyAction>,
}

impl KeyActionSequence {
    pub fn from(actions: Vec<KeyAction>) -> Self {
        Self { actions }
    }

    pub(crate) fn send(&self) {
        let inputs: Vec<INPUT> = self.actions.iter().map(|a| a.create_input()).collect();
        unsafe { SendInput(inputs.as_slice(), size_of::<INPUT>() as i32) };
    }
}

impl PartialEq for KeyActionSequence {
    fn eq(&self, other: &KeyActionSequence) -> bool {
        slices_equal(&self.actions, &other.actions)
    }
}

impl Serialize for KeyActionSequence {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Ok(self.actions.serialize(serializer)?)
    }
}

impl<'de> Deserialize<'de> for KeyActionSequence {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from(Vec::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use crate::key_action::KeyAction;
    use crate::key_code::{Key, VirtualKey};
    use crate::key_transition::KeyTransition;

    #[test]
    fn key_action_to_text() {
        let _action = KeyAction {
            key: Key::from_virtual_key(VirtualKey::by_name("VK_A").unwrap()),
            transition: KeyTransition::Down,
            modifiers: None,
        };

        // println!("{}", action.to_text());

        // типа "SHIFT↓ + A↓ → A↑ + SHIFT↑"
        // ↓ нажата                                     A↓
        // ↑ отпущена                                   A↑
        // → нажаты последовательно                     SHIFT↓ → A↓
        // + нажаты вместе в любой последовательности   SHIFT↓ + A↓
        // ↓↑ нажата и отпущена                         A↓↑ = A↓ → A↑

        // запись макроса: 
        //  начало = 
        //              ни одна не нажата
        //              нажата хоть одна клавиша
        //  конец = 
        //              нажата хоть одна клавиша
        //              ни одна не нажата (или вышло время тамера если нужо без отпускания)  
        //              

    }
}
