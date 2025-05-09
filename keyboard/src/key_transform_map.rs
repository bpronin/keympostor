use crate::key::{KeyCode, ScanCode, VirtualKey, MAX_VK_CODE};
use crate::key_action::KeyTransition;
use crate::key_event::KeyEvent;
use crate::key_transform_rule::{KeyTransformProfile, KeyTransformRule};
use std::array::from_fn;
use KeyCode::{SC, VK};
use crate::key_modifiers::KeyModifiers;

#[derive(Debug)]
struct VirtualKeyTransformMap {
    map: [[Vec<KeyTransformRule>; MAX_VK_CODE]; 2],
}

impl VirtualKeyTransformMap {
    fn get_group(&self, key: &VirtualKey, transition: KeyTransition) -> &[KeyTransformRule] {
        &self.map[transition.is_up() as usize][key.value as usize]
    }

    fn put(&mut self, key: &VirtualKey, transition: KeyTransition, rule: KeyTransformRule) {
        self.map[transition.is_up() as usize][key.value as usize].push(rule)
    }
}

impl Default for VirtualKeyTransformMap {
    fn default() -> Self {
        Self {
            map: [from_fn(|_| vec![]), from_fn(|_| vec![])],
        }
    }
}

#[derive(Debug)]
struct ScanCodeTransformMap {
    map: [[[Vec<KeyTransformRule>; MAX_VK_CODE]; 2]; 2],
}

impl ScanCodeTransformMap {
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

impl Default for ScanCodeTransformMap {
    fn default() -> Self {
        Self {
            map: [
                [from_fn(|_| vec![]), from_fn(|_| vec![])],
                [from_fn(|_| vec![]), from_fn(|_| vec![])],
            ],
        }
    }
}

#[derive(Debug, Default)]
pub struct KeyTransformMap {
    virtual_key_map: VirtualKeyTransformMap,
    scan_code_map: ScanCodeTransformMap,
}

impl KeyTransformMap {
    pub(crate) fn from_profile(profile: KeyTransformProfile) -> KeyTransformMap {
        let mut this = Self::default();
        for rule in profile.rules {
            this.put(rule)
        }
        this
    }

    fn get_group(&self, key: &KeyCode, transition: KeyTransition) -> &[KeyTransformRule] {
        match key {
            VK(vk) => self.virtual_key_map.get_group(vk, transition),
            SC(sc) => self.scan_code_map.get_group(sc, transition),
        }
    }

    pub(crate) fn get(
        &self,
        event: &KeyEvent,
        get_modifiers: fn() -> KeyModifiers,
    ) -> Option<&KeyTransformRule> {
        let mut rules = self.get_group(&VK(event.virtual_key()), event.transition());
        if rules.is_empty() {
            rules = self.get_group(&SC(event.scan_code()), event.transition());
        }

        //todo make it HashMap
        for rule in rules {
            if rule.source.modifiers == get_modifiers() {
                return Some(rule);
            }
        }

        None
    }

    fn put(&mut self, rule: KeyTransformRule) {
        let trigger = &rule.source;
        match trigger.key() {
            VK(vk) => self.virtual_key_map.put(vk, trigger.transition(), rule),
            SC(sc) => self.scan_code_map.put(sc, trigger.transition(), rule),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::key_action::KeyTransition::{Down, Up};
    use crate::key_event::KeyEvent;
    use crate::key_modifiers::{KM_LALT, KM_LCTRL, KM_LSHIFT, KM_NONE};
    use crate::key_transform_map::KeyCode;
    use crate::key_transform_map::KeyTransformMap;
    use crate::key_transform_rule::KeyTransformRule;
    use crate::{assert_none, key, key_event, key_rule};
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_A, VK_B, VK_MENU, VK_SHIFT};
    use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, LLKHF_UP};

    #[test]
    fn test_get_group() {
        let mut map = KeyTransformMap::default();

        map.put(key_rule!("VK_A↓ : VK_B↓"));
        map.put(key_rule!("[SHIFT] VK_A↓ : VK_C↓"));
        map.put(key_rule!("[CONTROL] VK_A↓ : VK_D↓"));

        map.put(key_rule!("SC_A↓ : SC_C↓"));
        map.put(key_rule!("[ALT] SC_B↓ : SC_0x1C↓"));

        assert!(map.get_group(&key!("VK_A"), Up).is_empty());
        assert!(map.get_group(&key!("VK_B"), Down).is_empty());

        let expected = [
            key_rule!("VK_A↓ : VK_B↓"),
            key_rule!("[SHIFT] VK_A↓ : VK_C↓"),
            key_rule!("[CONTROL] VK_A↓ : VK_D↓"),
        ];

        assert_eq!(expected, map.get_group(&key!("VK_A"), Down));
    }

    #[test]
    fn test_get() {
        let all_up = || KM_NONE;
        let shift_down = || KM_LSHIFT;
        let alt_down = || KM_LALT;
        let ctrl_down = || KM_LCTRL;
        let ctrl_alt_down = || KM_LCTRL | KM_LALT;

        let mut map = KeyTransformMap::default();
        map.put(key_rule!("VK_A↓ : VK_B↓"));
        map.put(key_rule!("[LSHIFT] VK_A↓ : VK_C↓"));
        map.put(key_rule!("[LALT + LCONTROL] VK_A↓ : VK_D↓"));

        let group = map.get_group(&key!("VK_A"), Down);
        assert_eq!(3, group.len());

        assert_none!(map.get(&key_event!(VK_B.0, false), all_up));
        assert_none!(map.get(&key_event!(VK_A.0, true), all_up));
        assert_none!(map.get(&key_event!(VK_MENU.0, false), alt_down));
        assert_none!(map.get(&key_event!(VK_SHIFT.0, false), shift_down));
        assert_none!(map.get(&key_event!(VK_A.0, false), ctrl_down));

        assert_eq!(
            &key_rule!("VK_A↓ : VK_B↓"),
            map.get(&key_event!(VK_A.0, false), all_up).unwrap()
        );

        assert_eq!(
            &key_rule!("[LSHIFT]VK_A↓ : VK_C↓"),
            map.get(&key_event!(VK_A.0, false), shift_down).unwrap()
        );

        assert_eq!(
            &key_rule!("[LALT + LCONTROL]VK_A↓ : VK_D↓"),
            map.get(&key_event!(VK_A.0, false), ctrl_alt_down).unwrap()
        );
    }
}
