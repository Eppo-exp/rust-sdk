// We export configuration object into Python world with a limited set of operations to allow
// backend to pass on configuration when initializing the frontend.
use std::sync::Arc;

use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PySet};

use eppo_core::Configuration as CoreConfiguration;

/// Eppo configuration of the client, including flags and bandits configuration.
///
/// Internally, this is a thin wrapper around Rust-owned configuration object.
#[pyclass(frozen, module = "eppo_client")]
pub struct Configuration {
    configuration: Arc<CoreConfiguration>,
}

#[pymethods]
impl Configuration {
    // Returns a set of all flag keys that have been initialized.
    // This can be useful to debug the initialization process.
    fn get_flag_keys<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<PySet>> {
        PySet::new_bound(py, self.configuration.flags.flags.keys())
    }

    // Returns a set of all bandit keys that have been initialized.
    // This can be useful to debug the initialization process.
    fn get_bandit_keys<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<PySet>> {
        PySet::new_bound(
            py,
            self.configuration
                .bandits
                .iter()
                .flat_map(|it| it.bandits.keys()),
        )
    }

    /// Return a string representing flags configuration.
    ///
    /// It should be treated as opaque and passed on to another Eppo client (e.g., javascript client
    /// on frontend) for initialization.
    fn get_flags_configuration(&self) -> PyResult<String> {
        serde_json::to_string(&self.configuration.flags).map_err(|err| {
            log::warn!(target:"eppo", "{err}");
            PyRuntimeError::new_err(err.to_string())
        })
    }
}

impl Configuration {
    pub fn new(configuration: Arc<CoreConfiguration>) -> Configuration {
        Configuration { configuration }
    }
}
