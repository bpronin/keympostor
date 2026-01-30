use crate::indicator::SerdeLightingColors;
use keympostor::rules::KeyTransformRules;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use std::slice::Iter;

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
pub(crate) struct KeyTransformLayouts(Vec<KeyTransformLayout>);

impl KeyTransformLayouts {
    pub(crate) fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
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

    pub(crate) fn load_default() -> Result<KeyTransformLayouts, Box<dyn Error>> {
        Self::load(LAYOUTS_PATH)
    }

    pub(crate) fn find(&self, name: Option<&str>) -> Option<&KeyTransformLayout> {
        name.and_then(|n| self.0.iter().find(|l| l.name == *n))
    }

    pub(crate) fn cyclic_next(&self, name: Option<&str>) -> Option<&KeyTransformLayout> {
        match name {
            None => self.0.get(0),
            Some(n) => {
                let mut iter = self.0.iter();
                iter.find(|l| l.name == *n);
                iter.next()
            }
        }
    }

    pub(crate) fn iter(&self) -> Iter<'_, KeyTransformLayout> {
        self.0.iter()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::indicator::SerdeLightingColors;
    use crate::layout::{KeyTransformLayout, KeyTransformLayouts};
    use crate::str;
    use keympostor::key_rule;
    use keympostor::rules::KeyTransformRule;
    use keympostor::rules::KeyTransformRules;
    use std::collections::HashMap;
    use std::str::FromStr;

    fn create_test_layout() -> KeyTransformLayout {
        KeyTransformLayout {
            name: "test".to_string(),
            title: "Test layout".to_string(),
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

    fn create_test_layouts() -> KeyTransformLayouts {
        KeyTransformLayouts(vec![
            KeyTransformLayout {
                name: "layout_1".to_string(),
                ..Default::default()
            },
            KeyTransformLayout {
                name: "layout_2".to_string(),
                ..Default::default()
            },
            KeyTransformLayout {
                name: "layout_3".to_string(),
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
            sound: Some(HashMap::from([(
                str!("default"),
                HashMap::from([
                    (str!("default"), str!("sound\\sound1.wav")),
                    (str!("ru_ru"), str!("sound\\sound2.wav")),
                ]),
            )])),
            keyboard_lighting: Some(HashMap::from([
                (
                    str!("default"),
                    HashMap::from([(
                        str!("default"),
                        SerdeLightingColors::from(vec![
                            str!("#0"),
                            str!("#0"),
                            str!("#0"),
                            str!("#0"),
                        ]),
                    )]),
                ),
                (
                    str!("num"),
                    HashMap::from([
                        (
                            str!("default"),
                            SerdeLightingColors::from(vec![
                                str!("#F"),
                                str!("#B"),
                                str!("#C"),
                                str!("#D"),
                            ]),
                        ),
                        (
                            str!("ru_ru"),
                            SerdeLightingColors::from(vec![
                                str!("#F"),
                                str!("#C"),
                                str!("#B"),
                                str!("#A"),
                            ]),
                        ),
                    ]),
                ),
            ])),
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
            keyboard_lighting: Some(HashMap::from([(
                str!("num"),
                HashMap::from([(
                    str!("ru_ru"),
                    SerdeLightingColors::from(vec![
                        str!("#AA0000"),
                        str!("#BB0000"),
                        str!(""),
                        str!("#DD0000"),
                    ]),
                )]),
            )])),
        };

        layout.save("etc/test_data/tmp/saved_layout.toml").unwrap();
    }

    #[test]
    fn test_layouts_load() {
        let result = KeyTransformLayouts::load("etc/test_data/layouts/");
        assert!(result.is_err());
    }

    #[test]
    fn test_layouts_find() {
        let layouts = create_test_layouts();

        assert_eq!(
            Some(&KeyTransformLayout {
                name: "layout_2".to_string(),
                ..Default::default()
            }),
            layouts.find(Some("layout_2"))
        );
        assert_eq!(None, layouts.find(Some("layout_4")));
        assert_eq!(None, layouts.find(None));
    }

    #[test]
    fn test_layouts_cyclic_next() {
        let layouts = create_test_layouts();

        assert_eq!(
            Some(&KeyTransformLayout {
                name: "layout_3".to_string(),
                ..Default::default()
            }),
            layouts.cyclic_next(Some("layout_2"))
        );

        assert_eq!(
            Some(&KeyTransformLayout {
                name: "layout_1".to_string(),
                ..Default::default()
            }),
            layouts.cyclic_next(None)
        );

        assert_eq!(None, layouts.cyclic_next(Some("layout_3")));

        assert_eq!(None, layouts.cyclic_next(Some("layout_4")));
    }
}
