use keympostor::rules::KeyTransformRules;
use log::warn;
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

    pub(crate) fn load_default() -> Self {
        Self::load(LAYOUTS_PATH).unwrap_or_else(|e| {
            warn!("Failed to load layouts: {}", e);
            Self::default()
        })
    }

    pub(crate) fn get(&self, name: &Option<&String>) -> Option<&Layout> {
        match name {
            None => None,
            Some(n) => self.0.iter().filter(|l| l.name == **n).next(),
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
}
