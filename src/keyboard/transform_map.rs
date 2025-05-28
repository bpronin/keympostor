use crate::keyboard::key_action::KeyAction;
use crate::keyboard::key_event::KeyEvent;
use crate::keyboard::key_modifiers::KeyModifiers;
use crate::keyboard::key_modifiers::KeyModifiers::{All, Any};
use crate::keyboard::transform_rules::{KeyTransformProfile, KeyTransformRule};
use fxhash::FxBuildHasher;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub(crate) struct KeyTransformMap {
    map: HashMap<KeyAction, HashMap<KeyModifiers, KeyTransformRule, FxBuildHasher>, FxBuildHasher>,
}

impl KeyTransformMap {
    pub(crate) fn from_profile(profile: KeyTransformProfile) -> KeyTransformMap {
        let mut this = Self::default();
        for rule in profile.rules.items {
            this.put(rule)
        }
        this
    }

    pub(crate) fn get(&self, event: &KeyEvent) -> Option<&KeyTransformRule> {
        let map = self.map.get(&event.action)?;
        map.get(&All(event.modifiers_state))
            .or_else(|| map.get(&Any))
    }

    fn put(&mut self, rule: KeyTransformRule) {
        let trigger = rule.trigger;
        self.map
            .entry(trigger.action)
            .or_default()
            .insert(trigger.modifiers, rule);
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key_modifiers::KeyModifiersState;
    use crate::keyboard::transform_map::KeyAction;
    use crate::keyboard::transform_map::KeyEvent;
    use crate::keyboard::transform_map::KeyTransformMap;
    use crate::keyboard::transform_rules::KeyTransformRule;
    use crate::{assert_none, key_action, key_event, key_rule};
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
        let mut map = KeyTransformMap::default();
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
        let mut map = KeyTransformMap::default();
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
        let mut map = KeyTransformMap::default();
        map.put(key_rule!("A↓ : B↓"));

        let expected = &key_rule!("A↓ : B↓");
        assert_eq!(expected, map.get(&key_event!("A↓", KS_ALL_UP)).unwrap());
        assert_eq!(expected, map.get(&key_event!("A↓", KS_LSHIFT)).unwrap());
        assert_eq!(expected, map.get(&key_event!("A↓", KS_LCTRL)).unwrap());
        assert_eq!(expected, map.get(&key_event!("A↓", KS_LALT)).unwrap());
        assert_eq!(expected, map.get(&key_event!("A↓", KS_LCTRL_LALT)).unwrap());
    }

    #[test]
    fn test_put_duplicates() {
        let mut map = KeyTransformMap::default();
        map.put(key_rule!("[LEFT_SHIFT] A↓ : B↓"));
        map.put(key_rule!("[LEFT_SHIFT] A↓ : B↓"));
        map.put(key_rule!("[LEFT_SHIFT] A↓ : B↓"));

        assert_eq!(1, map.map.len());
        assert_eq!(1, map.map.get(&key_action!("A↓")).unwrap().len());
        assert_eq!(
            &key_rule!("[LEFT_SHIFT] A↓ : B↓"),
            map.get(&key_event!("A↓", KS_LSHIFT)).unwrap()
        );
    }
}
