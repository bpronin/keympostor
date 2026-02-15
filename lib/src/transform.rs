use crate::action::KeyAction;
use crate::event::KeyEvent;
use crate::modifiers::KeyModifiers;
use crate::modifiers::KeyModifiers::{All, Any};
use crate::rules::{KeyTransformRule, KeyTransformRules};
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
        map.get(&All(event.modifiers)).or_else(|| map.get(&Any))
    }

    fn put(&mut self, rule: KeyTransformRule) {
        let trigger = &rule.trigger;
        self.map
            .entry(trigger.action)
            .or_default()
            .insert(trigger.modifiers, rule);
    }
}

#[cfg(test)]
mod tests {
    use crate::key::Key::{LeftAlt, LeftCtrl, LeftShift};
    use crate::modifiers::ModifierKeys;
    use crate::rules::KeyTransformRule;
    use crate::state::tests::state_from_keys;
    use crate::state::KeyboardState;
    use crate::transform::KeyAction;
    use crate::transform::KeyEvent;
    use crate::transform::KeyTransformMap;
    use crate::{assert_none, key_action, key_event, key_rule};
    use std::ops::Deref;
    use std::str::FromStr;
    use std::sync::LazyLock;

    static KS_ALL_UP: LazyLock<KeyboardState> = LazyLock::new(KeyboardState::new);
    static KS_LEFT_SHIFT: LazyLock<KeyboardState> = LazyLock::new(|| state_from_keys(&[LeftShift]));
    static KS_LEFT_CTRL: LazyLock<KeyboardState> = LazyLock::new(|| state_from_keys(&[LeftCtrl]));
    static KS_LEFT_ALT: LazyLock<KeyboardState> = LazyLock::new(|| state_from_keys(&[LeftAlt]));
    static KS_LEFT_CTRL_ALT: LazyLock<KeyboardState> =
        LazyLock::new(|| state_from_keys(&[LeftCtrl, LeftAlt]));

    #[test]
    fn test_put_get_normal() {
        let mut map = KeyTransformMap::default();
        map.put(key_rule!("[LEFT_SHIFT] A↓ : B↓"));
        map.put(key_rule!("[LEFT_ALT + LEFT_CTRL] A↓ : C↓"));

        assert_eq!(
            &key_rule!("[LEFT_SHIFT] A↓ : B↓"),
            map.get(&key_event!("A↓", KS_LEFT_SHIFT.deref())).unwrap()
        );

        assert_eq!(
            &key_rule!("[LEFT_ALT + LEFT_CTRL]A↓ : C↓"),
            map.get(&key_event!("A↓", KS_LEFT_CTRL_ALT.deref()))
                .unwrap()
        );

        assert_none!(map.get(&key_event!("A↓", KS_ALL_UP.deref())));
        assert_none!(map.get(&key_event!("A↑", KS_LEFT_SHIFT.deref())));
        assert_none!(map.get(&key_event!("LEFT_ALT↓", KS_LEFT_ALT.deref())));
        assert_none!(map.get(&key_event!("LEFT_SHIFT↓", KS_LEFT_SHIFT.deref())));
        assert_none!(map.get(&key_event!("LEFT_CTRL↓", KS_LEFT_CTRL.deref())));
    }

    #[test]
    fn test_get_no_modifiers() {
        let mut map = KeyTransformMap::default();
        map.put(key_rule!("[] A↓ : B↓"));

        assert_eq!(
            &key_rule!("[] A↓ : B↓"),
            map.get(&key_event!("A↓", KS_ALL_UP.deref())).unwrap()
        );
        assert_none!(map.get(&key_event!("A↓", KS_LEFT_SHIFT.deref())));
        assert_none!(map.get(&key_event!("A↓", KS_LEFT_CTRL.deref())));
        assert_none!(map.get(&key_event!("A↓", KS_LEFT_ALT.deref())));
        assert_none!(map.get(&key_event!("A↓", KS_LEFT_CTRL_ALT.deref())));
    }

    #[test]
    fn test_get_ignore_modifiers() {
        let mut map = KeyTransformMap::default();
        map.put(key_rule!("A↓ : B↓"));

        let expected = &key_rule!("A↓ : B↓");
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", KS_ALL_UP.deref())).unwrap()
        );
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", KS_LEFT_SHIFT.deref())).unwrap()
        );
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", KS_LEFT_CTRL.deref())).unwrap()
        );
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", KS_LEFT_ALT.deref())).unwrap()
        );
        assert_eq!(
            expected,
            map.get(&key_event!("A↓", KS_LEFT_CTRL_ALT.deref()))
                .unwrap()
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
            map.get(&key_event!("A↓", KS_LEFT_SHIFT.deref())).unwrap()
        );
    }
}
