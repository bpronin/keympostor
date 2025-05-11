use crate::key_action::KeyAction;
use crate::key_event::KeyEvent;
use crate::key_modifiers::KeyModifiers;
use crate::key_transform_rule::{KeyTransformProfile, KeyTransformRule};
use std::collections::HashMap;

#[derive(Debug)]
pub struct KeyTransformMap {
    map: HashMap<KeyAction, HashMap<KeyModifiers, KeyTransformRule>>,
}

impl KeyTransformMap {
    pub(crate) fn from_profile(profile: KeyTransformProfile) -> KeyTransformMap {
        let mut this = Self::default();
        for rule in profile.rules {
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

    pub(crate) fn get(
        &self,
        event: &KeyEvent,
        get_modifiers: fn() -> KeyModifiers,
    ) -> Option<&KeyTransformRule> {
        let rules = self.get_group(&event.action());
        rules.and_then(|rules| {
            let modifiers = get_modifiers();
            rules.get(&modifiers)
        })
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
    use crate::key_event::KeyEvent;
    use crate::key_modifiers::{KM_LALT, KM_LCONTROL, KM_LSHIFT, KM_NONE};
    use crate::key_transform_map::KeyAction;
    use crate::key_transform_map::KeyTransformMap;
    use crate::key_transform_rule::KeyTransformRule;
    use crate::{assert_none, key_act, key_event, key_rule};
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

        assert_none!(map.get_group(&key_act!("VK_A^")));
        assert_none!(map.get_group(&key_act!("VK_C^")));

        // let expected = [
        //     key_rule!("VK_A↓ : VK_B↓"),
        //     key_rule!("[SHIFT] VK_A↓ : VK_C↓"),
        //     key_rule!("[CONTROL] VK_A↓ : VK_D↓"),
        //     key_rule!("SC_A↓ : SC_C↓"), /* VK_A converted from SC_A */
        // ];
        // 
        // let contains_all = expected.iter().all(|r| map.get_group(r) == Some(v));
        // 
        // let option = map.get_group(&key_act!("VK_A*")).unwrap().a;
        // assert_eq!(expected, option);
        // 
        // let expected = [key_rule!("[ALT] SC_B↓ : SC_0x1C↓")];
        // /* VK_B converted from SC_B */
        // assert_eq!(expected, map.get_group(&key_act!("VK_B*")));
    }

    #[test]
    fn test_get() {
        let all_up = || KM_NONE;
        let shift_down = || KM_LSHIFT;
        let alt_down = || KM_LALT;
        let ctrl_down = || KM_LCONTROL;
        let ctrl_alt_down = || KM_LCONTROL | KM_LALT;

        let mut map = KeyTransformMap::default();
        map.put(key_rule!("VK_A↓ : VK_B↓"));
        map.put(key_rule!("[LSHIFT] VK_A↓ : VK_C↓"));
        map.put(key_rule!("[LALT + LCONTROL] VK_A↓ : VK_D↓"));

        let group = map.get_group(&key_act!("VK_A*"));
        assert_eq!(3, group.unwrap().len());

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
