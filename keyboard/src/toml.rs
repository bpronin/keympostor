use crate::key_transform_rule::{KeyTransformProfile, KeyTransformRule};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct KeyTransformProfileToml {
    profile: String,
    rules: BTreeMap<String, String>,
}

impl KeyTransformProfileToml {
    pub(crate) fn load_profile(path: &str) -> Result<Self, String> {
        toml::from_str(
            &fs::read_to_string(&path)
                .map_err(|e| format!("Unable to read {} file.\n{}", path, e))?,
        )
        .map_err(|e| format!("Unable to parse {}.\n{}", path, e))
    }

    pub(crate) fn from_profile(profile: &KeyTransformProfile) -> Self {
        Self {
            profile: profile.title.clone(),
            rules: BTreeMap::from_iter(
                profile
                    .rules
                    .iter()
                    .map(|rule| (rule.source.to_string(), rule.target.to_string()))
                    .collect::<Vec<_>>(),
            ),
        }
    }

    pub(crate) fn save(&self, path: &str) -> Result<(), String> {
        fs::write(
            path,
            toml::to_string(self).map_err(|e| format!("Unable to serialize {}.\n{}", path, e))?,
        )
        .map_err(|e| format!("Unable to write {} file.\n{}", path, e))
    }

    pub(crate) fn to_profile(&self) -> Result<KeyTransformProfile, String> {
        Ok(KeyTransformProfile {
            title: (&self.profile).to_string(),
            rules: self
                .rules
                .iter()
                .map(|entry| KeyTransformRule {
                    source: entry.0.parse().unwrap(),
                    target: entry.1.parse().unwrap(),
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::toml::KeyTransformProfileToml;

    #[test]
    fn test_key_transform_profile_toml() {
        let toml = KeyTransformProfileToml::load_profile("../test/profiles/game.toml").unwrap();
        toml.save("../test/profiles/game_copy.toml").unwrap();
    }
}
