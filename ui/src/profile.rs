use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::Iter;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Profile {
    pub(crate) rule: Option<String>,
    pub(crate) layout: Option<String>,
}

impl Profile {
    pub(crate) fn regex(&self) -> Option<Regex> {
        match &self.rule {
            Some(r) => Regex::new(r).ok(),
            None => None,
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Profiles(pub HashMap<String, Profile>);

impl Profiles {
    pub fn get(&self, name: &str) -> Option<&Profile> {
        self.0.get(name)
    }

    pub fn get_or_insert(&mut self, name: &str, default: Profile) -> &Profile {
        self.0.entry(name.to_string()).or_insert_with(|| default)
    }

    pub fn iter(&self) -> Iter<'_, String, Profile> {
        self.0.iter()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_regex_parsing() {
        let profile = Profile::default();
        assert!(profile.regex().is_none());

        let profile = Profile {
            rule: Some("".to_string()),
            ..Default::default()
        };

        assert!(profile.regex().unwrap().is_match("test"));
    }
}
