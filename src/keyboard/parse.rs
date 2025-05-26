use crate::keyboard::key::{Key, ScanCode, VirtualKey};
use crate::keyboard::key_action::KeyTransition::{Down, Up};
use crate::keyboard::key_action::{KeyAction, KeyActionSequence, KeyTransition};
use crate::keyboard::key_const::KEYS;
use crate::keyboard::key_modifiers::KeyboardState::{All, Any};
use crate::keyboard::key_modifiers::{
    KeyModifiers, KeyboardState, KM_LALT, KM_LCTRL, KM_LSHIFT, KM_LWIN, KM_NONE, KM_RALT, KM_RCTRL,
    KM_RSHIFT, KM_RWIN,
};
use crate::keyboard::key_trigger::KeyTrigger;
use crate::keyboard::transform_rules::{KeyTransformProfile, KeyTransformRule, KeyTransformRules};
use std::str::{FromStr, Lines};

impl FromStr for VirtualKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let st = s.trim();
        Self::from_code_name(st)
            .or_else(|_| Self::from_name(st))
            .copied()
    }
}

impl FromStr for ScanCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let st = s.trim();
        Self::from_code_name(st)
            .or_else(|_| Self::from_name(st))
            .copied()
    }
}

impl FromStr for Key {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        KEYS.with(|keys| {
            let key = keys
                .by_name(s.trim())
                .ok_or(format!("Illegal key name: `{}`.", s))?;
            Ok(*key)
        })
    }
}

impl FromStr for KeyTransition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.trim().chars();
        let symbol = chars.next().ok_or("Key transition symbol is empty.")?;
        if chars.next().is_none() {
            match symbol {
                '↑' | '^' => Ok(Up),
                '↓' | '*' => Ok(Down),
                _ => Err(format!("Illegal key transition symbol `{}`.", s)),
            }
        } else {
            Err(format!("Key transition symbols `{}` is too long.", s))
        }
    }
}

impl FromStr for KeyAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let st = s.trim();

        let suffix = st
            .chars()
            .last()
            .ok_or(&format!("Error parsing key action. String is empty. `{s}`"))?;

        let prefix = st
            .strip_suffix(suffix)
            .ok_or(&format!("Invalid key action suffix: `{suffix}`."))?;

        let action = Self {
            key: prefix.parse()?,
            transition: suffix.to_string().parse()?,
        };

        Ok(action)
    }
}

/*todo!
impl KeyActionSequence {
    fn from_str_group(s: &str) -> Result<Vec<Self>, String> {
        let mut list = vec![];

        Ok(list)
    }
}
 */

impl FromStr for KeyActionSequence {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let actions = s
            .split(|ch| ['→', '>'].contains(&ch))
            .flat_map(|part| {
                let part = part.trim();

                let (prefix, suffixes) = part
                    .char_indices()
                    .find(|(_, ch)| ['↑', '↓', '^', '*'].contains(ch))
                    .map(|(ix, _)| part.split_at(ix))
                    .unwrap_or((part, "↓↑"));

                suffixes.chars().map(move |suffix| {
                    Ok(KeyAction {
                        key: prefix.parse()?,
                        transition: suffix.to_string().parse()?,
                    })
                })
            })
            .collect::<Result<Vec<_>, Self::Err>>()?;

        Ok(Self::new(actions))
    }
}

impl FromStr for KeyboardState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "*" => Any,
            &_ => All(KeyModifiers::from_str(s)?),
        })
    }
}

impl FromStr for KeyModifiers {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ts = s.trim();
        if ts.is_empty() {
            Ok(KM_NONE)
        } else {
            let this = ts.split('+').fold(KM_NONE, |acc, part| match part.trim() {
                "LEFT_SHIFT" => acc | KM_LSHIFT,
                "RIGHT_SHIFT" => acc | KM_RSHIFT,
                // "SHIFT" => acc | KM_LSHIFT | KM_RSHIFT,
                "LEFT_CTRL" => acc | KM_LCTRL,
                "RIGHT_CTRL" => acc | KM_RCTRL,
                // "CTRL" => acc | KM_LCTRL | KM_RCTRL,
                "LEFT_ALT" => acc | KM_LALT,
                "RIGHT_ALT" => acc | KM_RALT,
                // "ALT" => acc | KM_LALT | KM_RALT,
                "LEFT_WIN" => acc | KM_LWIN,
                "RIGHT_WIN" => acc | KM_RWIN,
                // "WIN" => acc | KM_LWIN | KM_RWIN,
                &_ => KM_NONE,
            });

            if this != KM_NONE {
                Ok(this)
            } else {
                Err(format!("Error parsing key modifiers: `{s}`"))
            }
        }
    }
}

impl KeyTrigger {
    fn parse_group(s: &str) -> impl Iterator<Item = Result<Self, String>> + '_ {
        
        s.split('+').map(str::parse)
    }

    fn parse_list(s: &str) -> Result<Vec<Self>, String> {
        s.split(',').flat_map(Self::parse_group).collect()
    }
}

impl FromStr for KeyTrigger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.trim().strip_prefix('[') {
            let mut parts = s.split(']');
            Ok(Self {
                state: parts.next().unwrap().parse()?, /* Modifiers go first! */
                action: parts.next().unwrap().parse()?,
            })
        } else {
            Ok(Self {
                action: s.parse()?,
                state: Any,
            })
        }
    }
}

impl KeyTransformRule {
    fn parse_list(s: &str) -> Result<Vec<Self>, String> {
        let mut parts = s.trim().split(":");

        let triggers = KeyTrigger::parse_list(parts.next().ok_or("Missing source part.")?)?;
        let actions = KeyActionSequence::from_str(parts.next().ok_or("Missing target part.")?)?;

        let mut rules = vec![];
        for trigger in triggers {
            rules.push(KeyTransformRule {
                source: trigger,
                target: actions.clone(),
            })
        }

        Ok(rules)
    }
}

impl FromStr for KeyTransformRule {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.trim().split(":");
        Ok(Self {
            source: parts.next().ok_or("Missing source part.")?.parse()?,
            target: parts.next().ok_or("Missing target part.")?.parse()?,
        })
    }
}

impl KeyTransformRules {
    fn from_lines(lines: Lines) -> Result<Self, String> {
        let mut items = vec![];
        for line in lines {
            let rules = KeyTransformRule::parse_list(line.trim())?;
            items.extend(rules);
        }

        Ok(Self { items })
    }
}

impl FromStr for KeyTransformRules {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_lines(s.trim().lines())
    }
}

impl FromStr for KeyTransformProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.trim().lines();

        Ok(Self {
            title: lines.next().ok_or("Error parsing title.")?.trim().into(),
            rules: KeyTransformRules::from_lines(lines)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key::{Key, ScanCode, VirtualKey};
    use crate::keyboard::key_action::KeyTransition::{Down, Up};
    use crate::keyboard::key_action::{KeyAction, KeyTransition};
    use crate::keyboard::key_modifiers::KeyboardState::{All, Any};
    use crate::keyboard::key_modifiers::{
        KeyModifiers, KeyboardState, KM_LSHIFT, KM_NONE, KM_RSHIFT, KM_RWIN,
    };
    use crate::keyboard::key_trigger::KeyTrigger;
    use crate::keyboard::parse::KeyActionSequence;
    use crate::keyboard::transform_rules::{
        KeyTransformProfile, KeyTransformRule, KeyTransformRules,
    };
    use crate::{
        key, key_action, key_action_seq, key_mod, key_profile, key_rule, key_rules, key_trigger,
    };
    use std::str::FromStr;

    // Virtual key

    #[test]
    fn test_vk_from_str() {
        assert_eq!("VK_RETURN", VirtualKey::from_str("VK_RETURN").unwrap().name);
        assert_eq!("VK_RETURN", VirtualKey::from_str("RETURN").unwrap().name);
        assert_eq!("VK_RETURN", VirtualKey::from_str("VK_0x0D").unwrap().name);
    }

    #[test]
    fn test_vk_from_str_fails() {
        assert!(VirtualKey::from_str("BANANA").is_err());
    }

    // Scancode

    #[test]
    fn test_sc_from_str() {
        assert_eq!("SC_ENTER", ScanCode::from_str("SC_ENTER").unwrap().name);
        assert_eq!("SC_ENTER", ScanCode::from_str("ENTER").unwrap().name);
        assert_eq!(
            "SC_NUM_ENTER",
            ScanCode::from_str("SC_0xE01C").unwrap().name
        );
    }

    #[test]
    fn test_sc_from_str_fails() {
        assert!(ScanCode::from_str("BANANA").is_err());
    }

    // Key

    #[test]
    fn test_key_from_str() {
        assert_eq!(
            Key {
                vk_code: 0x0D,
                scan_code: 0x1C,
                is_ext_scan_code: false,
            },
            Key::from_str("ENTER").unwrap()
        );

        assert_eq!(
            Key {
                vk_code: 0x0D,
                scan_code: 0x1C,
                is_ext_scan_code: true,
            },
            Key::from_str("NUM_ENTER").unwrap()
        );

        assert_eq!(
            Key {
                vk_code: 0x72,
                scan_code: 0x3D,
                is_ext_scan_code: false,
            },
            Key::from_str("F3").unwrap()
        );
    }

    #[test]
    fn test_key_from_str_fails() {
        assert!(Key::from_str("BANANA").is_err());
    }

    // Key transition

    #[test]
    fn test_key_transition_from_str() {
        assert_eq!(Down, KeyTransition::from_str("↓").unwrap());
        assert_eq!(Up, KeyTransition::from_str("↑").unwrap());
        assert_eq!(Down, KeyTransition::from_str("*").unwrap());
        assert_eq!(Up, KeyTransition::from_str("^").unwrap());
    }

    #[test]
    fn test_key_transition_from_str_fails_illegal() {
        assert!(KeyTransition::from_str("BANANA").is_err())
    }

    #[test]
    fn test_key_transition_from_str_fails_empty() {
        assert!(KeyTransition::from_str("").is_err())
    }

    #[test]
    fn test_key_transition_from_str_fails_to_long() {
        assert!(KeyTransition::from_str("↑↑↑").is_err())
    }

    // Key modifiers

    #[test]
    fn test_key_modifiers_from_str() {
        assert_eq!(KM_NONE, KeyModifiers::from_str("").unwrap());

        assert_eq!(
            KM_LSHIFT | KM_RSHIFT | KM_RWIN,
            KeyModifiers::from_str("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN").unwrap()
        );
    }

    #[test]
    fn test_key_modifiers_from_str_fails() {
        assert!(KeyModifiers::from_str("BANANA").is_err());
    }

    // Keyboard state

    #[test]
    fn test_keyboard_state_all_from_str() {
        assert_eq!(
            All(KM_LSHIFT | KM_RSHIFT | KM_RWIN),
            KeyboardState::from_str("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN").unwrap()
        );

        assert_eq!(All(KM_NONE), KeyboardState::from_str("").unwrap());
    }

    #[test]
    fn test_keyboard_state_any_from_str() {
        assert_eq!(Any, KeyboardState::from_str("*").unwrap());
    }

    // Key Action

    #[test]
    fn test_key_action_from_str() {
        let expected = KeyAction {
            key: key!("ENTER"),
            transition: Down,
        };
        assert_eq!(expected, KeyAction::from_str("ENTER↓").unwrap());

        let expected = KeyAction {
            key: key!("F3"),
            transition: Down,
        };
        assert_eq!(expected, KeyAction::from_str("    F3\n*").unwrap());
    }

    // Key action sequence

    /*todo!
        #[test]
        fn test_key_action_sequence_from_str_no_transiion() {
            let actual = KeyActionSequence::from_str_group("A").unwrap();

            assert_eq!(key_action_seq!("A↓"), actual[0]);
            assert_eq!(key_action_seq!("A↑"), actual[1]);
        }
    */

    #[test]
    fn test_key_action_sequence_from_str_up_down_transition() {
        assert_eq!(
            key_action_seq!("A↓ → A↑"),
            KeyActionSequence::from_str("A↓↑").unwrap()
        );
    }

    // Key trigger

    #[test]
    fn test_key_trigger_from_str_all_modifiers() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                state: All(key_mod!("LEFT_SHIFT")),
            },
            KeyTrigger::from_str("[LEFT_SHIFT] A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_parse_no_modifiers() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                state: All(KM_NONE),
            },
            KeyTrigger::from_str("[] A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_parse_any_modifiers() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                state: Any,
            },
            KeyTrigger::from_str("[*]A*").unwrap()
        );

        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                state: Any,
            },
            KeyTrigger::from_str("A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_list() {
        let expected = vec![
            key_trigger!("A*"),
            key_trigger!("[LEFT_CTRL]B^"),
            key_trigger!("C*"),
        ];
        let actual = KeyTrigger::parse_list("A*, [LEFT_CTRL]B^, C*").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_trigger_from_str_no_transition() {
        let expected = vec![key_trigger!("[LEFT_CTRL]A↓"), key_trigger!("[LEFT_CTRL]A↑")];
        let actual = KeyTrigger::parse_list("[LEFT_CTRL]A").unwrap();
        assert_eq!(expected, actual);
    }

    // Transform rule

    #[test]
    fn test_key_transform_rule_from_str() {
        let expected = KeyTransformRule {
            source: key_trigger!("[LEFT_SHIFT] ENTER↓"),
            target: key_action_seq!("A↓"),
        };

        assert_eq!(
            expected,
            KeyTransformRule::from_str("[LEFT_SHIFT] ENTER↓ : A↓").unwrap()
        );
    }

    #[test]
    fn test_key_transform_rule_from_str_group() {
        let expected = vec![
            key_rule!("A* : ENTER*"),
            key_rule!("[LEFT_CTRL]B* : ENTER*"),
            key_rule!("C^ : ENTER*"),
        ];
        let actual = KeyTransformRule::parse_list("A*, [LEFT_CTRL]B*, C^ : ENTER*").unwrap();
        assert_eq!(expected, actual);
    }

    // Transform rules

    #[test]
    fn test_key_transform_rules_from_str_up_down_transition() {
        let actual = key_rules!(
            "
            A↓ : A↓↑ → B↓↑
            "
        );
        let expected = key_rules!(
            "
            A↓ : A↓ → A↑ → B↓ → B↑ 
            "
        );

        assert_eq!(expected, actual);
    }

    /*todo!
    #[test]
    fn test_key_transform_rules_from_str_no_transition() {
        let actual = key_rules!(
            "
        A : B
        "
        );
        let expected = key_rules!(
            "
        A↓ : B↓
        A↑ : B↑
        "
        );

        assert_eq!(expected, actual);
    }
    */

    #[test]
    fn test_key_transform_rules_from_str_group() {
        let actual = key_rules!(
            "
            A↓, B↓ : C↓
            "
        );

        let expected = key_rules!(
            "
            A↓ : C↓
            B↓ : C↓
            "
        );

        assert_eq!(expected, actual);
    }

    // Transform profile

    #[test]
    fn test_key_transform_profile_from_str() {
        let actual = key_profile!(
            "
        Test profile
        A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
        [LEFT_CTRL + LEFT_SHIFT] ENTER↓ : ENTER↓ → ENTER↑
        "
        );

        let expected = KeyTransformProfile {
            title: "Test profile".to_string(),
            rules: KeyTransformRules {
                items: vec![
                    key_rule!("A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"),
                    key_rule!("[LEFT_CTRL + LEFT_SHIFT] ENTER↓: ENTER↓ → ENTER↑"),
                ],
            },
        };

        assert_eq!(expected, actual);
    }
}
