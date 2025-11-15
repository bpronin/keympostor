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

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Profiles(Vec<Profile>);

impl Profiles {
    pub(crate) fn new(vec: Vec<Profile>) -> Self {
        Self(vec)
    }

    pub(crate) fn get(&self, name: &str) -> Option<&Profile> {
        self.0.iter().filter(|p| p.name == name).next()
    }

    pub(crate) fn get_or_insert(&mut self, name: &str, default: Profile) -> &Profile {
        if let Some(p) = self.0.iter().position(|p| p.name == name) {
            return &self.0[p];
        }
        self.0.push(default);
        self.0.last().unwrap()
    }

    pub(crate) fn iter(&self) -> Iter<'_, Profile> {
        self.0.iter()
    }
}

// #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
// pub struct Profiles(HashMap<String, Profile>);
//
// impl Profiles {
//     pub(crate) fn new(vec: Vec<Profile>) -> Self {
//         let mut map = HashMap::new();
//         for p in vec {
//             map.insert(p.name.clone(), p);
//         }
//         Self(map)
//     }
//
//     pub fn get(&self, name: &str) -> Option<&Profile> {
//         self.0.get(name)
//     }
//
//     pub fn get_or_insert(&mut self, name: &str, default: Profile) -> &Profile {
//         self.0.entry(name.to_string()).or_insert_with(|| default)
//     }
//
//     pub fn iter(&self) -> Values<'_, String, Profile> {
//         self.0.values()
//     }
// }
