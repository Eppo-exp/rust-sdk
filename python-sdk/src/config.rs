use pyo3::{exceptions::PyValueError, prelude::*, PyTraverseError, PyVisit};

use eppo_core::{configuration_fetcher::DEFAULT_BASE_URL, poller_thread::PollerThreadConfig};

use crate::assignment_logger::AssignmentLogger;

#[pyclass(module = "eppo_client", get_all, set_all)]
pub struct Config {
    pub(crate) api_key: String,
    pub(crate) base_url: String,
    pub(crate) assignment_logger: Option<Py<AssignmentLogger>>,
    pub(crate) is_graceful_mode: bool,
    pub(crate) poll_interval_seconds: u64,
    pub(crate) poll_jitter_seconds: u64,
}

#[pymethods]
impl Config {
    #[new]
    #[pyo3(signature = (
            api_key,
            *,
            base_url=DEFAULT_BASE_URL.to_owned(),
            assignment_logger,
            is_graceful_mode=true,
            poll_interval_seconds=PollerThreadConfig::DEFAULT_POLL_INTERVAL.as_secs(),
            poll_jitter_seconds=PollerThreadConfig::DEFAULT_POLL_JITTER.as_secs(),
        ))]
    fn new(
        api_key: String,
        base_url: String,
        assignment_logger: Py<AssignmentLogger>,
        is_graceful_mode: bool,
        poll_interval_seconds: u64,
        poll_jitter_seconds: u64,
    ) -> PyResult<Config> {
        if api_key.is_empty() {
            return Err(PyValueError::new_err(
                "Invalid value for api_key: cannot be blank",
            ));
        }

        Ok(Config {
            api_key,
            base_url,
            assignment_logger: Some(assignment_logger),
            is_graceful_mode,
            poll_interval_seconds,
            poll_jitter_seconds,
        })
    }

    // Overriding the default setter to make `assignment_logger` non-optional.
    #[setter]
    fn set_assignment_logger(&mut self, assignment_logger: Py<AssignmentLogger>) {
        self.assignment_logger = Some(assignment_logger);
    }

    // Implementing [Garbage Collector integration][1] in case user's `AssignmentLogger` holds a
    // reference to `Config`. This will allow the GC to detect this cycle and break it.
    //
    // [1]: https://pyo3.rs/v0.22.2/class/protocols.html#garbage-collector-integration
    fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {
        if let Some(assignment_logger) = &self.assignment_logger {
            visit.call(assignment_logger)?;
        }
        Ok(())
    }
    fn __clear__(&mut self) {
        self.assignment_logger = None;
    }
}
