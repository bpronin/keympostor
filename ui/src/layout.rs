use keympostor::rules::KeyTransformRules;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use std::slice::Iter;

const LAYOUTS_PATH: &str = "layouts";

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Layout {
    pub name: String,
    pub title: String,
    pub icon: Option<String>,
    pub sound: Option<String>,
    pub rules: KeyTransformRules,
}

impl Layout {
    pub(crate) fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let text = fs::read_to_string(path)?;
        let this = toml::from_str(&text)?;
        Ok(this)
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.title, self.rules)
    }
}

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Layouts(Vec<Layout>);

impl Layouts {
    pub(crate) fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let mut items = vec![];

        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_file() {
                let layout = Layout::load(path)?;
                items.push(layout);
            }
        }

        Ok(Self(items))
    }

    pub(crate) fn load_default() -> Result<Layouts, Box<dyn Error>> {
        Self::load(LAYOUTS_PATH)
    }

    pub(crate) fn find(&self, name: Option<&str>) -> Option<&Layout> {
        name.and_then(|n| self.0.iter().find(|l| l.name == *n))
    }

    pub(crate) fn cyclic_next(&self, name: Option<&str>) -> Option<&Layout> {
        match name {
            None => self.0.get(0),
            Some(n) => {
                let mut iter = self.0.iter();
                iter.find(|l| l.name == *n);
                iter.next()
            }
        }
    }

    pub(crate) fn iter(&self) -> Iter<'_, Layout> {
        self.0.iter()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::layout::{Layout, Layouts};
    use keympostor::rules::KeyTransformRules;

    #[test]
    fn test_layout_serialize() {
        let layout = Layout {
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
        };

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
        let expected = Layout {
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
        };

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_layout_load() {
        let actual = Layout::load("etc/test_data/layouts/test.toml").unwrap();

        /* NOTE: rules deserialized as a sorted map so check the "expected" order */
        let expected = Layout {
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
        };

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_layout_load_fails() {
        assert!(Layout::load("test/layouts/bad.toml").is_err());
    }

    #[test]
    fn test_layouts_load() {
        let result = Layouts::load("etc/test_data/layouts/");
        assert!(result.is_err());
    }

    #[test]
    fn test_layouts_find() {
        let layouts = Layouts(vec![
            Layout {
                name: "layout_1".to_string(),
                ..Default::default()
            },
            Layout {
                name: "layout_2".to_string(),
                ..Default::default()
            },
            Layout {
                name: "layout_3".to_string(),
                ..Default::default()
            },
        ]);

        assert_eq!(
            Some(&Layout {
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
        let layouts = Layouts(vec![
            Layout {
                name: "layout_1".to_string(),
                ..Default::default()
            },
            Layout {
                name: "layout_2".to_string(),
                ..Default::default()
            },
            Layout {
                name: "layout_3".to_string(),
                ..Default::default()
            },
        ]);

        assert_eq!(
            Some(&Layout {
                name: "layout_3".to_string(),
                ..Default::default()
            }),
            layouts.cyclic_next(Some("layout_2"))
        );

        assert_eq!(
            Some(&Layout {
                name: "layout_1".to_string(),
                ..Default::default()
            }),
            layouts.cyclic_next(None)
        );

        assert_eq!(None, layouts.cyclic_next(Some("layout_3")));

        assert_eq!(None, layouts.cyclic_next(Some("layout_4")));
    }
}
