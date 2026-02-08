use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct LayoutAutoswitchProfile {
    pub(crate) activation_rule: Option<String>,
    pub(crate) layout: String,
}

impl LayoutAutoswitchProfile {
    pub(crate) fn new(layout: String) -> Self {
        Self {
            layout,
            ..Default::default()
        }
    }

    pub(crate) fn regex(&self) -> Option<Regex> {
        match &self.activation_rule {
            Some(r) => Regex::new(r).ok(),
            None => None,
        }
    }
}

// #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
// pub(crate) struct Profiles(pub(crate) HashMap<String, LayoutAutoswitchProfile>);
//
// impl Profiles {
//     pub(crate) fn get(&self, name: Option<&str>) -> Option<&LayoutAutoswitchProfile> {
//         name.and_then(|n| self.0.get(n))
//     }
//
//     pub(crate) fn get_or_insert(&mut self, name: &str, default: LayoutAutoswitchProfile) -> &LayoutAutoswitchProfile {
//         self.0.entry(name.to_string()).or_insert_with(|| default)
//     }
//
//     pub(crate) fn iter(&self) -> Iter<'_, String, LayoutAutoswitchProfile> {
//         self.0.iter()
//     }
// }

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::str;

    #[test]
    fn test_regex_parsing() {
        let profile = LayoutAutoswitchProfile::default();
        assert!(profile.regex().is_none());

        let profile = LayoutAutoswitchProfile {
            activation_rule: Some(str!("")),
            ..Default::default()
        };

        assert!(profile.regex().unwrap().is_match("test"));
    }
}
