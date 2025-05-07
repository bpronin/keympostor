use crate::key::{KeyCode, ScanCode, VirtualKey, MAX_VK_CODE};
use crate::key_action::{KeyAction, KeyTransition};
use crate::key_event::KeyEvent;
use crate::keyboard_state::KeyboardState;
use crate::transform_rule::KeyTransformRule;
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

    fn get_group(&self, key: &VirtualKey, transition: KeyTransition) -> &[KeyTransformRule] {
        &self.map[transition.is_up() as usize][key.value as usize]
    }

    fn put(&mut self, key: &VirtualKey, transition: KeyTransition, rule: KeyTransformRule) {
        self.map[transition.is_up() as usize][key.value as usize].push(rule)
    }
}

#[derive(Debug)]
struct ScanCodeTransformMap {
    map: [[[Vec<KeyTransformRule>; MAX_VK_CODE]; 2]; 2],
}

impl ScanCodeTransformMap {
    fn new() -> Self {
        Self {
            map: [
                [from_fn(|_| vec![]), from_fn(|_| vec![])],
                [from_fn(|_| vec![]), from_fn(|_| vec![])],
            ],
        }
    }

    fn get_group(&self, key: &ScanCode, transition: KeyTransition) -> &[KeyTransformRule] {
        &self.map[transition.is_up() as usize][key.is_extended as usize][key.value as usize]
    }

    pub(crate) fn put(
        &mut self,
        key: &ScanCode,
        transition: KeyTransition,
        rule: KeyTransformRule,
    ) {
        self.map[transition.is_up() as usize][key.is_extended as usize][key.value as usize]
            .push(rule)
    }
}

#[derive(Debug)]
struct KeyTransformMap {
    virtual_key_map: VirtualKeyTransformMap,
    scan_code_map: ScanCodeTransformMap,
}

impl KeyTransformMap {
    pub fn new() -> Self {
        Self {
            virtual_key_map: VirtualKeyTransformMap::new(),
            scan_code_map: ScanCodeTransformMap::new(),
        }
    }

    pub(crate) fn get_rule(
        &self,
        event: &KeyEvent,
        get_kbd_state: fn() -> KeyboardState,
    ) -> Option<&KeyTransformRule> {
        let mut rules = self.get_group(&event.as_virtual_key_action());
        if rules.is_empty() {
            rules = self.get_group(&event.as_scan_code_action());
        }

        for rule in rules {
            if get_kbd_state().has_state(&rule.modifiers()) {
                return Some(rule);
            }
        }

        None
    }

    // pub fn get(&self, event: &KeyEvent) -> Option<&KeyTransformRule> {
    //     let action = event.as_virtual_key_action();
    //     let mut rules = self.get_all(action);
    //
    //     // dbg!(&rules);
    //
    //     if rules.is_empty() {
    //         rules = self.get_all(event.as_scan_code_action());
    //     }
    //
    //     for rule in rules {
    //         if let Some(modifiers) = rule.modifiers() {
    //             let keyboard_state = KeyboardState::capture();
    //             if keyboard_state.has_state(&modifiers) {
    //                 return Some(rule);
    //             }
    //         } else {
    //             return Some(rule);
    //         }
    //     }
    //
    //     None
    // }

    fn get_group(&self, trigger: &KeyAction) -> &[KeyTransformRule] {
        match trigger.key {
            VK(vk) => self.virtual_key_map.get_group(vk, trigger.transition),
            SC(sc) => self.scan_code_map.get_group(sc, trigger.transition),
        }
    }

    fn put(&mut self, rule: KeyTransformRule) {
        let trigger = rule.trigger();
        match trigger.key {
            VK(vk) => self.virtual_key_map.put(vk, trigger.transition, rule),
            SC(sc) => self.scan_code_map.put(sc, trigger.transition, rule),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::key::MAX_VK_CODE;
    use crate::key_action::KeyAction;
    use crate::key_event::KeyEvent;
    use crate::key_transform_map::KeyTransformMap;
    use crate::keyboard_state::{KeyboardState, DOWN_STATE, UP_STATE};
    use crate::transform_rule::KeyTransformRule;
    use crate::{key_action, key_event, key_rule};
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_A, VK_B, VK_CONTROL, VK_MENU, VK_SHIFT};
    use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, LLKHF_UP};

    #[test]
    fn test_all() {
        // todo!("Search for all posible KBDLLHOOKSTRUCT's")
    }

    #[test]
    fn test_get_group() {
        let mut map = KeyTransformMap::new();

        map.put(key_rule!("VK_A↓ : VK_B↓"));
        map.put(key_rule!("VK_A↓ → VK_SHIFT↓ : VK_C↓"));
        map.put(key_rule!("VK_A↓ → VK_CONTROL↓ : VK_D↓"));
        map.put(key_rule!("SC_A↓ : SC_C↓"));
        map.put(key_rule!("SC_B↓ → SC_ALT↓ : SC_0x1C↓"));

        assert!(map.get_group(&key_action!("VK_B↓")).is_empty());
        assert!(map.get_group(&key_action!("VK_A↑")).is_empty());

        let expected = [
            key_rule!("VK_A↓ : VK_B↓"),
            key_rule!("VK_A↓ → VK_SHIFT↓ : VK_C↓"),
            key_rule!("VK_A↓ → VK_CONTROL↓ : VK_D↓"),
        ];

        assert_eq!(expected, map.get_group(&key_action!("VK_A↓")));
    }

    #[test]
    fn test_get_rule() {
        let mut map = KeyTransformMap::new();
        map.put(key_rule!("VK_A↓ : VK_B↓"));
        map.put(key_rule!("VK_A↓ → VK_SHIFT↓ : VK_C↓"));
        map.put(key_rule!("VK_A↓ → VK_MENU↓ → VK_CONTROL↓ : VK_D↓"));

        let all_up = || KeyboardState::new([UP_STATE; MAX_VK_CODE]);

        let shift_down = || {
            let mut keys = [UP_STATE; MAX_VK_CODE];
            keys[VK_SHIFT.0 as usize] = DOWN_STATE;
            KeyboardState::new(keys)
        };

        let alt_down = || {
            let mut keys = [UP_STATE; MAX_VK_CODE];
            keys[VK_MENU.0 as usize] = DOWN_STATE;
            KeyboardState::new(keys)
        };

        let ctrl_down = || {
            let mut keys = [UP_STATE; MAX_VK_CODE];
            keys[VK_CONTROL.0 as usize] = DOWN_STATE;
            KeyboardState::new(keys)
        };

        let ctrl_alt_down = || {
            let mut keys = [UP_STATE; MAX_VK_CODE];
            keys[VK_CONTROL.0 as usize] = DOWN_STATE;
            keys[VK_MENU.0 as usize] = DOWN_STATE;
            KeyboardState::new(keys)
        };

        assert_eq!(None, map.get_rule(&key_event!(VK_B.0, false), all_up));
        assert_eq!(None, map.get_rule(&key_event!(VK_MENU.0, false), alt_down));
        assert_eq!(
            None,
            map.get_rule(&key_event!(VK_SHIFT.0, false), shift_down)
        );
        assert_eq!(None, map.get_rule(&key_event!(VK_A.0, false), ctrl_down));
        assert_eq!(None, map.get_rule(&key_event!(VK_A.0, false), alt_down));

        assert_eq!(
            &key_rule!("VK_A↓ : VK_B↓"),
            map.get_rule(&key_event!(VK_A.0, false), all_up).unwrap()
        );
        assert_ne!(
            &key_rule!("VK_A↓ : VK_C↓"),
            map.get_rule(&key_event!(VK_A.0, false), shift_down)
                .unwrap()
        );
        assert_ne!(
            &key_rule!("VK_A↓ : VK_C↓"),
            map.get_rule(&key_event!(VK_A.0, false), ctrl_alt_down)
                .unwrap()
        );

        assert_eq!(
            &key_rule!("VK_A↓ → VK_SHIFT↓ : VK_C↓"),
            map.get_rule(&key_event!(VK_A.0, false), shift_down)
                .unwrap()
        );
        assert_ne!(
            &key_rule!("VK_A↓ → VK_SHIFT↓ : VK_C↓"),
            map.get_rule(&key_event!(VK_A.0, false), all_up).unwrap()
        );
        assert_ne!(
            &key_rule!("VK_A↓ → VK_SHIFT↓ : VK_C↓"),
            map.get_rule(&key_event!(VK_A.0, false), ctrl_alt_down)
                .unwrap()
        );

        assert_eq!(
            &key_rule!("VK_A↓ → VK_MENU↓ → VK_CONTROL↓ : VK_D↓"),
            map.get_rule(&key_event!(VK_A.0, false), ctrl_alt_down)
                .unwrap()
        );
        assert_ne!(
            &key_rule!("VK_A↓ → VK_MENU↓ → VK_CONTROL↓ : VK_D↓"),
            map.get_rule(&key_event!(VK_A.0, false), all_up).unwrap()
        );
    }
}
