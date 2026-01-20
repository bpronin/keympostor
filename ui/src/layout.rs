use keympostor::error::KeyError;
use keympostor::key_err;
use keympostor::rules::KeyTransformRules;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::slice::Iter;
use std::str::FromStr;

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    pub name: String,
    pub title: String,
    pub icon: Option<String>,
    pub sound: Option<String>,
    pub rules: KeyTransformRules,
}

impl Layout {
    pub fn load(path: &str) -> Result<Self, KeyError> {
        let text = fs::read_to_string(&path).or(key_err!("Unable to read `{path}` file"))?;
        toml::from_str(&text).or(key_err!("Unable to parse `{path}` file"))
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.title, self.rules)
    }
}

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Layouts(Vec<Layout>);

impl Layouts {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut items = vec![];
        for entry in fs::read_dir(Path::new(path))? {
            let path = entry?.path();
            if path.is_file() {
                let filename = path.to_str().unwrap();
                if let Ok(layout) = Layout::load(filename) {
                    items.push(layout);
                } else {
                    return Err(format!("Corrupted layout: {}", filename))?;
                }
            }
        }
        Ok(Self(items))
    }

    pub fn try_get(&self, name: &Option<&String>) -> Option<&Layout> {
        match name {
            None => None,
            Some(n) => self.get(n),
        }
    }

    fn get(&self, name: &str) -> Option<&Layout> {
        self.0.iter().filter(|p| p.name == name).next()
    }

    pub fn first(&self) -> &Layout {
        &self.0[0]
    }

    pub fn iter(&self) -> Iter<'_, Layout> {
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
