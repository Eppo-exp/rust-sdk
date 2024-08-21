use std::{
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use pyo3::prelude::*;
use pyo3::{
    exceptions::PyException,
    types::{PyDict, PyString},
};

use eppo_core::{
    configuration_fetcher::{ConfigurationFetcher, DEFAULT_BASE_URL},
    configuration_store::ConfigurationStore,
    eval::get_assignment,
    poller_thread::{PollerThread, PollerThreadConfig},
    pyo3::TryToPyObject,
    ufc::VariationType,
    Attributes,
};

#[pymodule(module = "eppo_client", name = "_eppo_client")]
mod eppo_client {
    static CLIENT_INSTANCE: RwLock<Option<Py<EppoClient>>> = RwLock::new(None);

    use eppo_core::{eval::get_assignment_details, events::AssignmentEvent};
    use pyo3::{
        exceptions::{PyRuntimeError, PyValueError},
        intern,
        types::{PyBool, PyFloat, PyInt},
        PyTraverseError, PyVisit,
    };

    use super::*;

    #[pyclass(get_all, set_all)]
    struct Config {
        api_key: String,
        base_url: String,
        assignment_logger: Option<Py<AssignmentLogger>>,
        is_graceful_mode: bool,
        poll_interval_seconds: u64,
        poll_jitter_seconds: u64,
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

    #[derive(Debug, Clone)]
    #[pyclass(frozen, subclass)]
    struct AssignmentLogger {}
    #[pymethods]
    impl AssignmentLogger {
        #[new]
        fn new() -> AssignmentLogger {
            AssignmentLogger {}
        }

        #[allow(unused_variables)]
        fn log_assignment(slf: Bound<Self>, event: Bound<PyDict>) {}

        #[allow(unused_variables)]
        fn log_bandit_action(slf: Bound<Self>, event: Bound<PyDict>) {}
    }

    #[pyclass(frozen, get_all)]
    struct EvaluationResultWithDetails {
        variation: Py<PyAny>,
        action: Option<Py<PyString>>,
        evaluation_details: Py<PyAny>,
    }
    #[pymethods]
    impl EvaluationResultWithDetails {
        fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
            use pyo3::types::PyList;

            let pieces = PyList::new_bound(
                py,
                [
                    intern!(py, "EvaluationResultWithDetails(variation=").clone(),
                    self.variation.bind(py).repr()?,
                    intern!(py, ", action=").clone(),
                    self.action.to_object(py).into_bound(py).repr()?,
                    intern!(py, ", evaluation_details=").clone(),
                    self.evaluation_details.bind(py).repr()?,
                    intern!(py, ")").clone(),
                ],
            );
            intern!(py, "").call_method1(intern!(py, "join"), (pieces,))
        }
    }
    impl EvaluationResultWithDetails {
        fn from_core<T: TryToPyObject>(
            py: Python,
            result: eppo_core::eval::eval_details::EvaluationResultWithDetails<T>,
            default: Py<PyAny>,
        ) -> PyResult<EvaluationResultWithDetails> {
            let eppo_core::eval::eval_details::EvaluationResultWithDetails {
                variation,
                action,
                evaluation_details,
            } = result;

            let variation = if let Some(variation) = variation {
                variation.try_to_pyobject(py)?
            } else {
                default
            };

            Ok(EvaluationResultWithDetails {
                variation,
                action: action.map(|it| PyString::new_bound(py, &it).unbind()),
                evaluation_details: evaluation_details.try_to_pyobject(py)?,
            })
        }
    }

    #[pyclass(frozen)]
    struct EppoClient {
        configuration_store: Arc<ConfigurationStore>,
        poller_thread: PollerThread,
        assignment_logger: Py<AssignmentLogger>,
    }

    #[pymethods]
    impl EppoClient {
        fn get_string_assignment(
            slf: &Bound<EppoClient>,
            flag_key: &str,
            subject_key: &str,
            subject_attributes: Attributes,
            default: Py<PyString>,
        ) -> PyResult<PyObject> {
            slf.get().get_assignment(
                slf.py(),
                flag_key,
                subject_key,
                subject_attributes,
                Some(VariationType::String),
                default.into_any(),
            )
        }
        fn get_integer_assignment(
            slf: &Bound<EppoClient>,
            flag_key: &str,
            subject_key: &str,
            subject_attributes: Attributes,
            default: Py<PyInt>,
        ) -> PyResult<PyObject> {
            slf.get().get_assignment(
                slf.py(),
                flag_key,
                subject_key,
                subject_attributes,
                Some(VariationType::Integer),
                default.into_any(),
            )
        }
        fn get_numeric_assignment(
            slf: &Bound<EppoClient>,
            flag_key: &str,
            subject_key: &str,
            subject_attributes: Attributes,
            default: Py<PyFloat>,
        ) -> PyResult<PyObject> {
            slf.get().get_assignment(
                slf.py(),
                flag_key,
                subject_key,
                subject_attributes,
                Some(VariationType::Numeric),
                default.into_any(),
            )
        }
        fn get_boolean_assignment(
            slf: &Bound<EppoClient>,
            flag_key: &str,
            subject_key: &str,
            subject_attributes: Attributes,
            default: Py<PyBool>,
        ) -> PyResult<PyObject> {
            slf.get().get_assignment(
                slf.py(),
                flag_key,
                subject_key,
                subject_attributes,
                Some(VariationType::Boolean),
                default.into_any(),
            )
        }
        fn get_json_assignment(
            slf: &Bound<EppoClient>,
            flag_key: &str,
            subject_key: &str,
            subject_attributes: Attributes,
            default: PyObject,
        ) -> PyResult<PyObject> {
            slf.get().get_assignment(
                slf.py(),
                flag_key,
                subject_key,
                subject_attributes,
                Some(VariationType::Json),
                default.into_any(),
            )
        }

        fn get_string_assignment_details(
            slf: &Bound<EppoClient>,
            flag_key: &str,
            subject_key: &str,
            subject_attributes: Attributes,
            default: Py<PyString>,
        ) -> PyResult<EvaluationResultWithDetails> {
            slf.get().get_assignment_details(
                slf.py(),
                flag_key,
                subject_key,
                subject_attributes,
                Some(VariationType::String),
                default.into_any(),
            )
        }
        fn get_integer_assignment_details(
            slf: &Bound<EppoClient>,
            flag_key: &str,
            subject_key: &str,
            subject_attributes: Attributes,
            default: Py<PyInt>,
        ) -> PyResult<EvaluationResultWithDetails> {
            slf.get().get_assignment_details(
                slf.py(),
                flag_key,
                subject_key,
                subject_attributes,
                Some(VariationType::Integer),
                default.into_any(),
            )
        }
        fn get_numeric_assignment_details(
            slf: &Bound<EppoClient>,
            flag_key: &str,
            subject_key: &str,
            subject_attributes: Attributes,
            default: Py<PyFloat>,
        ) -> PyResult<EvaluationResultWithDetails> {
            slf.get().get_assignment_details(
                slf.py(),
                flag_key,
                subject_key,
                subject_attributes,
                Some(VariationType::Numeric),
                default.into_any(),
            )
        }
        fn get_boolean_assignment_details(
            slf: &Bound<EppoClient>,
            flag_key: &str,
            subject_key: &str,
            subject_attributes: Attributes,
            default: Py<PyBool>,
        ) -> PyResult<EvaluationResultWithDetails> {
            slf.get().get_assignment_details(
                slf.py(),
                flag_key,
                subject_key,
                subject_attributes,
                Some(VariationType::Boolean),
                default.into_any(),
            )
        }
        fn get_json_assignment_details(
            slf: &Bound<EppoClient>,
            flag_key: &str,
            subject_key: &str,
            subject_attributes: Attributes,
            default: Py<PyAny>,
        ) -> PyResult<EvaluationResultWithDetails> {
            slf.get().get_assignment_details(
                slf.py(),
                flag_key,
                subject_key,
                subject_attributes,
                Some(VariationType::Json),
                default.into_any(),
            )
        }

        // Implementing [Garbage Collector integration][1] in case user's `AssignmentLogger` holds a
        // reference to `EppoClient`. This will allow the GC to detect this cycle and break it.
        //
        // [1]: https://pyo3.rs/v0.22.2/class/protocols.html#garbage-collector-integration
        fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {
            visit.call(&self.assignment_logger)
        }
        fn __clear__(&self) {
            // We're frozen and don't hold mutable Python references, so there's nothing to clear.
        }
    }

    // Rust-only methods
    impl EppoClient {
        fn new(py: Python, config: &Config) -> PyResult<EppoClient> {
            let configuration_store = Arc::new(ConfigurationStore::new());
            let poller_thread = PollerThread::start_with_config(
                ConfigurationFetcher::new(
                    eppo_core::configuration_fetcher::ConfigurationFetcherConfig {
                        base_url: config.base_url.clone(),
                        api_key: config.api_key.clone(),
                        sdk_name: "python".to_owned(),
                        sdk_version: env!("CARGO_PKG_VERSION").to_owned(),
                    },
                ),
                configuration_store.clone(),
                PollerThreadConfig {
                    interval: Duration::from_secs(config.poll_interval_seconds),
                    jitter: Duration::from_secs(config.poll_jitter_seconds),
                },
            )
            .map_err(|err| {
                // This should normally never happen.
                PyRuntimeError::new_err(format!("unable to start poller thread: {err}"))
            })?;
            Ok(EppoClient {
                configuration_store,
                poller_thread,
                assignment_logger: config
                    .assignment_logger
                    .as_ref()
                    .ok_or_else(|| {
                        // This should never happen as assigment_logger setter requires a valid
                        // logger.
                        PyRuntimeError::new_err(format!("Config.assignment_logger is None"))
                    })?
                    .clone_ref(py),
            })
        }

        fn get_assignment(
            &self,
            py: Python,
            flag_key: &str,
            subject_key: &str,
            subject_attributes: Attributes,
            expected_type: Option<VariationType>,
            default: Py<PyAny>,
        ) -> PyResult<PyObject> {
            let config = self.configuration_store.get_configuration();

            let result = get_assignment(
                config.as_ref().map(AsRef::as_ref),
                &flag_key,
                &subject_key,
                &subject_attributes,
                expected_type,
            );

            let assignment = match result {
                Ok(assignment) => assignment,
                Err(err) => {
                    let graceful_mode = true;
                    if graceful_mode {
                        None
                    } else {
                        return Err(PyErr::new::<PyRuntimeError, _>(err.to_string()));
                    }
                }
            };

            if let Some(assignment) = assignment {
                if let Some(event) = assignment.event {
                    if let Err(err) = self.log_assignment_event(py, event) {
                        log::warn!(target: "eppo", "error logging assignment event: {err}")
                    }
                }

                Ok(assignment.value.try_to_pyobject(py)?)
            } else {
                Ok(default)
            }
        }

        fn get_assignment_details(
            &self,
            py: Python,
            flag_key: &str,
            subject_key: &str,
            subject_attributes: Attributes,
            expected_type: Option<VariationType>,
            default: Py<PyAny>,
        ) -> PyResult<EvaluationResultWithDetails> {
            let config = self.configuration_store.get_configuration();

            let (result, event) = get_assignment_details(
                config.as_ref().map(AsRef::as_ref),
                &flag_key,
                &subject_key,
                &subject_attributes,
                expected_type,
            );

            if let Some(event) = event {
                if let Err(err) = self.log_assignment_event(py, event) {
                    log::warn!(target: "eppo", "error logging assignment event: {err}")
                }
            }

            EvaluationResultWithDetails::from_core(py, result, default)
        }

        /// Try to log assignment event using `self.assignment_logger`.
        fn log_assignment_event(&self, py: Python, event: AssignmentEvent) -> PyResult<()> {
            let event = event.try_to_pyobject(py)?;
            self.assignment_logger
                .call_method1(py, intern!(py, "log_assignment"), (event,))?;
            Ok(())
        }

        fn shutdown(&self) {
            // Using `.stop()` instead of `.shutdown()` here because we don't need to wait for the
            // poller thread to exit.
            self.poller_thread.stop();
        }
    }

    impl Drop for EppoClient {
        fn drop(&mut self) {
            self.shutdown();
        }
    }

    /// Initializes a global Eppo client instance.
    ///
    /// This method should be called once on application startup.
    /// If invoked more than once, it will re-initialize the global client instance.
    /// Use the :func:`EppoClient.get_instance()` method to access the client instance.
    ///
    /// :param config: client configuration containing the API Key
    /// :type config: Config
    #[pyfunction]
    fn init(config: Bound<Config>) -> PyResult<Py<EppoClient>> {
        initialize_pyo3_log();

        let py = config.py();

        let client = Bound::new(py, EppoClient::new(py, &*config.borrow())?)?.unbind();

        // minimizing the scope of holding the write lock
        let existing = {
            let client = Py::clone_ref(&client, py);

            let mut instance = CLIENT_INSTANCE.write().map_err(|err| {
                // This should normally never happen as it signifies that another thread
                // panicked while holding the lock.
                PyException::new_err(format!("failed to acquire writer lock: {err}"))
            })?;
            std::mem::replace(&mut *instance, Some(client))
        };
        if let Some(existing) = existing {
            existing.get().shutdown();
            existing.drop_ref(py);
        }

        Ok(client)
    }

    /// Used to access an initialized client instance.
    ///
    /// Use this method to get a client instance for assigning variants.
    /// This method may only be called after invocation of :func:`eppo_client.init()`, otherwise it
    /// throws an exception.
    ///
    /// :return: a shared client instance
    /// :rtype: EppoClient
    #[pyfunction]
    fn get_instance(py: Python) -> PyResult<Py<EppoClient>> {
        let instance = CLIENT_INSTANCE.read().map_err(|err| {
            // This should normally never happen as it signifies that another thread
            // panicked while holding the lock.
            PyException::new_err(format!("failed to acquire reader lock: {err}"))
        })?;
        if let Some(existing) = &*instance {
            Ok(Py::clone_ref(existing, py))
        } else {
            Err(PyException::new_err(
                "init() must be called before get_instance()",
            ))
        }
    }
}

/// Initialize `pyo3_log` crate connecting Rust's `log` to Python's `logger`.
///
/// If called multiple times, resets the pyo3_log cache.
fn initialize_pyo3_log() {
    static LOG_RESET_HANDLE: Mutex<Option<pyo3_log::ResetHandle>> = Mutex::new(None);
    {
        if let Ok(mut reset_handle) = LOG_RESET_HANDLE.lock() {
            if let Some(previous_handle) = &mut *reset_handle {
                // There's a previous handle. Logging is already initialized, but we reset
                // caches.
                previous_handle.reset();
            } else {
                if let Ok(new_handle) = pyo3_log::try_init() {
                    *reset_handle = Some(new_handle);
                } else {
                    // This should not happen as initialization error signals that we already
                    // initialized logging. (In which case, `LOG_RESET_HANDLE` should contain
                    // `Some()`.)
                    debug_assert!(false, "tried to initialize pyo3_log second time");
                }
            }
        } else {
            // This should normally never happen as it shows that another thread has panicked
            // while holding `LOG_RESET_HANDLE`.
            //
            // That's probably not sever enough to throw an exception into user's code.
            debug_assert!(false, "failed to acquire LOG_RESET_HANDLE lock");
        }
    }
}
