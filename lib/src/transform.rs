use crate::action::KeyAction;
use crate::modifiers::KeyModifiers;
use crate::modifiers::KeyModifiers::Any;
use crate::rule::KeyTransformRule;
use crate::trigger::KeyTrigger;
use fxhash::FxHashMap;
use std::slice::Iter;

#[derive(Debug, Default)]
pub(crate) struct KeyTransformMap {
    map: FxHashMap<KeyAction, FxHashMap<KeyModifiers, KeyTransformRule>>,
}

impl KeyTransformMap {
    pub(crate) fn new(rules: Iter<KeyTransformRule>) -> Self {
        let mut map: FxHashMap<KeyAction, FxHashMap<KeyModifiers, KeyTransformRule>> =
            Default::default();

        for rule in rules {
            let trigger = &rule.trigger;
            map.entry(trigger.action)
                .or_default()
                .insert(trigger.modifiers, rule.clone());
        }

        Self { map }
    }

    pub(crate) fn get(&self, trigger: &KeyTrigger) -> Option<&KeyTransformRule> {
        self.map
            .get(&trigger.action)?
            .get(&trigger.modifiers)
            .or_else(|| self.map.get(&trigger.action)?.get(&Any))
    }
}

#[cfg(test)]
mod tests {
    use crate::rule::KeyTransformRule;
    use crate::transform::KeyAction;
    use crate::transform::KeyTransformMap;
    use crate::trigger::KeyTrigger;
    use crate::{key_action, key_rule, key_trigger};
    use std::str::FromStr;

    #[test]
    fn test_get() {
        let map = KeyTransformMap::new(
            [
                key_rule!("[LEFT_SHIFT] A↓ : B↓"),
                key_rule!("[LEFT_ALT + LEFT_CTRL] A↓ : C↓"),
            ]
            .iter(),
        );

        assert_eq!(
            Some(&key_rule!("[LEFT_SHIFT] A↓ : B↓")),
            map.get(&key_trigger!("[LEFT_SHIFT] A↓"))
        );
        assert_eq!(
            Some(&key_rule!("[LEFT_ALT + LEFT_CTRL]A↓ : C↓")),
            map.get(&key_trigger!("[LEFT_CTRL + LEFT_ALT] A↓"))
        );
        assert_eq!(None, map.get(&key_trigger!("A↓")));
        assert_eq!(None, map.get(&key_trigger!("[LEFT_SHIFT] A↑")));
        assert_eq!(None, map.get(&key_trigger!("[LEFT_ALT] LEFT_ALT↓")));
        assert_eq!(None, map.get(&key_trigger!("[LEFT_SHIFT] LEFT_SHIFT↓")));
        assert_eq!(None, map.get(&key_trigger!("[LEFT_CTRL] LEFT_CTRL↓")));
    }

    #[test]
    fn test_get_any_modifiers() {
        let map = KeyTransformMap::new([key_rule!("A↓ : B↓")].iter());

        let rule = key_rule!("A↓ : B↓");
        let exp = Some(&rule);

        assert_eq!(exp, map.get(&key_trigger!("A↓")));
        assert_eq!(exp, map.get(&key_trigger!("[] A↓")));
        assert_eq!(exp, map.get(&key_trigger!("[LEFT_SHIFT] A↓")));
        assert_eq!(exp, map.get(&key_trigger!("[LEFT_CTRL] A↓")));
        assert_eq!(exp, map.get(&key_trigger!("[LEFT_ALT] A↓")));
        assert_eq!(exp, map.get(&key_trigger!("[LEFT_CTRL + LEFT_ALT] A↓")));
    }

    #[test]
    fn test_get_no_modifiers() {
        let map = KeyTransformMap::new([key_rule!("[] A↓ : B↓")].iter());

        assert_eq!(
            Some(&key_rule!("[] A↓ : B↓")),
            map.get(&key_trigger!("[] A↓"))
        );
        assert_eq!(None, map.get(&key_trigger!("[LEFT_SHIFT] A↓")));
        assert_eq!(None, map.get(&key_trigger!("[LEFT_CTRL] A↓")));
        assert_eq!(None, map.get(&key_trigger!("[LEFT_ALT] A↓")));
        assert_eq!(None, map.get(&key_trigger!("[LEFT_CTRL + LEFT_ALT] A↓")));
    }

    #[test]
    fn test_get_ignore_modifiers() {
        let map = KeyTransformMap::new([key_rule!("A↓ : B↓")].iter());

        let rule = key_rule!("A↓ : B↓");
        let exp = Some(&rule);

        assert_eq!(exp, map.get(&key_trigger!("A↓")));
        assert_eq!(exp, map.get(&key_trigger!("[LEFT_SHIFT] A↓")));
        assert_eq!(exp, map.get(&key_trigger!("[LEFT_CTRL] A↓")));
        assert_eq!(exp, map.get(&key_trigger!("[LEFT_ALT] A↓")));
        assert_eq!(exp, map.get(&key_trigger!("[LEFT_CTRL + LEFT_ALT] A↓")));
    }

    #[test]
    fn test_put_duplicates() {
        let map = KeyTransformMap::new(
            [
                key_rule!("[LEFT_SHIFT] A↓ : B↓"),
                key_rule!("[LEFT_SHIFT] A↓ : B↓"),
                key_rule!("[LEFT_SHIFT] A↓ : B↓"),
            ]
            .iter(),
        );

        assert_eq!(1, map.map.len());
        assert_eq!(1, map.map.get(&key_action!("A↓")).unwrap().len());
        assert_eq!(
            Some(&key_rule!("[LEFT_SHIFT] A↓ : B↓")),
            map.get(&key_trigger!("[LEFT_SHIFT] A↓"))
        );
    }
}
