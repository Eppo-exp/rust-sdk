// We export configuration object into Python world with a limited set of operations to allow
// backend to pass on configuration when initializing the frontend.
use std::{borrow::Cow, sync::Arc};

use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::PySet,
};

use eppo_core::{ufc::UniversalFlagConfig, Configuration as CoreConfiguration};

use crate::SDK_METADATA;

/// Eppo configuration of the client, including flags and bandits configuration.
///
/// Internally, this is a thin wrapper around Rust-owned configuration object.
#[pyclass(frozen, module = "eppo_client")]
pub struct Configuration {
    pub configuration: Arc<CoreConfiguration>,
}

#[pymethods]
impl Configuration {
    #[new]
    #[pyo3(signature = (*, flags_configuration, bandits_configuration = None))]
    fn py_new(
        flags_configuration: Vec<u8>,
        bandits_configuration: Option<&[u8]>,
    ) -> PyResult<Configuration> {
        let flag_config = UniversalFlagConfig::from_json(SDK_METADATA, flags_configuration)
            .map_err(|err| {
                PyValueError::new_err(format!("argument 'flags_configuration': {err:?}"))
            })?;
        let bandits_config = bandits_configuration
            .map(|it| serde_json::from_slice(it))
            .transpose()
            .map_err(|err| {
                PyValueError::new_err(format!("argument 'bandits_configuration': {err:?}"))
            })?;

        Ok(Configuration {
            configuration: Arc::new(CoreConfiguration::from_server_response(
                flag_config,
                bandits_config,
            )),
        })
    }

    // Returns a set of all flag keys that have been initialized.
    // This can be useful to debug the initialization process.
    fn get_flag_keys<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<PySet>> {
        PySet::new_bound(py, &self.configuration.flag_keys())
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

    /// Return bytes representing flags configuration.
    ///
    /// It should be treated as opaque and passed on to another Eppo client (e.g., javascript client
    /// on frontend) for initialization.
    fn get_flags_configuration(&self) -> Cow<[u8]> {
        Cow::Borrowed(self.configuration.flags.to_json())
    }

    /// Return bytes representing bandits configuration.
    ///
    /// It should be treated as opaque and passed on to another Eppo client for initialization.
    fn get_bandits_configuration(&self) -> PyResult<Option<Cow<[u8]>>> {
        self.configuration
            .bandits
            .as_ref()
            .map(|it| serde_json::to_vec(it).map(Cow::Owned))
            .transpose()
            .map_err(|err| {
                // This should normally never happen.
                PyRuntimeError::new_err(format!(
                    "failed to serialize bandits configuration: {err:?}"
                ))
            })
    }
}

impl Configuration {
    pub fn new(configuration: Arc<CoreConfiguration>) -> Configuration {
        Configuration { configuration }
    }
}
