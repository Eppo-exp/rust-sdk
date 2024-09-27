use pyo3::prelude::*;

use eppo_core::SdkMetadata;

mod assignment_logger;
mod client;
mod client_config;
mod configuration;
mod init;

pub(crate) const SDK_METADATA: SdkMetadata = SdkMetadata {
    name: "python",
    version: env!("CARGO_PKG_VERSION"),
};

#[pymodule(module = "eppo_client", name = "_eppo_client")]
mod eppo_client {
    use pyo3::prelude::*;

    #[pymodule_export]
    use crate::{
        assignment_logger::AssignmentLogger,
        client::{EppoClient, EvaluationResult},
        client_config::ClientConfig,
        configuration::Configuration,
        init::{get_instance, init},
    };

    #[pymodule_export]
    use eppo_core::ContextAttributes;

    #[pymodule_init]
    fn module_init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add("__version__", env!("CARGO_PKG_VERSION"))?;
        Ok(())
    }
}
