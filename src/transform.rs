use crate::config::TransformRule;
use crate::key_action::{KeyAction, KeyActionSequence, KeyTransition};
use crate::key_id::{ScanCode, VirtualKey};
use crate::key_modifier::KeyModifiers;
use std::collections::HashMap;

#[derive(Debug)]
struct ScancodeTransformMap {
    map: HashMap<ScanCode, HashMap<KeyTransition, HashMap<KeyModifiers, KeyActionSequence>>>,
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
        modifiers: KeyModifiers,
    ) -> Option<&KeyActionSequence> {
        self.map.get(&scancode)?.get(&transition)?.get(&modifiers)
    }

    fn put(
        &mut self,
        transition: KeyTransition,
        scancode: ScanCode,
        modifiers: KeyModifiers,
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
    map: HashMap<VirtualKey, HashMap<KeyTransition, HashMap<KeyModifiers, KeyActionSequence>>>,
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
        modifiers: KeyModifiers,
    ) -> Option<&KeyActionSequence> {
        self.map.get(&virtual_key)?.get(&transition)?.get(&modifiers)
    }

    fn put(
        &mut self,
        transition: KeyTransition,
        virtual_key: VirtualKey,
        modifiers: KeyModifiers,
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
    use crate::key_modifier::{KM_LEFT_ALT, KM_LEFT_CONTROL, KM_LEFT_SHIFT, KM_NONE};
    use crate::transform::TransformMap;

    #[test]
    fn test_map() {
        let enter_key = VirtualKey::by_name("VK_RETURN").unwrap();
        let a_key = VirtualKey::by_name("VK_A").unwrap();
        let b_key = VirtualKey::by_name("VK_B").unwrap();

        let mut map = TransformMap::new();
        let source = KeyAction {
            key: KeyIdentifier::from_virtual_key(enter_key),
            transition: KeyTransition::Up,
            modifiers: KM_LEFT_SHIFT | KM_LEFT_CONTROL,
        };

        let target = KeyActionSequence::from(vec![
            KeyAction {
                key: KeyIdentifier::from_virtual_key(a_key),
                transition: KeyTransition::Up,
                modifiers: KM_LEFT_ALT,
            },
            KeyAction {
                key: KeyIdentifier::from_virtual_key(b_key),
                transition: KeyTransition::Up,
                modifiers: KM_LEFT_CONTROL,
            },
        ]);
        // let expected = target.clone();

        map.put(source, target);

        // dbg!(map.virtual_key_map.map.keys());
        let actual = map.get(&source);
        //
        dbg!(&actual);
        // assert_eq!(actual, &expected);
    }
    #[test]

    fn test_any_modifier() {
        let a_key = VirtualKey::by_name("VK_A").unwrap();
        let b_key = VirtualKey::by_name("VK_B").unwrap();

        let mut map = TransformMap::new();
        let source = KeyAction {
            key: KeyIdentifier::from_virtual_key(a_key),
            transition: KeyTransition::Up,
            modifiers:KM_NONE,
        };

        let target = KeyActionSequence::from(vec![
            KeyAction {
                key: KeyIdentifier::from_virtual_key(b_key),
                transition: KeyTransition::Up,
                modifiers: KM_LEFT_CONTROL,
            },
        ]);

        let expected = target.clone();
        
        map.put(source, target);
        
        assert_eq!(map.get(&source).unwrap(), &expected);

        let source = KeyAction {
            key: KeyIdentifier::from_virtual_key(a_key),
            transition: KeyTransition::Down,
            modifiers:KM_NONE,
        };
        assert!(map.get(&source).is_none());

        // dbg!(map.virtual_key_map.map.keys());
        //
        // dbg!(&actual);
    }
}
