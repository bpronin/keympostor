use crate::indicator::SerdeLightingColors;
use keympostor::rule::KeyTransformRules;
use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

const LAYOUTS_PATH: &str = "layouts";
pub(crate) const DEFAULT_LAYOUT: &str = "default";

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct KeyTransformLayout {
    #[serde(skip)]
    pub(crate) name: String,
    #[serde(default = "serde_untitled")]
    pub(crate) title: String,
    pub(crate) rules: Option<KeyTransformRules>,
    pub(crate) icon: Option<String>,
    pub(crate) sound: Option<HashMap<String, HashMap<String, String>>>,
    pub(crate) keyboard_lighting: Option<HashMap<String, HashMap<String, SerdeLightingColors>>>,
}

impl KeyTransformLayout {
    fn load_from<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let this = match fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text)?,
            Err(error) => {
                warn!(
                    "Failed to load layouts from `{:?}`: {}",
                    path.as_ref(),
                    error
                );
                Self::default()
            }
        };
        Ok(this)
    }
}

impl Display for KeyTransformLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let rules = match &self.rules {
            None => "".to_string(),
            Some(r) => r.to_string(),
        };
        write!(f, "{}\n{}", self.title, rules)
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct TransformLayouts(Vec<KeyTransformLayout>);

impl<'a> IntoIterator for &'a TransformLayouts {
    type Item = &'a KeyTransformLayout;
    type IntoIter = std::slice::Iter<'a, KeyTransformLayout>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl TransformLayouts {
    pub(crate) fn load() -> Result<Self, Box<dyn Error>> {
        let mut this = Self::load_from(LAYOUTS_PATH)?;

        if !this.contains(DEFAULT_LAYOUT) {
            this.0.insert(
                0,
                KeyTransformLayout {
                    name: DEFAULT_LAYOUT.to_string(),
                    title: "Default".to_string(),
                    ..Default::default()
                },
            )
        }

        Ok(this)
    }

    pub(crate) fn get(&self, name: &str) -> &KeyTransformLayout {
        self.try_get(name)
            .expect(&format!("Layout not found: `{}`", name))
    }

    pub(crate) fn contains(&self, name: &str) -> bool {
        self.try_get(name).is_some()
    }

    pub(crate) fn cyclic_next(&self, name: &str) -> &KeyTransformLayout {
        let mut iter = self.0.iter();
        iter.find(|l| l.name == *name);
        iter.next()
            .or_else(|| self.0.first())
            .expect("Layouts cannot be empty")
    }

    fn try_get(&self, name: &str) -> Option<&KeyTransformLayout> {
        self.0.iter().find(|l| l.name == *name)
    }

    fn load_from<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let mut items = vec![];

        if let Ok(dir) = fs::read_dir(path) {
            for entry in dir {
                let path = entry?.path();
                if path.is_file() {
                    let mut layout = KeyTransformLayout::load_from(&path)?;
                    layout.name = path.file_stem().unwrap().to_str().unwrap().to_string();
                    items.push(layout);
                }
            }

            items.sort_by(|a, b| a.title.cmp(&b.title));
        }

        Ok(Self(items))
    }
}

fn serde_untitled() -> String {
    "Untitled layout".to_string()
}

#[cfg(test)]
pub mod tests {
    use crate::indicator::SerdeLightingColors;
    use crate::layout::{KeyTransformLayout, TransformLayouts};
    use crate::{map, str};
    use keympostor::key_rule;
    use keympostor::rule::KeyTransformRule;
    use keympostor::rule::KeyTransformRules;
    use std::str::FromStr;

    fn create_test_layout() -> KeyTransformLayout {
        KeyTransformLayout {
            name: str!("test"),
            title: str!("Test layout"),
            rules: Some(KeyTransformRules::from(vec![
                "[LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑"
                    .parse()
                    .unwrap(),
                "[]CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"
                    .parse()
                    .unwrap(),
            ])),
            ..Default::default()
        }
    }

    fn create_test_layouts() -> TransformLayouts {
        TransformLayouts(vec![
            KeyTransformLayout {
                name: str!("layout_1"),
                ..Default::default()
            },
            KeyTransformLayout {
                name: str!("layout_2"),
                ..Default::default()
            },
            KeyTransformLayout {
                name: str!("layout_3"),
                ..Default::default()
            },
        ])
    }

    #[test]
    fn test_layout_serialize() {
        let layout = create_test_layout();

        let expected = r#"
            name = "test"
            title = "Test layout"
            [rules]
            "[LEFT_SHIFT]CAPS_LOCK↓" = "CAPS_LOCK↓ → CAPS_LOCK↑"
            "[]CAPS_LOCK↓" = "LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"
            "#;

        let actual = toml::to_string_pretty(&layout).unwrap();

        assert_eq!(
            expected.split_whitespace().collect::<String>(),
            actual.split_whitespace().collect::<String>()
        );
    }

    #[test]
    fn test_layout_deserialize() {
        let actual = toml::from_str(
            &r#"
            name = "test"
            title = "Test layout"
            [rules]
            "[LEFT_SHIFT]CAPS_LOCK↓" = "CAPS_LOCK↓ → CAPS_LOCK↑"
            "[]CAPS_LOCK↓" = "LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"
            "#,
        )
        .unwrap();

        /* NOTE: rules deserialized as a sorted map so check the "expected" order */
        let expected = create_test_layout();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_layout_load() {
        let expected = KeyTransformLayout {
            name: str!("sample"),
            title: str!("Sample layout"),
            icon: Some(str!("image\\default.ico")),
            sound: Some(map![
                str!("default") => map![
                    str!("default")=> str!("sound\\sound1.wav"),
                    str!("ru_ru")=> str!("sound\\sound2.wav"),
                ],
            ]),
            keyboard_lighting: Some(map![
                str!("default") => map![
                    str!("default") => SerdeLightingColors::from(vec![
                        str!("#0"),
                        str!("#0"),
                        str!("#0"),
                        str!("#0"),
                    ],
                )],
                str!("num") => map![
                    str!("default") => SerdeLightingColors::from(vec![
                        str!("#F"),
                        str!("#B"),
                        str!("#C"),
                        str!("#D"),
                    ]),
                    str!("ru_ru") => SerdeLightingColors::from(vec![
                        str!("#F"),
                        str!("#C"),
                        str!("#B"),
                        str!("#A"),
                    ]),
                ],
            ]),
            rules: Some(KeyTransformRules::from(vec![
                key_rule!("[LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑"),
                key_rule!("[]CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"),
            ])),
        };

        let actual = KeyTransformLayout::load_from("etc/test_data/layouts/test.toml").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_layout_load_fails() {
        assert!(KeyTransformLayout::load_from("test/layouts/bad.toml").is_err());
    }
    //
    // #[test]
    // fn test_layout_save() {
    //     let layout = KeyTransformLayout {
    //         name: str!("Sample layout"),
    //         rules: Default::default(),
    //         title: str!("Sample layout"),
    //         icon: Some(str!("image\\default.ico")),
    //         sound: None,
    //         keyboard_lighting: Some(map![
    //             str!("num") =>
    //             map![
    //                 str!("ru_ru") =>
    //                 SerdeLightingColors::from(vec![
    //                     str!("#AA0000"),
    //                     str!("#BB0000"),
    //                     str!(""),
    //                     str!("#DD0000"),
    //                 ]),
    //             ],
    //         ]),
    //     };
    //
    //     layout.save("etc/test_data/tmp/saved_layout.toml").unwrap();
    // }

    #[test]
    fn test_layouts_load() {
        let result = TransformLayouts::load_from("etc/test_data/layouts/");
        assert!(result.is_err());
    }

    #[test]
    fn test_layouts_find() {
        let layouts = create_test_layouts();

        assert_eq!(
            Some(&KeyTransformLayout {
                name: str!("layout_2"),
                ..Default::default()
            }),
            layouts.try_get("layout_2")
        );
        assert_eq!(None, layouts.try_get("layout_4"));
        assert_eq!(None, layouts.try_get(""));
    }

    #[test]
    fn test_layouts_cyclic_next() {
        let layouts = create_test_layouts();

        assert_eq!(
            &KeyTransformLayout {
                name: str!("layout_3"),
                ..Default::default()
            },
            layouts.cyclic_next("layout_2")
        );

        assert_eq!(
            &KeyTransformLayout {
                name: str!("layout_1"),
                ..Default::default()
            },
            layouts.cyclic_next("")
        );
    }
}
