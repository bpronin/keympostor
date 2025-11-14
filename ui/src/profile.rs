use std::ops::Deref;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::slice::Iter;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Profile {
    pub(crate) name: String,
    pub(crate) rule: String,
    pub(crate) layout: Option<String>,
}

impl Profile {
    pub(crate) fn regex(&self) -> Regex {
        Regex::new(self.rule.as_str()).unwrap()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Profiles(Vec<Profile>);

impl Profiles {
    pub(crate) fn new(vec: Vec<Profile>) -> Self {
        Self(vec)
    }

    pub fn get(&self, name: &str) -> Option<&Profile> {
        self.iter().filter(|p| p.name.as_str() == name).next()
    }

    pub fn iter(&self) -> Iter<'_, Profile> {
        self.0.iter()
    }
}

impl Default for Profiles {
    fn default() -> Self {
        Self::new(vec![Profile::default()])
    }
}
