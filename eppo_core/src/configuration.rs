use std::collections::HashSet;

use chrono::{DateTime, Utc};

use crate::{
    bandits::{BanditConfiguration, BanditResponse},
    ufc::UniversalFlagConfig,
};

/// Remote configuration for the eppo client. It's a central piece that defines client behavior.
#[derive(Debug)]
pub struct Configuration {
    /// Timestamp when configuration was fetched by the SDK.
    pub fetched_at: DateTime<Utc>,
    /// Flags configuration.
    pub flags: UniversalFlagConfig,
    /// Bandits configuration.
    pub bandits: Option<BanditResponse>,
}

impl Configuration {
    /// Create a new configuration from server responses.
    pub fn from_server_response(
        config: UniversalFlagConfig,
        bandits: Option<BanditResponse>,
    ) -> Configuration {
        let now = Utc::now();

        Configuration {
            fetched_at: now,
            flags: config,
            bandits,
        }
    }

    /// Return a bandit variant for the specified flag key and string flag variation.
    pub(crate) fn get_bandit_key<'a>(&'a self, flag_key: &str, variation: &str) -> Option<&'a str> {
        self.flags
            .compiled
            .flag_to_bandit_associations
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

    /// Get a set of all available flags. Note that this may return both disabled flags and flags
    /// with bad configuration.
    pub fn flag_keys(&self) -> HashSet<String> {
        self.flags.compiled.flags.keys().cloned().collect()
    }
}
