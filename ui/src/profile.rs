use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Iter;
use std::collections::HashMap;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Profile {
    pub(crate) activation_rule: Option<String>,
    pub(crate) layout: String,
}

impl Profile {
    pub(crate) fn regex(&self) -> Option<Regex> {
        match &self.activation_rule {
            Some(r) => Regex::new(r).ok(),
            None => None,
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Profiles(pub(crate) HashMap<String, Profile>);

impl Profiles {
    pub(crate) fn get(&self, name: Option<&str>) -> Option<&Profile> {
        name.and_then(|n| self.0.get(n))
    }

    pub(crate) fn get_or_insert(&mut self, name: &str, default: Profile) -> &Profile {
        self.0.entry(name.to_string()).or_insert_with(|| default)
    }

    pub(crate) fn iter(&self) -> Iter<'_, String, Profile> {
        self.0.iter()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::str;
    use super::*;

    #[test]
    fn test_regex_parsing() {
        let profile = Profile::default();
        assert!(profile.regex().is_none());

        let profile = Profile {
            activation_rule: Some(str!("")),
            ..Default::default()
        };

        assert!(profile.regex().unwrap().is_match("test"));
    }
}
