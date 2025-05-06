use crate::key::{KeyCode, VirtualKey, MAX_VK_CODE};
use crate::key_action::{KeyAction, KeyTransformRule};
use crate::key_event::{KeyEvent, KeyTransition};
use std::array::from_fn;
use KeyCode::{SC, VK};

#[derive(Debug)]
struct VirtualKeyTransformMap {
    map: [[Vec<KeyTransformRule>; MAX_VK_CODE]; 2],
}

impl VirtualKeyTransformMap {
    fn new() -> Self {
        Self {
            map: [from_fn(|_| vec![]), from_fn(|_| vec![])],
        }
    }

    // fn get(
    //     &self,
    //     key: &VirtualKey,
    //     transition: KeyTransition,
    // ) -> &[KeyTransformRule] {
    //     self.get_all(key, transition).iter().find(|rule| {rule})
    // }

    fn get_all(&self, key: &VirtualKey, transition: KeyTransition) -> &[KeyTransformRule] {
        &self.map[transition.is_up() as usize][key.value as usize]
    }

    fn put(&mut self, key: &VirtualKey, transition: KeyTransition, rule: KeyTransformRule) {
        self.map[transition.is_up() as usize][key.value as usize].push(rule)
    }
}

#[derive(Debug)]
struct KeyTransformMap {
    virtual_key_map: VirtualKeyTransformMap,
}

impl KeyTransformMap {
    pub fn new() -> Self {
        Self {
            virtual_key_map: VirtualKeyTransformMap::new(),
        }
    }

    pub fn get(&self, event: &KeyEvent) -> Option<KeyTransformRule> {
        let mut rules = self.get_all(event.as_virtual_key_action());
        if rules.is_empty() {
            rules = self.get_all(event.as_scan_code_action());
        }
        todo!()
    }

    fn get_all(&self, trigger: KeyAction) -> &[KeyTransformRule] {
        match trigger.key {
            VK(vk) => self.virtual_key_map.get_all(vk, trigger.transition),
            SC(sc) => todo!(),
        }
    }

    fn put(&mut self, rule: KeyTransformRule) {
        let trigger = rule.trigger();
        match trigger.key {
            VK(vk) => self.virtual_key_map.put(vk, trigger.transition, rule),
            SC(sc) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::key_action::{KeyAction, KeyTransformRule};
    use crate::key_event::KeyEvent;
    use crate::key_transform::KeyTransformMap;
    use crate::{key_action, key_rule};
    use std::str::FromStr;
    use windows::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT;

    #[test]
    fn test_all() {
        todo!("Search for all posible KBDLLHOOKSTRUCT's")
    }

    // #[test]
    // fn test_vk_map_put_get_all() {
    //     let mut map = VirtualKeyTransformMap::new();
    //
    //     let source = kt_rule!("VK_A↓ : VK_B↓");
    //     let tr = &source.trigger();
    //     map.put(tr.key.as_virtual_key().unwrap(), tr.transition, source);
    //
    //     let expected = kt_rule!("VK_A↓ : VK_B↓");
    //     let tr = expected.trigger();
    //     let actual = map.get_all(&tr.key.as_virtual_key().unwrap(), tr.transition);
    //
    //     assert_eq!([expected], actual);
    // }

    #[test]
    fn test_get_all() {
        let mut map = KeyTransformMap::new();

        map.put(key_rule!("VK_A↓ : VK_B↓"));
        map.put(key_rule!("VK_A↓ → VK_SHIFT↓ : VK_C↓"));
        map.put(key_rule!("VK_A↓ → VK_CONTROL↓ : VK_D↓"));

        assert!(map.get_all("VK_B↓".parse().unwrap()).is_empty());
        assert!(map.get_all("VK_A↑".parse().unwrap()).is_empty());

        let expected = [
            key_rule!("VK_A↓ : VK_B↓"),
            key_rule!("VK_A↓ → VK_SHIFT↓ : VK_C↓"),
            key_rule!("VK_A↓ → VK_CONTROL↓ : VK_D↓"),
        ];

        assert_eq!(expected, map.get_all(key_action!("VK_A↓")));
    }

    #[test]
    fn test_get() {
        let mut map = KeyTransformMap::new();

        map.put(key_rule!("VK_A↓ : VK_B↓"));
        map.put(key_rule!("VK_A↓ → VK_SHIFT↓ : VK_C↓"));
        map.put(key_rule!("VK_A↓ → VK_CONTROL↓ : VK_D↓"));

        assert!(map.get_all("VK_B↓".parse().unwrap()).is_empty());
        assert!(map.get_all("VK_A↑".parse().unwrap()).is_empty());

        let expected = key_rule!("VK_A↓ → VK_SHIFT↓ : VK_C↓");

        let event = KeyEvent {
            kb: KBDLLHOOKSTRUCT {
                vkCode: 0,
                scanCode: 0,
                flags: Default::default(),
                time: 0,
                dwExtraInfo: 0,
            },
        };

        assert_eq!(expected, map.get(&event).unwrap());
    }
}
