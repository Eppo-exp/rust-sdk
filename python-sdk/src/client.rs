use std::{
    collections::HashMap,
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError},
    intern,
    prelude::*,
    types::{PyBool, PyFloat, PyInt, PyString},
    PyTraverseError, PyVisit,
};

use eppo_core::{
    configuration_fetcher::ConfigurationFetcher,
    configuration_store::ConfigurationStore,
    eval::{
        eval_details::EvaluationResultWithDetails, get_assignment, get_assignment_details,
        get_bandit_action, BanditResult,
    },
    events::{AssignmentEvent, BanditEvent},
    poller_thread::{PollerThread, PollerThreadConfig},
    pyo3::TryToPyObject,
    ufc::VariationType,
    Attributes, ContextAttributes,
};

use crate::{assignment_logger::AssignmentLogger, config::Config};

#[pyclass(frozen, get_all, module = "eppo_client")]
pub struct EvaluationResult {
    variation: Py<PyAny>,
    action: Option<Py<PyString>>,
    /// Optional evaluation details.
    evaluation_details: Option<Py<PyAny>>,
}
#[pymethods]
impl EvaluationResult {
    #[new]
    #[pyo3(signature = (variation, action=None, evaluation_details=None))]
    fn new(
        variation: Py<PyAny>,
        action: Option<Py<PyString>>,
        evaluation_details: Option<Py<PyAny>>,
    ) -> EvaluationResult {
        EvaluationResult {
            variation,
            action,
            evaluation_details,
        }
    }

    fn to_string(&self, py: Python) -> PyResult<Py<PyString>> {
        // use pyo3::types::PyAnyMethods;
        let s = if let Some(action) = &self.action {
            action.clone_ref(py)
        } else {
            self.variation.bind(py).str()?.unbind()
        };
        Ok(s)
    }

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
                self.evaluation_details
                    .to_object(py)
                    .into_bound(py)
                    .repr()?,
                intern!(py, ")").clone(),
            ],
        );
        intern!(py, "").call_method1(intern!(py, "join"), (pieces,))
    }
}
impl EvaluationResult {
    fn from_details<T: TryToPyObject>(
        py: Python,
        result: EvaluationResultWithDetails<T>,
        default: Py<PyAny>,
    ) -> PyResult<EvaluationResult> {
        let EvaluationResultWithDetails {
            variation,
            action,
            evaluation_details,
        } = result;

        let variation = if let Some(variation) = variation {
            variation.try_to_pyobject(py)?
        } else {
            default
        };

        Ok(EvaluationResult {
            variation,
            action: action.map(|it| PyString::new_bound(py, &it).unbind()),
            evaluation_details: Some(evaluation_details.try_to_pyobject(py)?),
        })
    }

    fn from_bandit_result(py: Python, result: BanditResult) -> EvaluationResult {
        let variation = result.variation.into_py(py);
        let action = result
            .action
            .map(|it| PyString::new_bound(py, &it).unbind());

        EvaluationResult {
            variation,
            action,
            evaluation_details: None,
        }
    }
}

#[pyclass(frozen, module = "eppo_client")]
pub struct EppoClient {
    configuration_store: Arc<ConfigurationStore>,
    poller_thread: PollerThread,
    assignment_logger: Py<AssignmentLogger>,
    is_graceful_mode: AtomicBool,
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
    ) -> PyResult<EvaluationResult> {
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
    ) -> PyResult<EvaluationResult> {
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
    ) -> PyResult<EvaluationResult> {
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
    ) -> PyResult<EvaluationResult> {
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
    ) -> PyResult<EvaluationResult> {
        slf.get().get_assignment_details(
            slf.py(),
            flag_key,
            subject_key,
            subject_attributes,
            Some(VariationType::Json),
            default.into_any(),
        )
    }

    fn get_bandit_action(
        slf: &Bound<EppoClient>,
        flag_key: &str,
        subject_key: &str,
        #[pyo3(from_py_with = "context_attributes_from_py")] subject_attributes: RefOrOwned<
            ContextAttributes,
            PyRef<ContextAttributes>,
        >,
        #[pyo3(from_py_with = "actions_from_py")] actions: HashMap<String, ContextAttributes>,
        default: &str,
    ) -> PyResult<EvaluationResult> {
        let py = slf.py();
        let this = slf.get();
        let configuration = this.configuration_store.get_configuration();

        let mut result = get_bandit_action(
            configuration.as_ref().map(|it| it.as_ref()),
            flag_key,
            subject_key,
            &subject_attributes,
            &actions,
            default,
        );

        if let Some(event) = result.assignment_event.take() {
            let _ = this.log_assignment_event(py, event);
        }
        if let Some(event) = result.bandit_event.take() {
            let _ = this.log_bandit_event(py, event);
        }

        Ok(EvaluationResult::from_bandit_result(py, result))
    }

    fn set_is_graceful_mode(&self, is_graceful_mode: bool) {
        self.is_graceful_mode
            .store(is_graceful_mode, Ordering::Release);
    }

    // Returns True if the client has successfully initialized the flag configuration and is ready
    // to serve requests.
    fn is_initialized(&self) -> bool {
        let config = self.configuration_store.get_configuration();
        config.is_some()
    }

    /// Wait for configuration to get fetches.
    ///
    /// This method releases GIL, so other Python thread can make progress.
    fn wait_for_initialization(&self, py: Python) -> PyResult<()> {
        py.allow_threads(|| self.poller_thread.wait_for_configuration())
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))
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

#[derive(Debug, Clone, Copy)]
enum RefOrOwned<T, Ref>
where
    Ref: Deref<Target = T>,
{
    Ref(Ref),
    Owned(T),
}
impl<T, Ref> Deref for RefOrOwned<T, Ref>
where
    Ref: Deref<Target = T>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            RefOrOwned::Ref(r) => r,
            RefOrOwned::Owned(owned) => owned,
        }
    }
}

fn context_attributes_from_py<'py>(
    obj: &'py Bound<'py, PyAny>,
) -> PyResult<RefOrOwned<ContextAttributes, PyRef<'py, ContextAttributes>>> {
    if let Ok(attrs) = obj.downcast::<ContextAttributes>() {
        return Ok(RefOrOwned::Ref(attrs.borrow()));
    }
    if let Ok(attrs) = Attributes::extract_bound(obj) {
        return Ok(RefOrOwned::Owned(attrs.into()));
    }
    Err(PyTypeError::new_err(format!(
        "attributes must be either ContextAttributes or Attributes"
    )))
}

fn actions_from_py(obj: &Bound<PyAny>) -> PyResult<HashMap<String, ContextAttributes>> {
    if let Ok(result) = FromPyObject::extract_bound(&obj) {
        return Ok(result);
    }

    if let Ok(result) = HashMap::<String, Attributes>::extract_bound(&obj) {
        let result = result
            .into_iter()
            .map(|(name, attrs)| (name, ContextAttributes::from(attrs)))
            .collect();
        return Ok(result);
    }

    Err(PyTypeError::new_err(format!(
        "action attributes must be either ContextAttributes or Attributes"
    )))
}

// Rust-only methods
impl EppoClient {
    pub fn new(py: Python, config: &Config) -> PyResult<EppoClient> {
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
                    // This should never happen as assigment_logger setter requires a valid logger.
                    PyRuntimeError::new_err(format!("Config.assignment_logger is None"))
                })?
                .clone_ref(py),
            is_graceful_mode: AtomicBool::new(config.is_graceful_mode),
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
                if self.is_graceful_mode.load(Ordering::Acquire) {
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
    ) -> PyResult<EvaluationResult> {
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

        EvaluationResult::from_details(py, result, default)
    }

    /// Try to log assignment event using `self.assignment_logger`.
    pub fn log_assignment_event(&self, py: Python, mut event: AssignmentEvent) -> PyResult<()> {
        event.add_sdk_metadata("python".to_owned(), env!("CARGO_PKG_VERSION").to_owned());
        let event = event.try_to_pyobject(py)?;
        self.assignment_logger
            .call_method1(py, intern!(py, "log_assignment"), (event,))?;
        Ok(())
    }

    /// Try to log bandit event using `self.assignment_logger`.
    pub fn log_bandit_event(&self, py: Python, mut event: BanditEvent) -> PyResult<()> {
        event.add_sdk_metadata("python".to_owned(), env!("CARGO_PKG_VERSION").to_owned());
        let event = event.try_to_pyobject(py)?;
        self.assignment_logger
            .call_method1(py, intern!(py, "log_bandit_action"), (event,))?;
        Ok(())
    }

    pub fn shutdown(&self) {
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
