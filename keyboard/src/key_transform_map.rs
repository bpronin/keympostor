use crate::key::{KeyCode, MAX_VK_CODE};
use crate::key_action::KeyTransition;
use crate::key_event::KeyEvent;
use crate::key_modifiers::KeyModifiers;
use crate::key_transform_rule::{KeyTransformProfile, KeyTransformRule};
use std::array::from_fn;

#[derive(Debug)]
pub struct KeyTransformMap {
    map: [[Vec<KeyTransformRule>; MAX_VK_CODE]; 2],
}

impl KeyTransformMap {
    pub(crate) fn from_profile(profile: KeyTransformProfile) -> KeyTransformMap {
        let mut this = Self::default();
        for rule in profile.rules {
            this.put(rule)
        }
        this
    }

    fn get_group(&self, key: &KeyCode, transition: KeyTransition) -> &[KeyTransformRule] {
        &self.map[transition.is_up() as usize][key.id()]
    }

    pub(crate) fn get(
        &self,
        event: &KeyEvent,
        get_modifiers: fn() -> KeyModifiers,
    ) -> Option<&KeyTransformRule> {
        let rules = self.get_group(&event.key(), event.transition());
        //todo make it HashMap
        for rule in rules {
            if rule.source.modifiers == get_modifiers() {
                return Some(rule);
            }
        }

        None
    }

    fn put(&mut self, rule: KeyTransformRule) {
        let trigger = &rule.source;
        self.map[trigger.transition().is_up() as usize][trigger.key().id()].push(rule);
    }
}

impl Default for KeyTransformMap {
    fn default() -> Self {
        Self {
            map: [from_fn(|_| vec![]), from_fn(|_| vec![])],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::key_action::KeyTransition::{Down, Up};
    use crate::key_event::KeyEvent;
    use crate::key_modifiers::{KM_LALT, KM_LCONTROL, KM_LSHIFT, KM_NONE};
    use crate::key_transform_map::KeyCode;
    use crate::key_transform_map::KeyTransformMap;
    use crate::key_transform_rule::KeyTransformRule;
    use crate::{assert_none, key, key_event, key_rule};
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_A, VK_B, VK_MENU, VK_SHIFT};
    use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, LLKHF_UP};

    #[test]
    fn test_get_group() {
        let mut map = KeyTransformMap::default();

        map.put(key_rule!("VK_A↓ : VK_B↓"));
        map.put(key_rule!("[SHIFT] VK_A↓ : VK_C↓"));
        map.put(key_rule!("[CONTROL] VK_A↓ : VK_D↓"));

        map.put(key_rule!("SC_A↓ : SC_C↓"));
        map.put(key_rule!("[ALT] SC_B↓ : SC_0x1C↓"));

        assert!(map.get_group(&key!("VK_A"), Up).is_empty());
        assert!(map.get_group(&key!("VK_C"), Up).is_empty());

        let expected = [
            key_rule!("VK_A↓ : VK_B↓"),
            key_rule!("[SHIFT] VK_A↓ : VK_C↓"),
            key_rule!("[CONTROL] VK_A↓ : VK_D↓"),
            key_rule!("SC_A↓ : SC_C↓"), /* VK_A converted from SC_A */
        ];

        assert_eq!(expected, map.get_group(&key!("VK_A"), Down));
        
        let expected = [
            key_rule!("[ALT] SC_B↓ : SC_0x1C↓")
        ];
        /* VK_B converted from SC_B */
        assert_eq!(expected, map.get_group(&key!("VK_B"), Down)); 

    }

    #[test]
    fn test_get() {
        let all_up = || KM_NONE;
        let shift_down = || KM_LSHIFT;
        let alt_down = || KM_LALT;
        let ctrl_down = || KM_LCONTROL;
        let ctrl_alt_down = || KM_LCONTROL | KM_LALT;

        let mut map = KeyTransformMap::default();
        map.put(key_rule!("VK_A↓ : VK_B↓"));
        map.put(key_rule!("[LSHIFT] VK_A↓ : VK_C↓"));
        map.put(key_rule!("[LALT + LCONTROL] VK_A↓ : VK_D↓"));

        let group = map.get_group(&key!("VK_A"), Down);
        assert_eq!(3, group.len());

        assert_none!(map.get(&key_event!(VK_B.0, false), all_up));
        assert_none!(map.get(&key_event!(VK_A.0, true), all_up));
        assert_none!(map.get(&key_event!(VK_MENU.0, false), alt_down));
        assert_none!(map.get(&key_event!(VK_SHIFT.0, false), shift_down));
        assert_none!(map.get(&key_event!(VK_A.0, false), ctrl_down));

        assert_eq!(
            &key_rule!("VK_A↓ : VK_B↓"),
            map.get(&key_event!(VK_A.0, false), all_up).unwrap()
        );

        assert_eq!(
            &key_rule!("[LSHIFT]VK_A↓ : VK_C↓"),
            map.get(&key_event!(VK_A.0, false), shift_down).unwrap()
        );

        assert_eq!(
            &key_rule!("[LALT + LCONTROL]VK_A↓ : VK_D↓"),
            map.get(&key_event!(VK_A.0, false), ctrl_alt_down).unwrap()
        );
    }
}
