use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct LayoutAutoswitchProfile {
    pub(crate) activation_rule: Option<String>,
    pub(crate) transform_layout: String,
}

impl LayoutAutoswitchProfile {
    pub(crate) fn rule_regex(&self) -> Option<Regex> {
        self.activation_rule
            .as_deref()
            .and_then(|r| Regex::from_str(r).ok())
    }
}

// #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
// pub(crate) struct LayoutAutoswitchProfileList(Vec<LayoutAutoswitchProfile>);
//
// impl LayoutAutoswitchProfileList {
//     pub(crate) fn push(&mut self, profile: LayoutAutoswitchProfile) {
//         self.0.push(profile);
//     }
//
//     pub(crate) fn get(&self, profile_name: &str) -> Option<&LayoutAutoswitchProfile> {
//         self.find(profile_name).and_then(|i| self.0.get(i))
//     }
//
//     pub(crate) fn get_mut(&mut self, profile_name: &str) -> Option<&mut LayoutAutoswitchProfile> {
//         self.find(profile_name).and_then(|i| self.0.get_mut(i))
//     }
//
//     fn find(&self, profile_name: &str) -> Option<usize> {
//         self.0.iter().position(|p| p.name == profile_name)
//     }
// }

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::str;

    #[test]
    fn test_regex_parsing() {
        let profile = LayoutAutoswitchProfile {
            //name: str!("name"),
            activation_rule: Some(str!("")),
            transform_layout: Default::default(),
        };

        assert!(profile.rule_regex().unwrap().is_match("test"));
    }
}
