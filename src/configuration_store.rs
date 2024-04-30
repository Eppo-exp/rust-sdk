use std::sync::{Arc, RwLock};

use crate::ufc::Ufc;

/// `ConfigurationStore` provides a Sync storage for feature flags configuration that allows
/// concurrent access for readers and writers.
pub struct ConfigurationStore {
    configuration: RwLock<Option<Arc<Ufc>>>,
}

impl ConfigurationStore {
    pub fn new() -> Self {
        Self {
            configuration: RwLock::new(None),
        }
    }

    pub fn get_configuration(&self) -> Option<Arc<Ufc>> {
        // self.configuration.read() should always return Ok(). Err() is possible only if the lock
        // is poisoned (writer panicked while holding the lock), which should never happen. Still,
        // using .ok()? here to not crash the app.
        let configuration = self.configuration.read().ok()?;
        configuration.clone()
    }

    /// Set new configuration, returning the previous one.
    pub fn set_configuration(&self, ufc: Ufc) -> Option<Arc<Ufc>> {
        // Constructing new value before requesting the lock to minimize lock span.
        let new_value = Some(Arc::new(ufc));

        let mut configuration_slot = self.configuration.write().ok()?;
        std::mem::replace(&mut configuration_slot, new_value)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use crate::ufc::Ufc;

    use super::ConfigurationStore;

    #[test]
    fn can_set_configuration_from_another_thread() {
        let store = Arc::new(ConfigurationStore::new());

        {
            let store = store.clone();
            let _ = std::thread::spawn(move || {
                store.set_configuration(Ufc {
                    flags: HashMap::new(),
                });
            })
            .join();
        }

        assert!(store.get_configuration().is_some());
    }
}
