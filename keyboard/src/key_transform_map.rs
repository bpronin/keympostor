use crate::key::{KeyCode, MAX_VK_CODE, VirtualKey};
use crate::key_action::{KeyAction, KeyTransition};
use crate::key_event::KeyEvent;
use crate::keyboard_state::KeyboardState;
use crate::transform_rule::KeyTransformRule;
use KeyCode::{SC, VK};
use std::array::from_fn;

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

    pub(crate) fn get_with_state(
        &self,
        event: &KeyEvent,
        get_kbd_state: fn() -> KeyboardState,
    ) -> Option<&KeyTransformRule> {
        let mut rules = self.get_all(event.as_virtual_key_action());
        if rules.is_empty() {
            rules = self.get_all(event.as_scan_code_action());
        }
        // dbg!(&rules);

        for rule in rules {
            let modifiers = rule.modifiers();
            let state = get_kbd_state();
            dbg!(&modifiers);
            let has_state = state.has_state(&modifiers);
            dbg!(has_state);
            if has_state {
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
    use crate::key::MAX_VK_CODE;
    use crate::key_action::KeyAction;
    use crate::key_event::KeyEvent;
    use crate::key_transform_map::KeyTransformMap;
    use crate::keyboard_state::{DOWN_STATE, KeyboardState, UP_STATE};
    use crate::transform_rule::KeyTransformRule;
    use crate::{key_action, key_rule};
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_A, VK_SHIFT};
    use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, LLKHF_UP};

    #[test]
    fn test_all() {
        let set = 0x08 & 0x80 != 0;

        println!("{:08b}", 0x80);
        // dbg!(&set);
        // todo!("Search for all posible KBDLLHOOKSTRUCT's")
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

    macro_rules! key_event {
        ($vk_code:expr, $is_up:expr) => {
            KeyEvent {
                kb: KBDLLHOOKSTRUCT {
                    vkCode: $vk_code as u32,
                    flags: if $is_up { LLKHF_UP } else { Default::default() },
                    ..Default::default()
                },
            }
        };
    }

    #[test]
    fn test_get() {
        let mut map = KeyTransformMap::new();

        map.put(key_rule!("VK_A↓ : VK_B↓"));
        map.put(key_rule!("VK_A↓ → VK_SHIFT↓ : VK_C↓"));
        // map.put(key_rule!("VK_A↓ → VK_MENU↓ → VK_CONTROL↓ : VK_D↓"));

        // let option = map.get(&key_event!(VK_A.0, true));
        // dbg!(&option);
        // assert_eq!(None, option);
        // // assert_eq!(None, map.get(&key_event!(VK_B.0, false)));

        // let get_kbd_state = || KeyboardState::new([UP_STATE; MAX_VK_CODE]);
        // assert_eq!(
        //     &key_rule!("VK_A↓ : VK_B↓"),
        //     map.get_with_state(&key_event!(VK_A.0, false), get_kbd_state)
        //         .unwrap()
        // );

        let get_kbd_state = || {
            let mut keys = [UP_STATE; MAX_VK_CODE];
            keys[VK_SHIFT.0 as usize] = DOWN_STATE;
            KeyboardState::new(keys)
        };

        assert_eq!(
            &key_rule!("VK_A↓ → VK_SHIFT↓ : VK_C↓"),
            map.get_with_state(&key_event!(VK_A.0, false), get_kbd_state)
                .unwrap()
        );
    }
}
