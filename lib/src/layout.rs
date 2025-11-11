use crate::keyboard::error::KeyError;
use crate::keyboard::rules::KeyTransformRules;
use anyhow::{Context, Result};
use log::warn;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use std::slice::Iter;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    pub name: String,
    pub title: String,
    pub rules: KeyTransformRules,
}

impl Layout {
    pub fn load(path: &str) -> Result<Self> {
        toml::from_str(&fs::read_to_string(&path).context(format!("Unable to read {} file", path))?)
            .context(format!("Unable to parse {}", path))
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.title, self.rules)
    }
}

impl FromStr for Layout {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.trim().lines();

        Ok(Self {
            name: lines
                .next()
                .ok_or(KeyError::new("Error parsing name."))?
                .trim()
                .into(),
            title: lines
                .next()
                .ok_or(KeyError::new("Error parsing title."))?
                .trim()
                .into(),
            rules: KeyTransformRules::from_str_lines(lines)?,
        })
    }
}

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Layouts(Vec<Layout>);

impl Layouts {
    pub fn load(path: &str) -> Result<Self> {
        let mut items = vec![];
        for entry in fs::read_dir(Path::new(path))? {
            let path = entry?.path();
            if path.is_file() {
                let filename = path.to_str().unwrap();
                if let Ok(layout) = Layout::load(filename) {
                    items.push(layout);
                } else {
                    warn!("Ignored corrupted layout: {}", filename);
                }
            }
        }
        Ok(Self(items))
    }

    pub fn get(&self, name: &str) -> Option<&Layout> {
        self.iter().filter(|p| p.name == name).next()
    }

    pub fn iter(&self) -> Iter<'_, Layout> {
        self.0.iter()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::key_rule;
    use crate::keyboard::rules::KeyTransformRule;
    use crate::keyboard::rules::KeyTransformRules;
    use crate::layout::{Layout, Layouts};
    use anyhow::{Context, Error};
    use std::fs;

    #[macro_export]
    macro_rules! key_layout {
        ($text:expr) => {
            $text.parse::<Layout>().unwrap()
        };
    }

    impl Layout {
        pub(crate) fn save(&self, path: &str) -> Result<(), Error> {
            fs::write(
                path,
                toml::to_string_pretty(self).context(format!("Unable to serialize {}", path))?,
            )
            .context(format!("Unable to write {} file", path))
        }
    }

    #[test]
    fn test_layout_from_str() {
        assert_eq!(
            Layout {
                name: "test".to_string(),
                title: "Test layout".to_string(),
                rules: KeyTransformRules::from(vec![
                    key_rule!("A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑"),
                    key_rule!("[LEFT_CTRL + LEFT_SHIFT] ENTER↓: ENTER↓ → ENTER↑"),
                ],)
            },
            key_layout!(
                r#"
                test
                Test layout
                A↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
                [LEFT_CTRL + LEFT_SHIFT] ENTER↓ : ENTER↓ → ENTER↑
                "#
            )
        );
    }

    #[test]
    fn test_layout_serialize() {
        let layout = key_layout!(
            r#"
            test
            Test layout
            [LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑
            []CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
            "#
        );
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
        let expected = key_layout!(
            r#"
            test
            Test layout
            [LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑
            []CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
            "#
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_layout_load() {
        let actual = Layout::load("etc/test_data/layouts/test.toml").unwrap();

        /* NOTE: rules deserialized as a sorted map so check the "expected" order */
        let expected = key_layout!(
            "
            test
            Test layout
            [LEFT_SHIFT]CAPS_LOCK↓ : CAPS_LOCK↓ → CAPS_LOCK↑
            []CAPS_LOCK↓ : LEFT_WIN↓ → SPACE↓ → SPACE↑ → LEFT_WIN↑
            "
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_layout_load_fails() {
        assert!(Layout::load("test/layouts/bad.toml").is_err());
    }

    #[test]
    fn test_layout_save() {
        let actual = Layout::load("etc/test_data/layouts/test.toml").unwrap();

        actual.save("etc/test_data/layouts/test-copy.toml").unwrap();

        let expected = Layout::load("etc/test_data/layouts/test-copy.toml").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_layouts_load() {
        let result = Layouts::load("etc/test_data/layouts/");
        assert!(result.is_ok());
    }
}
