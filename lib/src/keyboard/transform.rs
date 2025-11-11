use crate::keyboard::action::KeyAction;
use crate::keyboard::event::KeyEvent;
use crate::keyboard::modifiers::KeyModifiers;
use crate::keyboard::modifiers::KeyModifiers::{All, Any};
use crate::keyboard::rules::{KeyTransformRule, KeyTransformRules};
use fxhash::FxHashMap;

#[derive(Debug, Default)]
pub(crate) struct KeyTransformMap {
    map: FxHashMap<KeyAction, FxHashMap<KeyModifiers, KeyTransformRule>>,
}

impl KeyTransformMap {
    pub(crate) fn new(rules: &KeyTransformRules) -> Self {
        let mut this = Self::default();
        for rule in rules.iter() {
            this.put(rule.clone())
        }
        this
    }

    pub(crate) fn get(&self, event: &KeyEvent) -> Option<&KeyTransformRule> {
        let map = self.map.get(&event.action)?;
        map.get(&All(event.modifiers))
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
    use crate::keyboard::modifiers::ModifierKeys;
    use crate::keyboard::transform::KeyAction;
    use crate::keyboard::transform::KeyEvent;
    use crate::keyboard::transform::KeyTransformMap;
    use crate::keyboard::rules::KeyTransformRule;
    use crate::{assert_none, key_action, key_event, key_rule};
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LCONTROL, VK_LMENU, VK_LSHIFT};

    static KS_ALL_UP: [bool; 256] = [false; 256];
    static KS_LSHIFT: [bool; 256] = {
        let mut keys = KS_ALL_UP;
        keys[VK_LSHIFT.0 as usize] = true;
        keys
    };
    static KS_LCTRL: [bool; 256] = {
        let mut keys = KS_ALL_UP;
        keys[VK_LCONTROL.0 as usize] = true;
        keys
    };
    static KS_LALT: [bool; 256] = {
        let mut keys = KS_ALL_UP;
        keys[VK_LMENU.0 as usize] = true;
        keys
    };
    static KS_LCTRL_LALT: [bool; 256] = {
        let mut keys = KS_ALL_UP;
        keys[VK_LCONTROL.0 as usize] = true;
        keys[VK_LMENU.0 as usize] = true;
        keys
    };

    #[test]
    fn test_put_get_normal() {
        let mut map = KeyTransformMap::default();
        map.put(key_rule!("[LEFT_SHIFT] A↓ : B↓"));
        map.put(key_rule!("[LEFT_ALT + LEFT_CTRL] A↓ : C↓"));

        assert_eq!(
            &key_rule!("[LEFT_SHIFT] A↓ : B↓"),
            map.get(&key_event!("A↓", &KS_LSHIFT)).unwrap()
        );

        assert_eq!(
            &key_rule!("[LEFT_ALT + LEFT_CTRL]A↓ : C↓"),
            map.get(&key_event!("A↓", &KS_LCTRL_LALT)).unwrap()
        );

        assert_none!(map.get(&key_event!("A↓", &KS_ALL_UP)));
        assert_none!(map.get(&key_event!("A↑", &KS_LSHIFT)));
        assert_none!(map.get(&key_event!("LEFT_ALT↓", &KS_LALT)));
        assert_none!(map.get(&key_event!("LEFT_SHIFT↓", &KS_LSHIFT)));
        assert_none!(map.get(&key_event!("LEFT_CTRL↓", &KS_LCTRL)));
    }

    #[test]
    fn test_get_no_modifiers() {
        let mut map = KeyTransformMap::default();
        map.put(key_rule!("[] A↓ : B↓"));

        assert_eq!(
            &key_rule!("[] A↓ : B↓"),
            map.get(&key_event!("A↓", &KS_ALL_UP)).unwrap()
        );
        assert_none!(map.get(&key_event!("A↓", &KS_LSHIFT)));
        assert_none!(map.get(&key_event!("A↓", &KS_LCTRL)));
        assert_none!(map.get(&key_event!("A↓", &KS_LALT)));
        assert_none!(map.get(&key_event!("A↓", &KS_LCTRL_LALT)));
    }

    #[test]
    fn test_get_ignore_modifiers() {
        let mut map = KeyTransformMap::default();
        map.put(key_rule!("A↓ : B↓"));

        let expected = &key_rule!("A↓ : B↓");
        assert_eq!(expected, map.get(&key_event!("A↓", &KS_ALL_UP)).unwrap());
        assert_eq!(expected, map.get(&key_event!("A↓", &KS_LSHIFT)).unwrap());
        assert_eq!(expected, map.get(&key_event!("A↓", &KS_LCTRL)).unwrap());
        assert_eq!(expected, map.get(&key_event!("A↓", &KS_LALT)).unwrap());
        assert_eq!(expected, map.get(&key_event!("A↓", &KS_LCTRL_LALT)).unwrap());
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
            map.get(&key_event!("A↓", &KS_LSHIFT)).unwrap()
        );
    }
}
