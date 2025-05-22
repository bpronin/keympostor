use crate::keyboard::key_action::KeyAction;
use crate::keyboard::key_event::KeyEvent;
use crate::keyboard::key_modifiers::KeyModifiers;
use crate::keyboard::transform_rules::{KeyTransformProfile, KeyTransformRule};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct KeyTransformMap {
    map: HashMap<KeyAction, HashMap<KeyModifiers, KeyTransformRule>>,
}

impl KeyTransformMap {
    pub(crate) fn from_profile(profile: KeyTransformProfile) -> KeyTransformMap {
        let mut this = Self::default();
        for rule in profile.rules.items {
            this.put(rule)
        }
        this
    }

    fn get_group(
        &self,
        key_action: &KeyAction,
    ) -> Option<&HashMap<KeyModifiers, KeyTransformRule>> {
        self.map.get(key_action)
    }

    pub(crate) fn get(&self, event: &KeyEvent) -> Option<&KeyTransformRule> {
        if let Some(rules) = self.get_group(&event.trigger.action) {
            rules.get(&event.trigger.modifiers)
        } else {
            None
        }
    }

    fn put(&mut self, rule: KeyTransformRule) {
        let trigger = rule.source;
        self.map
            .entry(trigger.action)
            .or_default()
            .insert(trigger.modifiers, rule);
    }
}

impl Default for KeyTransformMap {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key_trigger::KeyTrigger;
use crate::keyboard::key_modifiers::{KM_ALL, KM_LALT, KM_LCTRL, KM_LSHIFT, KM_NONE};
    use crate::keyboard::transform_map::KeyAction;
    use crate::keyboard::transform_map::KeyEvent;
    use crate::keyboard::transform_map::KeyTransformMap;
    use crate::keyboard::transform_rules::KeyTransformRule;
    use crate::{assert_none, assert_not, key_action, key_event, key_rule};

    #[test]
    fn test_get_group() {
        let mut map = KeyTransformMap::default();

        map.put(key_rule!("A↓ : B↓"));
        map.put(key_rule!("[LEFT_SHIFT] A↓ : C↓"));
        map.put(key_rule!("[LEFT_CTRL] A↓ : D↓"));

        map.put(key_rule!("B↓ : BACKSPACE↓"));
        map.put(key_rule!("[LEFT_ALT] B↓ : ENTER↓"));
        map.put(key_rule!("[LEFT_ALT + LEFT_WIN] B↓ : NUM_ENTER↓"));

        assert_none!(map.get_group(&key_action!("A^")));
        assert_none!(map.get_group(&key_action!("C*")));

        let expected_for_a = [
            key_rule!("A↓ : B↓"),
            key_rule!("[LEFT_SHIFT] A↓ : C↓"),
            key_rule!("[LEFT_CTRL] A↓ : D↓"),
        ];
        let expected_for_b = [
            key_rule!("B↓ : BACKSPACE↓"),
            key_rule!("[LEFT_ALT] B↓  : ENTER↓"),
            key_rule!("[LEFT_ALT + LEFT_WIN] B↓ : NUM_ENTER↓"),
        ];

        let actual = map.get_group(&key_action!("A↓")).unwrap();
        assert!(
            expected_for_a
                .iter()
                .all(|rule| { actual.get(&rule.source.modifiers) == Some(rule) })
        );
        assert_not!(
            expected_for_b
                .iter()
                .all(|rule| { actual.get(&rule.source.modifiers) == Some(rule) })
        );

        let actual = map.get_group(&key_action!("B↓")).unwrap();
        assert!(
            expected_for_b
                .iter()
                .all(|rule| { actual.get(&rule.source.modifiers) == Some(rule) })
        );
        assert_not!(
            expected_for_a
                .iter()
                .all(|rule| { actual.get(&rule.source.modifiers) == Some(rule) })
        );
    }

    #[test]
    fn test_get() {
        let any_down = KM_ALL;
        let all_up = KM_NONE;
        let shift_down = KM_LSHIFT;
        let alt_down = KM_LALT;
        let ctrl_down = KM_LCTRL;
        let ctrl_alt_down = KM_LCTRL | KM_LALT;

        let mut map = KeyTransformMap::default();
        map.put(key_rule!("A↓ : B↓"));
        map.put(key_rule!("[]X↓ : Y↓"));
        map.put(key_rule!("[LEFT_SHIFT] A↓ : C↓"));
        map.put(key_rule!("[LEFT_ALT + LEFT_CTRL] A↓ : D↓"));

        assert_eq!(
            &key_rule!("A↓ : B↓"),
            map.get(&key_event!("A↓", any_down)).unwrap()
        );
        
        assert_eq!(
            &key_rule!("[]X↓ : Y↓"),
            map.get(&key_event!("X↓", all_up)).unwrap()
        );

        assert_eq!(
            &key_rule!("[LEFT_SHIFT] A↓ : C↓"),
            map.get(&key_event!("A↓", shift_down)).unwrap()
        );

        assert_eq!(
            &key_rule!("[LEFT_ALT + LEFT_CTRL]A↓ : D↓"),
            map.get(&key_event!("A↓", ctrl_alt_down)).unwrap()
        );

        assert_none!(map.get(&key_event!("B↓", all_up)));
        assert_none!(map.get(&key_event!("A↑", all_up)));
        assert_none!(map.get(&key_event!("LEFT_ALT↓", alt_down)));
        assert_none!(map.get(&key_event!("LEFT_SHIFT↓", shift_down)));
        assert_none!(map.get(&key_event!("LEFT_CTRL↓", ctrl_down)));
    }
}
