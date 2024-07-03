use std::collections::HashMap;

use crate::{
    bandits::{BanditConfiguration, BanditResponse},
    ufc::{BanditVariation, UniversalFlagConfig},
};

/// Remote configuration for the eppo client. It's a central piece that defines client behavior.
#[derive(Default, Clone)]
pub struct Configuration {
    /// Flags configuration.
    pub flags: Option<UniversalFlagConfig>,
    /// Bandits configuration.
    pub bandits: Option<BanditResponse>,
    /// Mapping from flag key to flag variation value to bandit variation.
    pub flag_to_bandit_associations:
        HashMap</* flag_key: */ String, HashMap</* variation_key: */ String, BanditVariation>>,
}

impl Configuration {
    /// Create a new configuration from server responses.
    pub fn new(
        config: Option<UniversalFlagConfig>,
        bandits: Option<BanditResponse>,
    ) -> Configuration {
        let flag_to_bandit_associations = config
            .as_ref()
            .map(get_flag_to_bandit_associations)
            .unwrap_or_default();
        Configuration {
            flags: config,
            bandits,
            flag_to_bandit_associations,
        }
    }

    /// Return a bandit variant for the specified flag key and string flag variation.
    pub(crate) fn get_bandit_key<'a>(&'a self, flag_key: &str, variation: &str) -> Option<&'a str> {
        self.flag_to_bandit_associations
            .get(flag_key)
            .and_then(|x| x.get(variation))
            .map(|variation| variation.key.as_str())
    }

    /// Return bandit configuration for the given key.
    ///
    /// Returns `None` if bandits are missing for bandit does not exist.
    pub(crate) fn get_bandit<'a>(&'a self, bandit_key: &str) -> Option<&'a BanditConfiguration> {
        self.bandits.as_ref()?.bandits.get(bandit_key)
    }
}

fn get_flag_to_bandit_associations(
    config: &UniversalFlagConfig,
) -> HashMap<String, HashMap<String, BanditVariation>> {
    config
        .bandits
        .iter()
        .flat_map(|(_, bandits)| bandits.iter())
        .fold(HashMap::new(), |mut acc, variation| {
            acc.entry(variation.flag_key.clone())
                .or_default()
                .insert(variation.variation_value.clone(), variation.clone());
            acc
        })
}
