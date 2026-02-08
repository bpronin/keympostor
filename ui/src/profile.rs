use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct LayoutAutoswitchProfile {
    pub(crate) activation_rule: Option<String>,
    pub(crate) layout: String,
}

impl LayoutAutoswitchProfile {
    pub(crate) fn regex(&self) -> Option<Regex> {
        match &self.activation_rule {
            Some(r) => Regex::new(r).ok(),
            None => None,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::str;

    #[test]
    fn test_regex_parsing() {
        let profile = LayoutAutoswitchProfile {
            activation_rule: Some(str!("")),
            layout: Default::default(),
        };

        assert!(profile.regex().unwrap().is_match("test"));
    }
}
