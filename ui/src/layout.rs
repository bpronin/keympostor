use crate::indicator::SerdeLightingColors;
use keympostor::rule::KeyTransformRules;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

const LAYOUTS_PATH: &str = "layouts";

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct KeyTransformLayout {
    pub(crate) name: String,
    pub(crate) rules: KeyTransformRules,
    pub(crate) title: String,
    pub(crate) icon: Option<String>,
    pub(crate) sound: Option<HashMap<String, HashMap<String, String>>>,
    pub(crate) keyboard_lighting: Option<HashMap<String, HashMap<String, SerdeLightingColors>>>,
}

impl KeyTransformLayout {
    pub(crate) fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let text = fs::read_to_string(path)?;
        let this = toml::from_str(&text)?;
        Ok(this)
    }

    #[allow(dead_code)]
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let text = toml::to_string(self)?;
        fs::write(path, text)?;
        Ok(())
    }
}

impl Display for KeyTransformLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.title, self.rules)
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct KeyTransformLayoutList(Vec<KeyTransformLayout>);

impl<'a> IntoIterator for &'a KeyTransformLayoutList {
    type Item = &'a KeyTransformLayout;
    type IntoIter = std::slice::Iter<'a, KeyTransformLayout>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl KeyTransformLayoutList {
    pub(crate) fn load() -> Result<KeyTransformLayoutList, Box<dyn Error>> {
        Self::load_from(LAYOUTS_PATH)
    }

    fn load_from<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let mut items = vec![];

        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_file() {
                let layout = KeyTransformLayout::load(path)?;
                items.push(layout);
            }
        }

        Ok(Self(items))
    }

    pub(crate) fn find(&self, name: &str) -> Option<&KeyTransformLayout> {
        self.0.iter().find(|l| l.name == *name)
    }

    pub(crate) fn first(&self) -> &KeyTransformLayout {
        self.0.first().expect("Layouts cannot be empty")
    }

    pub(crate) fn cyclic_next(&self, name: &str) -> &KeyTransformLayout {
        let mut iter = self.0.iter();
        iter.find(|l| l.name == *name);
        iter.next()
            .or_else(|| self.0.first())
            .expect("Layouts cannot be empty")
    }
}

#[cfg(test)]
pub mod tests {
    use crate::indicator::SerdeLightingColors;
    use crate::layout::{KeyTransformLayout, KeyTransformLayoutList};
    use crate::{map, str};
    use keympostor::key_rule;
    use keympostor::rule::KeyTransformRule;
    use keympostor::rule::KeyTransformRules;
    use std::str::FromStr;

    fn create_test_layout() -> KeyTransformLayout {
        KeyTransformLayout {
            name: str!("test"),
            title: str!("Test layout"),
            rules: KeyTransformRules::from(vec![
                "[LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑"
                    .parse()
                    .unwrap(),
                "[]CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"
                    .parse()
                    .unwrap(),
            ]),
            ..Default::default()
        }
    }

    fn create_test_layouts() -> KeyTransformLayoutList {
        KeyTransformLayoutList(vec![
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
            rules: KeyTransformRules::from(vec![
                key_rule!("[LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑"),
                key_rule!("[]CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"),
            ]),
        };

        let actual = KeyTransformLayout::load("etc/test_data/layouts/test.toml").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_layout_load_fails() {
        assert!(KeyTransformLayout::load("test/layouts/bad.toml").is_err());
    }

    #[test]
    fn test_layout_save() {
        let layout = KeyTransformLayout {
            name: str!("Sample layout"),
            rules: Default::default(),
            title: str!("Sample layout"),
            icon: Some(str!("image\\default.ico")),
            sound: None,
            keyboard_lighting: Some(map![
                str!("num") =>
                map![
                    str!("ru_ru") =>
                    SerdeLightingColors::from(vec![
                        str!("#AA0000"),
                        str!("#BB0000"),
                        str!(""),
                        str!("#DD0000"),
                    ]),
                ],
            ]),
        };

        layout.save("etc/test_data/tmp/saved_layout.toml").unwrap();
    }

    #[test]
    fn test_layouts_load() {
        let result = KeyTransformLayoutList::load_from("etc/test_data/layouts/");
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
            layouts.find("layout_2")
        );
        assert_eq!(None, layouts.find("layout_4"));
        assert_eq!(None, layouts.find(""));
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
