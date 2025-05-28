use crate::keyboard::key_action::KeyAction;
use crate::keyboard::key_const::{MAX_SCAN_CODE, MAX_VK_CODE};
use crate::keyboard::key_event::KeyEvent;
use crate::keyboard::key_modifiers::KeyModifiers;
use crate::keyboard::key_modifiers::KeyModifiers::{All, Any};
use crate::keyboard::transform_rules::{KeyTransformProfile, KeyTransformRule};
use std::collections::HashMap;

type Group = HashMap<KeyModifiers, KeyTransformRule>;

#[derive(Debug)]
pub struct KeyTransformMatrix {
    matrix: Box<[Vec<Vec<Vec<Option<Group>>>>]>,
}

impl Default for KeyTransformMatrix {
    fn default() -> Self {
        Self {
            matrix: vec![vec![vec![vec![None; MAX_VK_CODE]; MAX_SCAN_CODE]; 2]; 2]
                .into_boxed_slice(),
        }
    }
}

impl KeyTransformMatrix {
    pub(crate) fn from_profile(profile: KeyTransformProfile) -> KeyTransformMatrix {
        let mut this = Self::default();
        for rule in profile.rules.items {
            this.put(rule)
        }
        this
    }

    pub(crate) fn get(&self, event: &KeyEvent) -> Option<&KeyTransformRule> {
        if let Some(map) = self.get_group(&event.action) {
            map.get(&All(event.modifiers_state))
                .or_else(|| map.get(&Any))
        } else {
            None
        }
    }

    fn put(&mut self, rule: KeyTransformRule) {
        let trigger = rule.trigger;
        let action = trigger.action;

        if let Some(map) = self.get_group_mut(&action) {
            map.insert(trigger.modifiers, rule);
        } else {
            let mut map = HashMap::new();
            map.insert(trigger.modifiers, rule);
            self.put_group(&action, map);
        }
    }

    fn get_group_mut(&mut self, action: &KeyAction) -> &mut Option<Group> {
        &mut self.matrix[action.transition as usize][action.key.is_ext_scan_code as usize]
            [action.key.scan_code as usize][action.key.vk_code as usize]
    }

    fn get_group(&self, action: &KeyAction) -> &Option<Group> {
        &self.matrix[action.transition as usize][action.key.is_ext_scan_code as usize]
            [action.key.scan_code as usize][action.key.vk_code as usize]
    }

    fn put_group(&mut self, action: &KeyAction, group: Group) {
        self.matrix[action.transition as usize][action.key.is_ext_scan_code as usize]
            [action.key.scan_code as usize][action.key.vk_code as usize] = Some(group);
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key_modifiers::KeyModifiersState;
    use crate::keyboard::transform_matrix::KeyEvent;
    use crate::keyboard::transform_matrix::KeyTransformMatrix;
    use crate::keyboard::transform_rules::KeyTransformRule;
    use crate::{assert_none, key_event, key_rule};
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LCONTROL, VK_LMENU, VK_LSHIFT};

    static KS_ALL_UP: [u8; 256] = [0u8; 256];
    static KS_LSHIFT: [u8; 256] = {
        let mut keys = KS_ALL_UP;
        keys[VK_LSHIFT.0 as usize] = 0x80;
        keys
    };
    static KS_LCTRL: [u8; 256] = {
        let mut keys = KS_ALL_UP;
        keys[VK_LCONTROL.0 as usize] = 0x80;
        keys
    };
    static KS_LALT: [u8; 256] = {
        let mut keys = KS_ALL_UP;
        keys[VK_LMENU.0 as usize] = 0x80;
        keys
    };
    static KS_LCTRL_LALT: [u8; 256] = {
        let mut keys = KS_ALL_UP;
        keys[VK_LCONTROL.0 as usize] = 0x80;
        keys[VK_LMENU.0 as usize] = 0x80;
        keys
    };

    #[test]
    fn test_put_get_normal() {
        let mut map = KeyTransformMatrix::default();
        map.put(key_rule!("[LEFT_SHIFT] A↓ : B↓"));
        map.put(key_rule!("[LEFT_ALT + LEFT_CTRL] A↓ : C↓"));

        assert_eq!(
            &key_rule!("[LEFT_SHIFT] A↓ : B↓"),
            map.get(&key_event!("A↓", KS_LSHIFT)).unwrap()
        );

        assert_eq!(
            &key_rule!("[LEFT_ALT + LEFT_CTRL]A↓ : C↓"),
            map.get(&key_event!("A↓", KS_LCTRL_LALT)).unwrap()
        );

        assert_none!(map.get(&key_event!("A↓", KS_ALL_UP)));
        assert_none!(map.get(&key_event!("A↑", KS_LSHIFT)));
        assert_none!(map.get(&key_event!("LEFT_ALT↓", KS_LALT)));
        assert_none!(map.get(&key_event!("LEFT_SHIFT↓", KS_LSHIFT)));
        assert_none!(map.get(&key_event!("LEFT_CTRL↓", KS_LCTRL)));
    }

    #[test]
    fn test_get_no_modifiers() {
        let mut map = KeyTransformMatrix::default();
        map.put(key_rule!("[] A↓ : B↓"));

        assert_eq!(
            &key_rule!("[] A↓ : B↓"),
            map.get(&key_event!("A↓", KS_ALL_UP)).unwrap()
        );
        assert_none!(map.get(&key_event!("A↓", KS_LSHIFT)));
        assert_none!(map.get(&key_event!("A↓", KS_LCTRL)));
        assert_none!(map.get(&key_event!("A↓", KS_LALT)));
        assert_none!(map.get(&key_event!("A↓", KS_LCTRL_LALT)));
    }

    #[test]
    fn test_get_ignore_modifiers() {
        let mut map = KeyTransformMatrix::default();
        map.put(key_rule!("A↓ : B↓"));

        let expected = &key_rule!("A↓ : B↓");
        assert_eq!(expected, map.get(&key_event!("A↓", KS_ALL_UP)).unwrap());
        assert_eq!(expected, map.get(&key_event!("A↓", KS_LSHIFT)).unwrap());
        assert_eq!(expected, map.get(&key_event!("A↓", KS_LCTRL)).unwrap());
        assert_eq!(expected, map.get(&key_event!("A↓", KS_LALT)).unwrap());
        assert_eq!(expected, map.get(&key_event!("A↓", KS_LCTRL_LALT)).unwrap());
    }
}
