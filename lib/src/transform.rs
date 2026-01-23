use crate::action::KeyAction;
use crate::event::KeyEvent;
use crate::modifiers::KeyModifiers;
use crate::modifiers::KeyModifiers::{All, Any};
use crate::rules::{KeyTransformRule, KeyTransformRules};
use fxhash::FxHashMap;
use std::rc::Rc;

#[derive(Debug, Default)]
pub(crate) struct KeyTransformMap {
    map: FxHashMap<KeyAction, FxHashMap<KeyModifiers, Rc<KeyTransformRule>>>,
}

impl KeyTransformMap {
    pub(crate) fn new(rules: &KeyTransformRules) -> Self {
        let mut this = Self::default();
        for rule in rules.iter() {
            this.put(rule.clone())
        }
        this
    }

    pub(crate) fn get(&self, event: &KeyEvent) -> Option<&Rc<KeyTransformRule>> {
        let map = self.map.get(&event.action)?;
        map.get(&All(event.modifiers)).or_else(|| map.get(&Any))
    }

    fn put(&mut self, rule: KeyTransformRule) {
        let trigger = rule.trigger;
        self.map
            .entry(trigger.action)
            .or_default()
            .insert(trigger.modifiers, Rc::new(rule));
    }
}

#[cfg(test)]
mod tests {
    use crate::modifiers::ModifierKeys;
    use crate::rules::KeyTransformRule;
    use crate::state::KeyboardState;
    use crate::transform::KeyAction;
    use crate::transform::KeyEvent;
    use crate::transform::KeyTransformMap;
    use crate::{assert_none, key_action, key_event, key_rule};
    use std::str::FromStr;
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LCONTROL, VK_LMENU, VK_LSHIFT};

    static KS_ALL_UP: KeyboardState = KeyboardState::new();
    static KS_LSHIFT: KeyboardState = {
        let mut keys = KeyboardState::new();
        keys.set(VK_LSHIFT.0 as u8, true);
        keys
    };
    static KS_LCTRL: KeyboardState = {
        let mut keys = KeyboardState::new();
        keys.set(VK_LCONTROL.0 as u8, true);
        keys
    };
    static KS_LALT: KeyboardState = {
        let mut keys = KeyboardState::new();
        keys.set(VK_LMENU.0 as u8, true);
        keys
    };
    static KS_LCTRL_LALT: KeyboardState = {
        let mut keys = KeyboardState::new();
        keys.set(VK_LCONTROL.0 as u8, true);
        keys.set(VK_LMENU.0 as u8, true);
        keys
    };

    #[test]
    fn test_put_get_normal() {
        let mut map = KeyTransformMap::default();
        map.put(key_rule!("[LEFT_SHIFT] A↓ : B↓"));
        map.put(key_rule!("[LEFT_ALT + LEFT_CTRL] A↓ : C↓"));

        assert_eq!(
            &key_rule!("[LEFT_SHIFT] A↓ : B↓"),
            map.get(&key_event!("A↓", &KS_LSHIFT)).unwrap().as_ref()
        );

        assert_eq!(
            &key_rule!("[LEFT_ALT + LEFT_CTRL]A↓ : C↓"),
            map.get(&key_event!("A↓", &KS_LCTRL_LALT)).unwrap().as_ref()
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
            map.get(&key_event!("A↓", &KS_ALL_UP)).unwrap().as_ref()
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
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", &KS_ALL_UP)).unwrap().as_ref()
        );
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", &KS_LSHIFT)).unwrap().as_ref()
        );
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", &KS_LCTRL)).unwrap().as_ref()
        );
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", &KS_LALT)).unwrap().as_ref()
        );
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", &KS_LCTRL_LALT)).unwrap().as_ref()
        );
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
            map.get(&key_event!("A↓", &KS_LSHIFT)).unwrap().as_ref()
        );
    }
}
