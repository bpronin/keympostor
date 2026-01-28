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
    use std::ops::Deref;
    use std::str::FromStr;
    use std::sync::LazyLock;
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LCONTROL, VK_LMENU, VK_LSHIFT};

    static KS_ALL_UP: LazyLock<KeyboardState> = LazyLock::new(|| KeyboardState::new());
    static KS_LSHIFT: LazyLock<KeyboardState> =
        LazyLock::new(|| KeyboardState::from_bits(&[VK_LSHIFT.0 as u8]));
    static KS_LCTRL: LazyLock<KeyboardState> =
        LazyLock::new(|| KeyboardState::from_bits(&[VK_LCONTROL.0 as u8]));
    static KS_LALT: LazyLock<KeyboardState> =
        LazyLock::new(|| KeyboardState::from_bits(&[VK_LMENU.0 as u8]));
    static KS_LCTRL_LALT: LazyLock<KeyboardState> =
        LazyLock::new(|| KeyboardState::from_bits(&[VK_LCONTROL.0 as u8, VK_LMENU.0 as u8]));

    #[test]
    fn test_put_get_normal() {
        let mut map = KeyTransformMap::default();
        map.put(key_rule!("[LEFT_SHIFT] A↓ : B↓"));
        map.put(key_rule!("[LEFT_ALT + LEFT_CTRL] A↓ : C↓"));

        assert_eq!(
            &key_rule!("[LEFT_SHIFT] A↓ : B↓"),
            map.get(&key_event!("A↓", KS_LSHIFT.deref()))
                .unwrap()
                .as_ref()
        );

        assert_eq!(
            &key_rule!("[LEFT_ALT + LEFT_CTRL]A↓ : C↓"),
            map.get(&key_event!("A↓", KS_LCTRL_LALT.deref()))
                .unwrap()
                .as_ref()
        );

        assert_none!(map.get(&key_event!("A↓", KS_ALL_UP.deref())));
        assert_none!(map.get(&key_event!("A↑", KS_LSHIFT.deref())));
        assert_none!(map.get(&key_event!("LEFT_ALT↓", KS_LALT.deref())));
        assert_none!(map.get(&key_event!("LEFT_SHIFT↓", KS_LSHIFT.deref())));
        assert_none!(map.get(&key_event!("LEFT_CTRL↓", KS_LCTRL.deref())));
    }

    #[test]
    fn test_get_no_modifiers() {
        let mut map = KeyTransformMap::default();
        map.put(key_rule!("[] A↓ : B↓"));

        assert_eq!(
            &key_rule!("[] A↓ : B↓"),
            map.get(&key_event!("A↓", KS_ALL_UP.deref()))
                .unwrap()
                .as_ref()
        );
        assert_none!(map.get(&key_event!("A↓", KS_LSHIFT.deref())));
        assert_none!(map.get(&key_event!("A↓", KS_LCTRL.deref())));
        assert_none!(map.get(&key_event!("A↓", KS_LALT.deref())));
        assert_none!(map.get(&key_event!("A↓", KS_LCTRL_LALT.deref())));
    }

    #[test]
    fn test_get_ignore_modifiers() {
        let mut map = KeyTransformMap::default();
        map.put(key_rule!("A↓ : B↓"));

        let expected = &key_rule!("A↓ : B↓");
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", KS_ALL_UP.deref()))
                .unwrap()
                .as_ref()
        );
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", KS_LSHIFT.deref()))
                .unwrap()
                .as_ref()
        );
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", KS_LCTRL.deref()))
                .unwrap()
                .as_ref()
        );
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", KS_LALT.deref()))
                .unwrap()
                .as_ref()
        );
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", KS_LCTRL_LALT.deref()))
                .unwrap()
                .as_ref()
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
            map.get(&key_event!("A↓", KS_LSHIFT.deref()))
                .unwrap()
                .as_ref()
        );
    }
}
