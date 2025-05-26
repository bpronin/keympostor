use crate::keyboard::key::{Key, ScanCode, VirtualKey};
use crate::keyboard::key_action::KeyTransition::{Down, Up};
use crate::keyboard::key_action::{KeyAction, KeyActionSequence, KeyTransition};
use crate::keyboard::key_const::KEY_MAP;
use crate::keyboard::key_modifiers::KeyModifiers::{All, Any};
use crate::keyboard::key_modifiers::{
    KeyModifiers, KeyModifiersState, KM_LALT, KM_LCTRL, KM_LSHIFT, KM_LWIN, KM_NONE, KM_RALT, KM_RCTRL,
    KM_RSHIFT, KM_RWIN,
};
use crate::keyboard::key_trigger::KeyTrigger;
use crate::keyboard::transform_rules::{KeyTransformProfile, KeyTransformRule, KeyTransformRules};
use std::str::{FromStr, Lines};

impl FromStr for VirtualKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        Self::from_code_name(s).or_else(|_| Self::from_name(s))
    }
}

impl FromStr for ScanCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        Self::from_code_name(s).or_else(|_| Self::from_name(s))
    }
}

impl FromStr for Key {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        KEY_MAP.with(|keys| keys.by_name(s.trim()))
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

impl KeyAction {
    fn from_str_expand(s: &str) -> Result<Vec<Self>, String> {
        let s = s.trim();
        let mut list = Vec::new();

        if let Some(k) = s.strip_suffix("*^") {
            let key = Key::from_str(k)?;
            list.push(KeyAction {
                key: key.clone(),
                transition: Down,
            });
            list.push(KeyAction {
                key,
                transition: Up,
            });
        } else if let Some(k) = s.strip_suffix("↓↑") {
            let key = Key::from_str(k)?;
            list.push(KeyAction {
                key: key.clone(),
                transition: Down,
            });
            list.push(KeyAction {
                key,
                transition: Up,
            });
        } else if let Some(k) = s.strip_suffix("*") {
            list.push(KeyAction {
                key: Key::from_str(k)?,
                transition: Down,
            });
        } else if let Some(k) = s.strip_suffix("↓") {
            list.push(KeyAction {
                key: Key::from_str(k)?,
                transition: Down,
            });
        } else if let Some(k) = s.strip_suffix("^") {
            list.push(KeyAction {
                key: Key::from_str(k)?,
                transition: Up,
            });
        } else if let Some(k) = s.strip_suffix("↑") {
            list.push(KeyAction {
                key: Key::from_str(k)?,
                transition: Up,
            });
        } else {
            let key = Key::from_str(s)?;
            list.push(KeyAction {
                key: key.clone(),
                transition: Down,
            });
            list.push(KeyAction {
                key,
                transition: Up,
            });
        }

        Ok(list)
    }
}

impl FromStr for KeyAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str_expand(s)?[0])
    }
}

impl KeyActionSequence {
    fn from_str_list(s: &str) -> Result<Vec<Self>, String> {
        let mut down_actions = Vec::new();
        let mut up_actions = Vec::new();

        let mut has_up_actions = false;
        for part in s.split(|ch| ['→', '>'].contains(&ch)) {
            let actions = KeyAction::from_str_expand(part)?;
            if actions.len() == 1 {
                down_actions.push(actions[0]);
                up_actions.push(actions[0]);
            } else {
                down_actions.push(actions[0]);
                up_actions.push(actions[1]);
                has_up_actions = true;
            }
        }

        let mut list = Vec::new();
        list.push(KeyActionSequence::new(down_actions));
        if has_up_actions {
            list.push(KeyActionSequence::new(up_actions))
        }

        Ok(list)
    }
}

impl FromStr for KeyActionSequence {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str_list(s)?[0].clone())
    }
}

impl FromStr for KeyModifiers {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "*" => Any,
            &_ => All(KeyModifiersState::from_str(s)?),
        })
    }
}

impl FromStr for KeyModifiersState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            Ok(KM_NONE)
        } else {
            let this = s.split('+').fold(KM_NONE, |acc, part| match part.trim() {
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
    fn from_str_list(s: &str) -> Result<Vec<Self>, String> {
        let mut list = Vec::new();
        for part in s.split(',') {
            list.extend(Self::from_str_expand(part)?);
        }
        Ok(list)
    }

    fn from_str_expand(s: &str) -> Result<Vec<KeyTrigger>, String> {
        let s = s.trim();
        let mut list = Vec::new();
        if let Some(s) = s.strip_prefix('[') {
            let mut parts = s.split(']');
            let modifiers = KeyModifiers::from_str(parts.next().unwrap())?;
            for action in KeyAction::from_str_expand(parts.next().unwrap())? {
                list.push(Self { action, modifiers });
            }
        } else {
            for action in KeyAction::from_str_expand(s)? {
                list.push(Self {
                    action,
                    modifiers: Any,
                });
            }
        }

        Ok(list)
    }
}

impl FromStr for KeyTrigger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Some(s) = s.strip_prefix('[') {
            let mut parts = s.split(']');
            Ok(Self {
                modifiers: KeyModifiers::from_str(parts.next().expect("Missing modifiers part"))?, /* Modifiers go first! */
                action: KeyAction::from_str(parts.next().expect("Missing action part."))?,
            })
        } else {
            Ok(Self {
                action: KeyAction::from_str(s)?,
                modifiers: Any,
            })
        }
    }
}

impl KeyTransformRule {
    fn from_str_list(s: &str) -> Result<Vec<Self>, String> {
        let mut parts = s.trim().split(":");

        let triggers = KeyTrigger::from_str_list(parts.next().ok_or("Missing source part.")?)?;
        let actions = KeyActionSequence::from_str(parts.next().ok_or("Missing target part.")?)?;

        let mut rules = Vec::new();
        for trigger in triggers {
            rules.push(KeyTransformRule {
                trigger,
                actions: actions.clone(),
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
            trigger: KeyTrigger::from_str(parts.next().ok_or("Missing source part.")?)?,
            actions: KeyActionSequence::from_str(parts.next().ok_or("Missing target part.")?)?,
        })
    }
}

impl KeyTransformRules {
    fn from_lines(lines: Lines) -> Result<Self, String> {
        let mut items = Vec::new();
        for line in lines {
            let rules = KeyTransformRule::from_str_list(line.trim())?;
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
    use crate::keyboard::key_modifiers::KeyModifiers::{All, Any};
    use crate::keyboard::key_modifiers::{
        KeyModifiers, KeyModifiersState, KM_LSHIFT, KM_NONE, KM_RSHIFT, KM_RWIN,
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
        assert_eq!(KM_NONE, KeyModifiersState::from_str("").unwrap());

        assert_eq!(
            KM_LSHIFT | KM_RSHIFT | KM_RWIN,
            KeyModifiersState::from_str("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN").unwrap()
        );
    }

    #[test]
    fn test_key_modifiers_from_str_fails() {
        assert!(KeyModifiersState::from_str("BANANA").is_err());
    }

    // Keyboard state

    #[test]
    fn test_keyboard_state_all_from_str() {
        assert_eq!(
            All(KM_LSHIFT | KM_RSHIFT | KM_RWIN),
            KeyModifiers::from_str("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN").unwrap()
        );

        assert_eq!(All(KM_NONE), KeyModifiers::from_str("").unwrap());
    }

    #[test]
    fn test_keyboard_state_any_from_str() {
        assert_eq!(Any, KeyModifiers::from_str("*").unwrap());
    }

    // Key action

    #[test]
    fn test_key_action_from_str() {
        assert_eq!(
            KeyAction {
                key: key!("ENTER"),
                transition: Down,
            },
            KeyAction::from_str("ENTER↓").unwrap()
        );

        assert_eq!(
            KeyAction {
                key: key!("F3"),
                transition: Down,
            },
            KeyAction::from_str("    F3\n*").unwrap()
        );
    }

    #[test]
    fn test_key_action_from_str_expand() {
        assert_eq!(
            vec![KeyAction {
                key: key!("A"),
                transition: Down,
            }],
            KeyAction::from_str_expand("A↓").unwrap()
        );

        assert_eq!(
            vec![KeyAction {
                key: key!("B"),
                transition: Up,
            }],
            KeyAction::from_str_expand("B^").unwrap()
        );

        assert_eq!(
            vec![
                KeyAction {
                    key: key!("A"),
                    transition: Down,
                },
                KeyAction {
                    key: key!("A"),
                    transition: Up,
                }
            ],
            KeyAction::from_str_expand("A*^").unwrap()
        );

        assert_eq!(
            vec![
                KeyAction {
                    key: key!("A"),
                    transition: Down,
                },
                KeyAction {
                    key: key!("A"),
                    transition: Up,
                }
            ],
            KeyAction::from_str_expand("A↓↑").unwrap()
        );

        assert_eq!(
            vec![
                KeyAction {
                    key: key!("A"),
                    transition: Down,
                },
                KeyAction {
                    key: key!("A"),
                    transition: Up,
                }
            ],
            KeyAction::from_str_expand("A").unwrap()
        );
    }

    // Key action sequence

    #[test]
    fn test_key_action_sequence_from_str_list() {
        assert_eq!(
            vec![KeyActionSequence::new(vec![key_action!("A↓")]),],
            KeyActionSequence::from_str_list("A↓").unwrap()
        );

        assert_eq!(
            vec![KeyActionSequence::new(vec![
                key_action!("A↓"),
                key_action!("B↑"),
                key_action!("C↓")
            ]),],
            KeyActionSequence::from_str_list("A↓ → B↑ → C↓").unwrap()
        );
    }

    #[test]
    fn test_key_action_sequence_from_str_list_expand() {
        assert_eq!(
            vec![key_action_seq!("A↓"), key_action_seq!("A↑")],
            KeyActionSequence::from_str_list("A").unwrap()
        );

        assert_eq!(
            vec![key_action_seq!("A↓"), key_action_seq!("A↑")],
            KeyActionSequence::from_str_list("A↓↑").unwrap()
        );

        assert_eq!(
            vec![key_action_seq!("A↓ → B↓"), key_action_seq!("A↑ → B↑")],
            KeyActionSequence::from_str_list("A → B").unwrap()
        );

        assert_eq!(
            vec![
                key_action_seq!("A↓ → B↓ → C↓"),
                key_action_seq!("A↑ → B↑ → C↓")
            ],
            KeyActionSequence::from_str_list("A → B → C↓").unwrap()
        );

        assert_eq!(
            vec![
                key_action_seq!("C↓ → A↓ → B↓"),
                key_action_seq!("C↓ → A↑ → B↑")
            ],
            KeyActionSequence::from_str_list("C↓ → A → B").unwrap()
        );
    }

    // Key trigger

    #[test]
    fn test_key_trigger_from_str_all_modifiers() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                modifiers: All(key_mod!("LEFT_SHIFT")),
            },
            KeyTrigger::from_str("[LEFT_SHIFT] A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_no_modifiers() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                modifiers: All(KM_NONE),
            },
            KeyTrigger::from_str("[] A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_any_modifiers() {
        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                modifiers: Any,
            },
            KeyTrigger::from_str("[*]A*").unwrap()
        );

        assert_eq!(
            KeyTrigger {
                action: key_action!("A*"),
                modifiers: Any,
            },
            KeyTrigger::from_str("A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_list() {
        assert_eq!(
            vec![
                key_trigger!("A*"),
                key_trigger!("[LEFT_CTRL]B^"),
                key_trigger!("C*"),
            ],
            KeyTrigger::from_str_list("A*, [LEFT_CTRL]B^, C*").unwrap()
        );

        assert_eq!(
            vec![key_trigger!("A*")],
            KeyTrigger::from_str_list("A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_expand() {
        assert_eq!(
            vec![key_trigger!("A↓"), key_trigger!("A↑")],
            KeyTrigger::from_str_list("A↓↑").unwrap()
        );

        assert_eq!(
            vec![key_trigger!("A↓"), key_trigger!("A↑")],
            KeyTrigger::from_str_list("A").unwrap()
        );

        assert_eq!(
            vec![key_trigger!("[LEFT_CTRL]A↓"), key_trigger!("[LEFT_CTRL]A↑")],
            KeyTrigger::from_str_list("[LEFT_CTRL]A↓↑").unwrap()
        );

        assert_eq!(
            vec![key_trigger!("[LEFT_CTRL]A↓"), key_trigger!("[LEFT_CTRL]A↑")],
            KeyTrigger::from_str_list("[LEFT_CTRL]A").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_list_expand() {
        assert_eq!(
            vec![
                key_trigger!("A↓"),
                key_trigger!("A↑"),
                key_trigger!("B↓"),
                key_trigger!("B↑"),
            ],
            KeyTrigger::from_str_list("A,B").unwrap()
        );

        assert_eq!(
            vec![
                key_trigger!("[LEFT_SHIFT]A↓"),
                key_trigger!("[LEFT_SHIFT]A↑"),
                key_trigger!("[LEFT_CTRL + LEFT_ALT]B↓"),
                key_trigger!("[LEFT_CTRL + LEFT_ALT]B↑"),
            ],
            KeyTrigger::from_str_list("[LEFT_SHIFT] A, [LEFT_CTRL + LEFT_ALT] B").unwrap()
        );
    }

    // Transform rule

    #[test]
    fn test_key_transform_rule_from_str() {
        assert_eq!(
            KeyTransformRule {
                trigger: key_trigger!("[LEFT_SHIFT] ENTER↓"),
                actions: key_action_seq!("A↓"),
            },
            KeyTransformRule::from_str("[LEFT_SHIFT] ENTER↓ : A↓").unwrap()
        );
    }

    #[test]
    fn test_key_transform_rule_from_str_list() {
        let expected = vec![
            key_rule!("A* : ENTER*"),
            key_rule!("[LEFT_CTRL]B* : ENTER*"),
            key_rule!("C^ : ENTER*"),
        ];
        let actual = KeyTransformRule::from_str_list("A*, [LEFT_CTRL]B*, C^ : ENTER*").unwrap();
        assert_eq!(expected, actual);
    }

    // Transform rules

    #[test]
    fn test_key_transform_rules_from_str_no_trigger_transition() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : B↓
                A↑ : B↓
                "#
            ),
            key_rules!(
                "
                A : B↓
                "
            )
        );
    }

    // todo: #[test]
    // fn test_key_transform_rules_from_str_no_transition() {
    //     assert_eq!(
    //         key_rules!(
    //             r#"
    //             A↓ : B↓
    //             A↑ : B↑
    //             "#
    //         ),
    //         key_rules!(
    //             "
    //             A : B
    //             "
    //         )
    //     );
    // }

    #[test]
    fn test_key_transform_rules_from_str_list() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : C↓
                B↓ : C↓
                "#
            ),
            key_rules!("A↓, B↓ : C↓")
        );
    }

    // Transform profile

    #[test]
    fn test_key_transform_profile_from_str() {
        assert_eq!(
            KeyTransformProfile {
                title: "Test profile".to_string(),
                rules: KeyTransformRules {
                    items: vec![
                        key_rule!("A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"),
                        key_rule!("[LEFT_CTRL + LEFT_SHIFT] ENTER↓: ENTER↓ → ENTER↑"),
                    ],
                },
            },
            key_profile!(
                r#"
                Test profile
                A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
                [LEFT_CTRL + LEFT_SHIFT] ENTER↓ : ENTER↓ → ENTER↑
                "#
            )
        );
    }
}
