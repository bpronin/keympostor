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
use test_helpers::before_all;

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
        KEYS.with_borrow(|keys| {
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

        Ok(Self { actions })
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
    fn from_str_group(s: &str) -> Result<Vec<Self>, String> {
        let list = s
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<Vec<_>, String>>()?;
        Ok(list)
    }
}

impl FromStr for KeyTrigger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix('[') {
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
    fn from_str_group(s: &str) -> Result<Vec<Self>, String> {
        let mut parts = s.trim().split(":");
        let triggers = KeyTrigger::from_str_group(parts.next().ok_or("Missing source part.")?)?;
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
            let rules = KeyTransformRule::from_str_group(line.trim())?;
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

#[before_all]
#[cfg(test)]
mod tests {
    use crate::keyboard::key::{Key, ScanCode, VirtualKey};
    use crate::keyboard::key_action::KeyTransition::{Down, Up};
    use crate::keyboard::key_action::{KeyAction, KeyTransition};
    use crate::keyboard::key_modifiers::KeyboardState::{All, Any};
    use crate::keyboard::key_modifiers::{
        KeyModifiers, KeyModifiersMatrix, KeyboardState, KM_ALL, KM_LALT, KM_LSHIFT, KM_NONE, KM_RCTRL,
        KM_RSHIFT, KM_RWIN,
    };
    use crate::keyboard::key_trigger::KeyTrigger;
    use crate::keyboard::parse::KeyActionSequence;
    use crate::keyboard::tests::setup_logger;
    use crate::keyboard::transform_rules::{
        KeyTransformProfile, KeyTransformRule, KeyTransformRules,
    };
    use crate::{key, key_action, key_action_seq, key_mod, key_profile, key_rule, key_trigger};
    use std::str::FromStr;
    fn before_all() {
        setup_logger();
    }

    #[test]
    fn test_vk_parse() {
        assert_eq!("VK_RETURN", VirtualKey::from_str("VK_RETURN").unwrap().name);
        assert_eq!("VK_RETURN", VirtualKey::from_str("RETURN").unwrap().name);
        assert_eq!("VK_RETURN", VirtualKey::from_str("VK_0x0D").unwrap().name);
    }

    #[test]
    fn test_vk_parse_fails() {
        assert!(VirtualKey::from_str("BANANA").is_err());
    }

    #[test]
    fn test_sc_parse() {
        assert_eq!("SC_ENTER", ScanCode::from_str("SC_ENTER").unwrap().name);
        assert_eq!("SC_ENTER", ScanCode::from_str("ENTER").unwrap().name);
        assert_eq!(
            "SC_NUM_ENTER",
            ScanCode::from_str("SC_0xE01C").unwrap().name
        );
    }

    #[test]
    fn test_sc_parse_fails() {
        assert!(ScanCode::from_str("BANANA").is_err());
    }

    #[test]
    fn test_key_parse() {
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
    fn test_key_parse_fails() {
        assert!(Key::from_str("BANANA").is_err());
    }

    #[test]
    fn test_key_transition_parse() {
        assert_eq!(Down, "↓".parse().unwrap());
        assert_eq!(Up, "↑".parse().unwrap());
        assert_eq!(Down, "*".parse().unwrap());
        assert_eq!(Up, "^".parse().unwrap());
    }

    #[test]
    fn test_key_transition_parse_fails_illegal() {
        assert!(KeyTransition::from_str("BANANA").is_err())
    }

    #[test]
    fn test_key_transition_parse_fails_empty() {
        assert!(KeyTransition::from_str("").is_err())
    }

    #[test]
    fn test_key_transition_parse_fails_to_long() {
        assert!(KeyTransition::from_str("↑↑↑").is_err())
    }

    #[test]
    fn test_key_action_parse() {
        let expected = KeyAction {
            key: key!("ENTER"),
            transition: Down,
        };
        assert_eq!(expected, "ENTER↓".parse().unwrap());

        let expected = KeyAction {
            key: key!("F3"),
            transition: Down,
        };
        assert_eq!(expected, " F3\n*".parse().unwrap());
    }

    #[test]
    fn test_key_action_sequence_parse_expand_transition() {
        let expected = key_action_seq!("A↓ → A↑");

        assert_eq!(expected, "A↓↑".parse().unwrap());
        assert_eq!(expected, "A".parse().unwrap());
    }

    #[test]
    fn test_key_modifiers_parse() {
        assert_eq!(KM_NONE, "".parse().unwrap());

        assert_eq!(
            KM_LSHIFT | KM_RSHIFT | KM_RWIN,
            "LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN".parse().unwrap()
        );

        // assert_eq!(
        //     KM_LSHIFT | KM_RSHIFT | KM_LWIN | KM_RWIN | KM_LALT | KM_RALT | KM_LCTRL | KM_RCTRL,
        //     "SHIFT + WIN + ALT + CTRL".parse().unwrap()
        // );
    }

    #[test]
    fn test_key_modifiers_parse_fails() {
        assert!(KeyModifiers::from_str("BANANA").is_err());
    }

    #[test]
    fn test_keyboard_state_all_parse() {
        assert_eq!(
            All(KM_LSHIFT | KM_RSHIFT | KM_RWIN),
            KeyboardState::from_str("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN").unwrap()
        );
        
        assert_eq!(All(KM_NONE), KeyboardState::from_str("").unwrap());
    }

    #[test]
    fn test_keyboard_state_any_parse() {
        assert_eq!(Any, KeyboardState::from_str("*").unwrap());
    }

    // #[test]
    // fn test_key_modifiers_matrix_parse_empty() {
    //     let expected = KeyModifiersMatrix::new(&[
    //         KM_ALL, KM_ALL, KM_ALL, KM_ALL, KM_ALL, KM_ALL, KM_ALL, KM_ALL,
    //     ]);
    //
    //     assert_eq!(expected, KeyModifiersMatrix::from_str("").unwrap());
    // }

    #[test]
    fn test_key_trigger_parse_modifiers() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                state: All(key_mod!("LEFT_SHIFT")),
            },
            KeyTrigger::from_str("[LEFT_SHIFT] A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_parse_none() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                state: All(KM_NONE),
            },
            KeyTrigger::from_str("[] A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_parse_any() {
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
    fn test_key_trigger_parse_group() {
        let expected = vec![
            key_trigger!("A*"),
            key_trigger!("[LEFT_CTRL]B^"),
            key_trigger!("C*"),
        ];
        let actual = KeyTrigger::from_str_group("A*, [LEFT_CTRL]B^, C*").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_transform_rule_parse() {
        let expected = KeyTransformRule {
            source: key_trigger!("[LEFT_SHIFT] ENTER↓"),
            target: key_action_seq!("A↓"),
        };

        let actual = "[LEFT_SHIFT] ENTER↓ : A↓".parse().unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_transform_rule_parse_group() {
        let expected = vec![
            key_rule!("A* : ENTER*"),
            key_rule!("[LEFT_CTRL]B* : ENTER*"),
            key_rule!("C^ : ENTER*"),
        ];
        let actual = KeyTransformRule::from_str_group("A*, [LEFT_CTRL]B*, C^ : ENTER*").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_transform_profile_parse() {
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

    #[test]
    fn test_key_transform_rules_parse_split_keys() {
        let actual = KeyTransformRules::from_str(
            "
        A↓,B↓ : C↓
        ",
        )
        .unwrap();

        let expected = KeyTransformRules::from_str(
            "
        A↓ : C↓
        B↓ : C↓
        ",
        )
        .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_key_transform_rules_parse_expand_transition() {
        let actual = key_profile!(
            "
            Test profile
            A↓ : A↓↑ → B↓↑
            "
        );
        let expected = key_profile!(
            "
            Test profile
            A↓ : A↓ → A↑ → B↓ → B↑ 
            "
        );

        assert_eq!(expected, actual);
    }

    /*    todo:;
        #[test]
        fn test_key_transform_rules_parse_split_transition() {
            let actual: KeyTransformProfile = "
            Test profile;
            A : B;
            "
            .parse()
            .unwrap();

            println!("{}", actual);

            let expected: KeyTransformProfile = "
            Test profile;
            A↓ : B↓;
            A↑ : B↑;
            "
            .parse()
            .unwrap();

            assert_eq!(expected, actual);
        }
    */
}
