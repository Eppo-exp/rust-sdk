//! A thread-safe in-memory storage for currently active configuration. [`ConfigurationStore`]
//! provides concurrent access for readers (e.g., flag evaluation) and writers (e.g., periodic
//! configuration fetcher).
use std::sync::{Arc, RwLock};

use crate::Configuration;

/// `ConfigurationStore` provides a thread-safe (`Sync`) storage for Eppo configuration that allows
/// concurrent access for readers and writers.
///
/// `Configuration` itself is always immutable and can only be replaced completely.
#[derive(Default)]
pub struct ConfigurationStore {
    configuration: RwLock<Option<Arc<Configuration>>>,
}

impl ConfigurationStore {
    /// Create a new empty configuration store.
    pub fn new() -> Self {
        ConfigurationStore::default()
    }

    /// Get currently-active configuration. Returns None if configuration hasn't been fetched/stored
    /// yet.
    pub fn get_configuration(&self) -> Option<Arc<Configuration>> {
        // self.configuration.read() should always return Ok(). Err() is possible only if the lock
        // is poisoned (writer panicked while holding the lock), which should never happen.
        let configuration = self
            .configuration
            .read()
            .expect("thread holding configuration lock should not panic");

        configuration.clone()
    }

    /// Set new configuration.
    pub fn set_configuration(&self, config: Arc<Configuration>) {
        let mut configuration_slot = self
            .configuration
            .write()
            .expect("thread holding configuration lock should not panic");

        *configuration_slot = Some(config);
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use chrono::Utc;

    use super::ConfigurationStore;
    use crate::{
        ufc::{CompiledFlagsConfig, Environment, UniversalFlagConfig},
        Configuration,
    };

    #[test]
    fn can_set_configuration_from_another_thread() {
        let store = Arc::new(ConfigurationStore::new());

        assert!(store.get_configuration().is_none());

        {
            let store = store.clone();
            let _ = std::thread::spawn(move || {
                store.set_configuration(Arc::new(Configuration::from_server_response(
                    UniversalFlagConfig {
                        wire_json: b"test-bytes".to_vec(),
                        compiled: CompiledFlagsConfig {
                            created_at: Utc::now(),
                            environment: Environment {
                                name: "test".into(),
                            },
                            flags: HashMap::new(),
                            flag_to_bandit_associations: HashMap::new(),
                        },
                    },
                    None,
                )))
            })
            .join();
        }

        assert!(store.get_configuration().is_some());
    }
}
