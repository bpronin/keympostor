use crate::keyboard::key::Key;
use crate::keyboard::key_action::KeyTransition::{Down, Up};
use crate::keyboard::key_action::{KeyAction, KeyActionSequence};
use crate::keyboard::key_modifiers::KeyModifiers::{All, Any};
use crate::keyboard::key_modifiers::{
    KeyModifiers, KeyModifiersState, KM_LALT, KM_LCTRL, KM_LSHIFT, KM_LWIN, KM_NONE, KM_RALT, KM_RCTRL,
    KM_RSHIFT, KM_RWIN,
};
use crate::keyboard::key_trigger::KeyTrigger;
use crate::keyboard::transform_rules::{KeyTransformProfile, KeyTransformRule, KeyTransformRules};
use std::str::{FromStr, Lines};

impl FromStr for Key {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_name(s)
    }
}

impl KeyAction {
    fn from_str_expand(s: &str) -> Result<Vec<Self>, String> {
        let ts = s.trim();
        let mut list = Vec::with_capacity(2);

        if let Some(k) = ts.strip_suffix("*^") {
            let key = Key::from_str(k)?;
            list.push(KeyAction {
                key: key.clone(),
                transition: Down,
            });
            list.push(KeyAction {
                key,
                transition: Up,
            });
        } else if let Some(k) = ts.strip_suffix("↓↑") {
            let key = Key::from_str(k)?;
            list.push(KeyAction {
                key: key.clone(),
                transition: Down,
            });
            list.push(KeyAction {
                key,
                transition: Up,
            });
        } else if let Some(k) = ts.strip_suffix('*') {
            list.push(KeyAction {
                key: Key::from_str(k)?,
                transition: Down,
            });
        } else if let Some(k) = ts.strip_suffix('↓') {
            list.push(KeyAction {
                key: Key::from_str(k)?,
                transition: Down,
            });
        } else if let Some(k) = ts.strip_suffix('^') {
            list.push(KeyAction {
                key: Key::from_str(k)?,
                transition: Up,
            });
        } else if let Some(k) = ts.strip_suffix('↑') {
            list.push(KeyAction {
                key: Key::from_str(k)?,
                transition: Up,
            });
        } else {
            let key = Key::from_str(ts)?;
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

        let mut is_expanded = false;
        for part in s.split(|c| ['→', '>'].contains(&c)) {
            let actions = KeyAction::from_str_expand(part)?;
            down_actions.push(actions[0]);
            if actions.len() == 1 {
                up_actions.push(actions[0]);
            } else {
                up_actions.push(actions[1]);
                is_expanded = true;
            }
        }

        let mut list = Vec::new();
        list.push(KeyActionSequence::new(down_actions));
        if is_expanded {
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
        /* `Any` is parsed outside from `None` */
        Ok(All(KeyModifiersState::from_str(s.trim())?))
    }
}

impl FromStr for KeyModifiersState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ts = s.trim();
        if ts.is_empty() {
            Ok(KM_NONE)
        } else {
            let result = ts.split('+').fold(KM_NONE, |acc, part| {
                acc | match part.trim() {
                    "LEFT_SHIFT" => KM_LSHIFT,
                    "RIGHT_SHIFT" => KM_RSHIFT,
                    "SHIFT" => KM_LSHIFT | KM_RSHIFT,
                    "LEFT_CTRL" => KM_LCTRL,
                    "RIGHT_CTRL" => KM_RCTRL,
                    "CTRL" => KM_LCTRL | KM_RCTRL,
                    "LEFT_ALT" => KM_LALT,
                    "RIGHT_ALT" => KM_RALT,
                    "ALT" => KM_LALT | KM_RALT,
                    "LEFT_WIN" => KM_LWIN,
                    "RIGHT_WIN" => KM_RWIN,
                    "WIN" => KM_LWIN | KM_RWIN,
                    &_ => KM_NONE,
                }
            });

            if result != KM_NONE {
                Ok(result)
            } else {
                Err(format!("Error parsing key modifiers: `{ts}`"))
            }
        }
    }
}

impl KeyTrigger {
    fn from_str_list(s: &str) -> Result<Vec<Vec<Self>>, String> {
        let mut list = Vec::new();
        for part in s.split(',') {
            list.push(Self::from_str_expand(part)?);
        }
        
        Ok(list)
    }

    fn from_str_expand(s: &str) -> Result<Vec<KeyTrigger>, String> {
        let ts = s.trim();
        let mut list = Vec::with_capacity(2);

        if let Some(s) = ts.strip_prefix('[') {
            let mut parts = s.split(']');
            let modifiers = KeyModifiers::from_str(parts.next().expect("Missing modifiers part"))?;
            let actions = KeyAction::from_str_expand(parts.next().expect("Missing actions part"))?;
            for action in actions {
                list.push(Self { action, modifiers });
            }
        } else {
            let actions = KeyAction::from_str_expand(ts)?;
            for action in actions {
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
        let ts = s.trim();
        if let Some(s) = ts.strip_prefix('[') {
            let mut parts = s.split(']');
            Ok(Self { /* Modifiers go first! */
                modifiers: KeyModifiers::from_str(parts.next().expect("Missing modifiers part"))?, 
                action: KeyAction::from_str(parts.next().expect("Missing action part."))?,
            })
        } else {
            Ok(Self {
                action: KeyAction::from_str(ts)?,
                modifiers: Any,
            })
        }
    }
}

impl KeyTransformRule {
    pub(crate) fn from_str_pair(
        triggers_str: &str,
        actions_str: &str,
    ) -> Result<Vec<Self>, String> {
        let triggers_list = KeyTrigger::from_str_list(triggers_str)?;
        let sequences = KeyActionSequence::from_str_list(actions_str)?;
        let mut rules = Vec::new();

        for triggers in triggers_list {
            let len_t = triggers.len();
            let len_s = sequences.len();
            for i in 0..len_t.max(len_s) {
                let rule = KeyTransformRule {
                    trigger: if i < len_t {
                        &triggers[i]
                    } else {
                        &triggers[len_t - 1]
                    }
                    .clone(),
                    actions: if i < len_s {
                        &sequences[i]
                    } else {
                        &sequences[len_s - 1]
                    }
                    .clone(),
                };

                rules.push(rule);
            }
        }

        Ok(rules)
    }

    fn from_str_list(s: &str) -> Result<Vec<Self>, String> {
        let mut parts = s.trim().split(":");
        Self::from_str_pair(
            parts.next().ok_or("Missing source part.")?,
            parts.next().ok_or("Missing target part.")?,
        )
    }
}

impl FromStr for KeyTransformRule {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str_list(s)?[0].clone())
    }
}

impl KeyTransformRules {
    fn from_str_lines(lines: Lines) -> Result<Self, String> {
        let mut items = Vec::new();
        for line in lines {
            items.extend(KeyTransformRule::from_str_list(line.trim())?);
        }

        Ok(Self { items })
    }
}

impl FromStr for KeyTransformRules {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_lines(s.trim().lines())
    }
}

impl FromStr for KeyTransformProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.trim().lines();

        Ok(Self {
            title: lines.next().ok_or("Error parsing title.")?.trim().into(),
            rules: KeyTransformRules::from_str_lines(lines)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::keyboard::key::Key;
    use crate::keyboard::key_action::KeyAction;
    use crate::keyboard::key_action::KeyTransition::{Down, Up};
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

    // Key modifiers

    #[test]
    fn test_key_modifiers_from_str() {
        assert_eq!(All(KM_NONE), KeyModifiers::from_str("").unwrap());

        assert_eq!(
            All(KM_LSHIFT | KM_RSHIFT | KM_RWIN),
            KeyModifiers::from_str("LEFT_SHIFT + RIGHT_SHIFT + RIGHT_WIN").unwrap()
        );
    }

    #[test]
    fn test_key_modifiers_from_str_fails() {
        assert!(KeyModifiers::from_str("BANANA").is_err());
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
            KeyTrigger::from_str("A*").unwrap()
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
                vec![key_trigger!("A*")],
                vec![key_trigger!("[LEFT_CTRL]B^")],
                vec![key_trigger!("C*")],
            ],
            KeyTrigger::from_str_list("A*, [LEFT_CTRL]B^, C*").unwrap()
        );

        assert_eq!(
            vec![vec![key_trigger!("A*")]],
            KeyTrigger::from_str_list("A*").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_expand() {
        assert_eq!(
            vec![vec![key_trigger!("A↓"), key_trigger!("A↑")]],
            KeyTrigger::from_str_list("A↓↑").unwrap()
        );

        assert_eq!(
            vec![vec![key_trigger!("A↓"), key_trigger!("A↑")]],
            KeyTrigger::from_str_list("A").unwrap()
        );

        assert_eq!(
            vec![vec![
                key_trigger!("[LEFT_CTRL]A↓"),
                key_trigger!("[LEFT_CTRL]A↑")
            ]],
            KeyTrigger::from_str_list("[LEFT_CTRL]A↓↑").unwrap()
        );

        assert_eq!(
            vec![vec![
                key_trigger!("[LEFT_CTRL]A↓"),
                key_trigger!("[LEFT_CTRL]A↑")
            ]],
            KeyTrigger::from_str_list("[LEFT_CTRL]A").unwrap()
        );
    }

    #[test]
    fn test_key_trigger_from_str_list_expand() {
        assert_eq!(
            vec![
                vec![key_trigger!("A↓"), key_trigger!("A↑")],
                vec![key_trigger!("B↓"), key_trigger!("B↑")],
            ],
            KeyTrigger::from_str_list("A,B").unwrap()
        );

        assert_eq!(
            vec![
                vec![
                    key_trigger!("[LEFT_SHIFT]A↓"),
                    key_trigger!("[LEFT_SHIFT]A↑")
                ],
                vec![
                    key_trigger!("[LEFT_CTRL + LEFT_ALT]B↓"),
                    key_trigger!("[LEFT_CTRL + LEFT_ALT]B↑")
                ],
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
        assert_eq!(
            vec![
                key_rule!("NUM_DOT↓ : LEFT_ALT↓"),
                key_rule!("NUM_DELETE↓ : LEFT_ALT↓"),
            ],
            KeyTransformRule::from_str_list("NUM_DOT↓, NUM_DELETE↓ : LEFT_ALT↓").unwrap()
        );

        assert_eq!(
            vec![
                key_rule!("A* : ENTER*"),
                key_rule!("[LEFT_CTRL]B* : ENTER*"),
                key_rule!("C^ : ENTER*"),
            ],
            KeyTransformRule::from_str_list("A*, [LEFT_CTRL]B*, C^ : ENTER*").unwrap()
        );
    }

    // Transform rules

    #[test]
    fn test_key_transform_rules_from_str_expand_no_trigger_transition() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : B↓
                A↑ : B↓
                "#
            ),
            key_rules!(
                r#"
                A : B↓
                "#
            )
        );
    }

    #[test]
    fn test_key_transform_rules_from_str_expand_no_action_transition() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : B↓
                A↓ : B↑
                "#
            ),
            key_rules!(
                r#"
                A↓ : B
                "#
            )
        );
    }

    #[test]
    fn test_key_transform_rules_from_str_expand_no_any_transition() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : B↓
                A↑ : B↑
                "#
            ),
            key_rules!(
                "
                A : B
                "
            )
        );
    }

    #[test]
    fn test_key_transform_rules_from_str_expand_list_no_any_transition() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : C↓
                A↑ : C↑
                B↓ : C↓
                B↑ : C↑
                "#
            ),
            key_rules!(
                r#"
                    A,B : C
                    "#
            )
        );
    }

    #[test]
    fn test_key_transform_rules_from_str_expand_multiple_actions() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : B↓ → C↑
                A↑ : B↑ → C↑
                "#
            ),
            key_rules!(
                "
                A : B → C↑
                "
            )
        );
    }

    #[test]
    fn test_key_transform_rules_from_str_expand() {
        assert_eq!(
            key_rules!(
                r#"
                A↓ : B↓
                A↑ : B↓
                C↓ : D↓
                C↓ : D↑
                E↓ : F↓
                E↑ : F↑
                "#
            ),
            key_rules!(
                r#"
                A : B↓
                C↓ : D
                E : F
                "#
            )
        );
    }

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
