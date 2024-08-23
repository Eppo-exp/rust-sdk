use pyo3::prelude::*;

mod assignment_logger;
mod client;
mod client_config;
mod init;

#[pymodule(module = "eppo_client", name = "_eppo_client")]
mod eppo_client {
    #[pymodule_export]
    use crate::{
        assignment_logger::AssignmentLogger,
        client::{EppoClient, EvaluationResult},
        client_config::ClientConfig,
        init::{get_instance, init},
    };

    #[pymodule_export]
    use eppo_core::ContextAttributes;
}
