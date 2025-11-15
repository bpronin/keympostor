use regex::Regex;
use serde::de::{SeqAccess, Visitor};
use serde::ser::{SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

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

#[derive(Debug, Default, PartialEq)]
pub struct Profiles(HashMap<String, Profile>);

impl Profiles {
    // pub fn get(&self, name: &str) -> Option<&Profile> {
    //     self.0.get(name)
    // }

    pub fn get_or_insert(&mut self, name: &str, default: Profile) -> &Profile {
        self.0.entry(name.to_string()).or_insert_with(|| default)
    }

    pub fn iter(&self) -> Values<'_, String, Profile> {
        self.0.values()
    }
}

impl From<Vec<Profile>> for Profiles {
    fn from(vec: Vec<Profile>) -> Self {
        let mut map = HashMap::new();
        for p in vec {
            map.insert(p.name.clone(), p);
        }
        Self(map)
    }
}

impl Serialize for Profiles {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for profile in &self.0 {
            seq.serialize_element(&profile.1)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Profiles {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ProfileVisitor)
    }
}

struct ProfileVisitor;

impl<'de> Visitor<'de> for ProfileVisitor {
    type Value = Profiles;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a sequence")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut items = HashMap::new();

        while let Some(p) = seq.next_element::<Profile>()? {
            items.insert(p.name.clone(), p);
        }

        Ok(Profiles(items))
    }
}
