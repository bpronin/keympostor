use crate::config::TransformRule;
use crate::key_action::{KeyAction, KeyActionSequence, KeyTransition};
use crate::key_id::{ScanCode, VirtualKey};
use crate::key_modifier::KeyModifiers;
use std::collections::HashMap;

#[derive(Debug)]
struct ScancodeTransformMap {
    map:
        HashMap<ScanCode, HashMap<KeyTransition, HashMap<Option<KeyModifiers>, KeyActionSequence>>>,
}

impl ScancodeTransformMap {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn get(
        &self,
        transition: KeyTransition,
        scancode: ScanCode,
        modifiers: Option<KeyModifiers>,
    ) -> Option<&KeyActionSequence> {
        let submap = self.map.get(&scancode)?.get(&transition)?;
        submap.get(&modifiers).or(submap.get(&None))
    }

    fn put(
        &mut self,
        transition: KeyTransition,
        scancode: ScanCode,
        modifiers: Option<KeyModifiers>,
        target: KeyActionSequence,
    ) {
        self.map
            .entry(scancode)
            .or_default()
            .entry(transition)
            .or_default()
            .insert(modifiers, target);
    }
}

#[derive(Debug)]
struct VirtualKeyTransformMap {
    map: HashMap<
        VirtualKey,
        HashMap<KeyTransition, HashMap<Option<KeyModifiers>, KeyActionSequence>>,
    >,
}

impl VirtualKeyTransformMap {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn get(
        &self,
        transition: KeyTransition,
        virtual_key: VirtualKey,
        modifiers: Option<KeyModifiers>,
    ) -> Option<&KeyActionSequence> {
        let submap = self.map.get(&virtual_key)?.get(&transition)?;
        submap.get(&modifiers).or(submap.get(&None))
    }

    fn put(
        &mut self,
        transition: KeyTransition,
        virtual_key: VirtualKey,
        modifiers: Option<KeyModifiers>,
        target: KeyActionSequence,
    ) {
        self.map
            .entry(virtual_key)
            .or_default()
            .entry(transition)
            .or_default()
            .insert(modifiers, target);
    }
}

#[derive(Debug)]
pub(crate) struct TransformMap {
    scancode_map: ScancodeTransformMap,
    virtual_key_map: VirtualKeyTransformMap,
}

impl TransformMap {
    pub(crate) fn new() -> Self {
        Self {
            scancode_map: ScancodeTransformMap::new(),
            virtual_key_map: VirtualKeyTransformMap::new(),
        }
    }

    pub(crate) fn from_config(config: Vec<TransformRule>) -> Result<Self, String> {
        let mut this = Self::new();
        for item in config {
            this.put(item.source, item.target);
        }
        
        Ok(this)
    }

    pub(crate) fn get(&self, source: &KeyAction) -> Option<&KeyActionSequence> {
        if let Some(scancode) = source.key.scancode {
            self.scancode_map
                .get(source.transition, *scancode, source.modifiers)
        } else if let Some(virtual_key) = source.key.virtual_key {
            self.virtual_key_map
                .get(source.transition, *virtual_key, source.modifiers)
        } else {
            panic!("Action key cannot be blank.");
        }
    }

    fn put(&mut self, source: KeyAction, target: KeyActionSequence) {
        if let Some(scancode) = source.key.scancode {
            self.scancode_map
                .put(source.transition, *scancode, source.modifiers, target);
        } else if let Some(virtual_key) = source.key.virtual_key {
            self.virtual_key_map
                .put(source.transition, *virtual_key, source.modifiers, target);
        } else {
            panic!("Action key cannot be blank.");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::key_action::{KeyAction, KeyActionSequence, KeyTransition};
    use crate::key_id::KeyIdentifier;
    use crate::key_id::VirtualKey;
    use crate::key_modifier::{KM_LEFT_SHIFT, KM_NONE, KM_RIGHT_WIN};
    use crate::transform::TransformMap;

    #[test]
    fn test_get() {
        let a_key = KeyIdentifier::from_virtual_key(VirtualKey::by_name("VK_A").unwrap());
        let b_key = KeyIdentifier::from_virtual_key(VirtualKey::by_name("VK_B").unwrap());
        let c_key = KeyIdentifier::from_virtual_key(VirtualKey::by_name("VK_C").unwrap());
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
        let a_key = KeyIdentifier::from_virtual_key(VirtualKey::by_name("VK_A").unwrap());
        let b_key = KeyIdentifier::from_virtual_key(VirtualKey::by_name("VK_B").unwrap());
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
