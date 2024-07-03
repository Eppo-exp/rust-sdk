//! A thread-safe in-memory storage for currently active configuration. [`ConfigurationStore`]
//! provides a concurrent access for readers (e.g., flag evaluation) and writers (e.g., periodic
//! configuration fetcher).
use std::sync::{Arc, RwLock};

use crate::Configuration;

/// `ConfigurationStore` provides a thread-safe (`Sync`) storage for Eppo configuration that allows
/// concurrent access for readers and writers.
///
/// `Configuration` itself is always immutable and can only be replaced fully.
#[derive(Default)]
pub struct ConfigurationStore {
    configuration: RwLock<Arc<Configuration>>,
}

impl ConfigurationStore {
    pub fn new() -> Self {
        ConfigurationStore::default()
    }

    pub fn get_configuration(&self) -> Arc<Configuration> {
        // self.configuration.read() should always return Ok(). Err() is possible only if the lock
        // is poisoned (writer panicked while holding the lock), which should never happen.
        let configuration = self
            .configuration
            .read()
            .expect("thread holding configuration lock should not panic");

        configuration.clone()
    }

    /// Set new configuration.
    pub fn set_configuration(&self, config: Configuration) {
        let config = Arc::new(config);
        let mut configuration_slot = self
            .configuration
            .write()
            .expect("thread holding configuration lock should not panic");

        *configuration_slot = config;
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use super::ConfigurationStore;
    use crate::{ufc::UniversalFlagConfig, Configuration};

    #[test]
    fn can_set_configuration_from_another_thread() {
        let store = Arc::new(ConfigurationStore::new());

        {
            let store = store.clone();
            let _ = std::thread::spawn(move || {
                store.set_configuration(Configuration::new(
                    Some(UniversalFlagConfig {
                        flags: HashMap::new(),
                        bandits: HashMap::new(),
                    }),
                    None,
                ))
            })
            .join();
        }

        assert!(store.get_configuration().flags.is_some());
    }
}
