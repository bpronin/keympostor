use crate::key_action::{KeyAction, KeyActionSequence};
use crate::key_code::{ScanCode, VirtualKey};
use crate::key_modifier::KeyModifiers;
use crate::key_transition::KeyTransition;
use crate::profile::TransformRule;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
struct KeyCodeTransformMap<K: Hash + Eq> {
    map: HashMap<K, HashMap<KeyTransition, HashMap<Option<KeyModifiers>, KeyActionSequence>>>,
}

impl<K: Hash + Eq> KeyCodeTransformMap<K> {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn get(
        &self,
        transition: KeyTransition,
        key: K,
        modifiers: Option<KeyModifiers>,
    ) -> Option<&KeyActionSequence> {
        let submap = self.map.get(&key)?.get(&transition)?;
        submap.get(&modifiers).or(submap.get(&None))
    }

    fn put(
        &mut self,
        transition: KeyTransition,
        key: K,
        modifiers: Option<KeyModifiers>,
        target: KeyActionSequence,
    ) {
        self.map
            .entry(key)
            .or_default()
            .entry(transition)
            .or_default()
            .insert(modifiers, target);
    }
}

#[derive(Debug)]
pub(crate) struct TransformMap {
    scancode_map: KeyCodeTransformMap<ScanCode>,
    virtual_key_map: KeyCodeTransformMap<VirtualKey>,
}

impl TransformMap {
    pub(crate) fn new() -> Self {
        Self {
            scancode_map: KeyCodeTransformMap::new(),
            virtual_key_map: KeyCodeTransformMap::new(),
        }
    }

    pub(crate) fn from_rules(rules: Vec<TransformRule>) -> Result<Self, String> {
        let mut this = Self::new();
        for item in rules {
            this.put(item.source, item.target);
        }

        Ok(this)
    }

    fn get_from_scancodes(&self, source: &KeyAction) -> Option<&KeyActionSequence> {
        if let Some(key) = source.key.scancode {
            self.scancode_map
                .get(source.transition, *key, source.modifiers)
        } else {
            None
        }
    }

    fn get_from_virtual_keys(&self, source: &KeyAction) -> Option<&KeyActionSequence> {
        if let Some(key) = source.key.virtual_key {
            self.virtual_key_map
                .get(source.transition, *key, source.modifiers)
        } else {
            None
        }
    }

    pub(crate) fn get(&self, source: &KeyAction) -> Option<&KeyActionSequence> {
        self.get_from_scancodes(source)
            .or(self.get_from_virtual_keys(source))
    }

    fn put(&mut self, source: KeyAction, target: KeyActionSequence) {
        if let Some(key) = source.key.scancode {
            self.scancode_map
                .put(source.transition, *key, source.modifiers, target);
        } else if let Some(key) = source.key.virtual_key {
            self.virtual_key_map
                .put(source.transition, *key, source.modifiers, target);
        } else {
            panic!("Action key cannot be blank.");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::key_action::{KeyAction, KeyActionSequence};
    use crate::key_code::Key;
    use crate::key_code::VirtualKey;
    use crate::key_modifier::{KM_LEFT_SHIFT, KM_NONE, KM_RIGHT_WIN};
    use crate::key_transition::KeyTransition;
    use crate::transform::TransformMap;

    #[test]
    fn test_get() {
        let a_key = Key::from_virtual_key(VirtualKey::by_name("VK_A").unwrap());
        let b_key = Key::from_virtual_key(VirtualKey::by_name("VK_B").unwrap());
        let c_key = Key::from_virtual_key(VirtualKey::by_name("VK_C").unwrap());
        let target = KeyActionSequence::from(vec![KeyAction {
            key: b_key,
            transition: KeyTransition::Up,
            modifiers: Some(KM_NONE),
        }]);

        let mut map = TransformMap::new();
        map.put(
            KeyAction {
                key: a_key,
                transition: KeyTransition::Up,
                modifiers: Some(KM_NONE),
            },
            target.clone(),
        );

        assert_eq!(
            map.get(&KeyAction {
                key: a_key,
                transition: KeyTransition::Up,
                modifiers: Some(KM_NONE),
            }),
            Some(&target)
        );

        let source = KeyAction {
            key: c_key, /* differs */
            transition: KeyTransition::Up,
            modifiers: Some(KM_NONE),
        };
        assert!(map.get(&source).is_none());

        let source = KeyAction {
            key: a_key,
            transition: KeyTransition::Down, /* differs */
            modifiers: Some(KM_NONE),
        };
        assert!(map.get(&source).is_none());

        let source = KeyAction {
            key: a_key,
            transition: KeyTransition::Up,
            modifiers: Some(KM_LEFT_SHIFT), /* differs */
        };
        assert!(map.get(&source).is_none());
    }

    #[test]
    fn test_any_modifier() {
        let a_key = Key::from_virtual_key(VirtualKey::by_name("VK_A").unwrap());
        let b_key = Key::from_virtual_key(VirtualKey::by_name("VK_B").unwrap());
        let target = KeyActionSequence::from(vec![KeyAction {
            key: b_key,
            transition: KeyTransition::Up,
            modifiers: Some(KM_NONE),
        }]);

        let mut map = TransformMap::new();
        map.put(
            KeyAction {
                key: a_key,
                transition: KeyTransition::Up,
                modifiers: None, /* == any */
            },
            target.clone(),
        );

        assert_eq!(
            map.get(&KeyAction {
                key: a_key,
                transition: KeyTransition::Up,
                modifiers: Some(KM_NONE),
            }),
            Some(&target)
        );

        assert_eq!(
            map.get(&KeyAction {
                key: a_key,
                transition: KeyTransition::Up,
                modifiers: Some(KM_LEFT_SHIFT), /* differs */
            }),
            Some(&target)
        );

        assert_eq!(
            map.get(&KeyAction {
                key: a_key,
                transition: KeyTransition::Up,
                modifiers: Some(KM_LEFT_SHIFT | KM_RIGHT_WIN), /* differs */
            }),
            Some(&target)
        );
    }
}
