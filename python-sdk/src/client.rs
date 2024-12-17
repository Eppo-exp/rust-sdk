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
    types::{PyBool, PyFloat, PyInt, PySet, PyString},
    PyTraverseError, PyVisit,
};

use eppo_core::{
    configuration_fetcher::ConfigurationFetcher,
    configuration_store::ConfigurationStore,
    eval::{
        eval_details::{EvaluationDetails, EvaluationResultWithDetails},
        BanditResult, Evaluator, EvaluatorConfig,
    },
    events::{AssignmentEvent, BanditEvent},
    poller_thread::{PollerThread, PollerThreadConfig},
    pyo3::TryToPyObject,
    ufc::VariationType,
    Attributes, ContextAttributes, Str,
};

use crate::{
    assignment_logger::AssignmentLogger, client_config::ClientConfig, configuration::Configuration,
    SDK_METADATA,
};

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
                intern!(py, "EvaluationResult(variation=").clone(),
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

    fn from_bandit_result(
        py: Python,
        result: BanditResult,
        details: Option<EvaluationDetails>,
    ) -> PyResult<EvaluationResult> {
        let variation = result.variation.into_py(py);
        let action = result
            .action
            .map(|it| PyString::new_bound(py, &it).unbind());

        let evaluation_details = if let Some(details) = details {
            Some(details.try_to_pyobject(py)?)
        } else {
            None
        };

        Ok(EvaluationResult {
            variation,
            action,
            evaluation_details,
        })
    }
}

#[pyclass(frozen, module = "eppo_client")]
pub struct EppoClient {
    configuration_store: Arc<ConfigurationStore>,
    evaluator: Evaluator,
    poller_thread: Option<PollerThread>,
    assignment_logger: Py<AssignmentLogger>,
    is_graceful_mode: AtomicBool,
}

#[pymethods]
impl EppoClient {
    fn get_string_assignment(
        slf: &Bound<EppoClient>,
        flag_key: &str,
        subject_key: Str,
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
        subject_key: Str,
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
        subject_key: Str,
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
        subject_key: Str,
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
        subject_key: Str,
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
        subject_key: Str,
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
        subject_key: Str,
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
        subject_key: Str,
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
        subject_key: Str,
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
        subject_key: Str,
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

    /// Determines the bandit action for a given subject based on the provided bandit key and subject attributes.
    ///
    /// This method performs the following steps:
    /// 1. Retrieves the experiment assignment for the given bandit key and subject.
    /// 2. Checks if the assignment matches the bandit key. If not, it means the subject is not allocated in the bandit,
    ///    and the method returns a EvaluationResult with the assignment.
    /// 3. Evaluates the bandit action using the bandit evaluator.
    /// 4. Logs the bandit action event.
    /// 5. Returns the EvaluationResult containing the selected action key and the assignment.
    ///
    /// Args:
    ///     flag_key (str): The feature flag key that contains the bandit as one of the variations.
    ///     subject_key (str): The key identifying the subject.
    ///     subject_context (Union[ContextAttributes, Attributes]): The subject context.
    ///         If supplying an ActionAttributes, it gets converted to an ActionContexts instance
    ///     actions (Union[ActionContexts, ActionAttributes]): The dictionary that maps action keys
    ///         to their context of actions with their contexts.
    ///         If supplying an ActionAttributes, it gets converted to an ActionContexts instance.
    ///     default (str): The default variation to use if an error is encountered retrieving the
    ///         assigned variation.
    ///
    /// Returns:
    ///     EvaluationResult: The result containing either the bandit action if the subject is part of the bandit,
    ///                   or the assignment if they are not. The EvaluationResult includes:
    ///                   - variation (str): The assignment key indicating the subject's variation.
    ///                   - action (Optional[str]): The key of the selected action if the subject was assigned one
    ///                     by the bandit.
    ///
    /// Example:
    /// result = client.get_bandit_action(
    ///     "flag_key",
    ///     "subject_key",
    ///     ContextAttributes(
    ///         numeric_attributes={"age": 25},
    ///         categorical_attributes={"country": "USA"}),
    ///     {
    ///         "action1": ContextAttributes(
    ///             numeric_attributes={"price": 10.0},
    ///             categorical_attributes={"category": "A"}
    ///         ),
    ///         "action2": {"price": 10.0, "category": "B"}
    ///         "action3": ContextAttributes.empty(),
    ///     },
    ///     "default"
    /// )
    /// if result.action:
    ///     do_action(result.variation)
    /// else:
    ///     do_status_quo()
    fn get_bandit_action(
        slf: &Bound<EppoClient>,
        flag_key: &str,
        subject_key: Str,
        #[pyo3(from_py_with = "context_attributes_from_py")] subject_context: RefOrOwned<
            ContextAttributes,
            PyRef<ContextAttributes>,
        >,
        #[pyo3(from_py_with = "actions_from_py")] actions: HashMap<Str, ContextAttributes>,
        default: Str,
    ) -> PyResult<EvaluationResult> {
        let py = slf.py();
        let this = slf.get();

        let mut result = this.evaluator.get_bandit_action(
            flag_key,
            &subject_key,
            &subject_context,
            &actions,
            &default,
        );

        if let Some(event) = result.assignment_event.take() {
            let _ = this.log_assignment_event(py, event);
        }
        if let Some(event) = result.bandit_event.take() {
            let _ = this.log_bandit_event(py, event);
        }

        EvaluationResult::from_bandit_result(py, result, None)
    }

    /// Same as get_bandit_action() but returns EvaluationResult with evaluation_details.
    fn get_bandit_action_details(
        slf: &Bound<EppoClient>,
        flag_key: &str,
        subject_key: Str,
        #[pyo3(from_py_with = "context_attributes_from_py")] subject_context: RefOrOwned<
            ContextAttributes,
            PyRef<ContextAttributes>,
        >,
        #[pyo3(from_py_with = "actions_from_py")] actions: HashMap<Str, ContextAttributes>,
        default: Str,
    ) -> PyResult<EvaluationResult> {
        let py = slf.py();
        let this = slf.get();

        let (mut result, details) = this.evaluator.get_bandit_action_details(
            flag_key,
            &subject_key,
            &subject_context,
            &actions,
            &default,
        );

        if let Some(event) = result.assignment_event.take() {
            let _ = this.log_assignment_event(py, event);
        }
        if let Some(event) = result.bandit_event.take() {
            let _ = this.log_bandit_event(py, event);
        }

        EvaluationResult::from_bandit_result(py, result, Some(details))
    }

    fn get_configuration(&self) -> Option<Configuration> {
        self.configuration_store
            .get_configuration()
            .map(Configuration::new)
    }

    fn set_configuration(&self, configuration: &Configuration) {
        self.configuration_store
            .set_configuration(Arc::clone(&configuration.configuration));
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
        if let Some(poller) = &self.poller_thread {
            py.allow_threads(|| poller.wait_for_configuration())
                .map_err(|err| PyRuntimeError::new_err(err.to_string()))
        } else {
            Err(PyRuntimeError::new_err("poller is disabled"))
        }
    }

    /// Returns a set of all flag keys that have been initialized.
    /// This can be useful to debug the initialization process.
    ///
    /// Deprecated. Use EppoClient.get_configuration() instead.
    fn get_flag_keys<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<PySet>> {
        let config = self.configuration_store.get_configuration();
        match config {
            Some(config) => PySet::new_bound(py, &config.flag_keys()),
            None => PySet::empty_bound(py),
        }
    }

    /// Returns a set of all bandit keys that have been initialized.
    /// This can be useful to debug the initialization process.
    ///
    /// Deprecated. Use EppoClient.get_configuration() instead.
    fn get_bandit_keys<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<PySet>> {
        let config = self.configuration_store.get_configuration();
        match config {
            Some(config) => {
                PySet::new_bound(py, config.bandits.iter().flat_map(|it| it.bandits.keys()))
            }
            None => PySet::empty_bound(py),
        }
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

fn actions_from_py(obj: &Bound<PyAny>) -> PyResult<HashMap<Str, ContextAttributes>> {
    if let Ok(result) = FromPyObject::extract_bound(&obj) {
        return Ok(result);
    }

    if let Ok(result) = HashMap::<Str, Attributes>::extract_bound(&obj) {
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
    pub fn new(py: Python, config: &ClientConfig) -> PyResult<EppoClient> {
        let configuration_store = Arc::new(ConfigurationStore::new());
        if let Some(configuration) = &config.initial_configuration {
            let configuration = Arc::clone(&configuration.get().configuration);
            configuration_store.set_configuration(configuration);
        }

        let evaluator = Evaluator::new(EvaluatorConfig {
            configuration_store: configuration_store.clone(),
            sdk_metadata: SDK_METADATA,
        });

        let poller_thread = config
            .poll_interval_seconds
            .map(|poll_interval_seconds| {
                PollerThread::start_with_config(
                    ConfigurationFetcher::new(
                        eppo_core::configuration_fetcher::ConfigurationFetcherConfig {
                            base_url: config.base_url.clone(),
                            api_key: config.api_key.clone(),
                            sdk_metadata: SDK_METADATA,
                        },
                    ),
                    configuration_store.clone(),
                    PollerThreadConfig {
                        interval: Duration::from_secs(poll_interval_seconds.into()),
                        jitter: Duration::from_secs(config.poll_jitter_seconds),
                    },
                )
            })
            .transpose()
            .map_err(|err| {
                // This should normally never happen.
                PyRuntimeError::new_err(format!("unable to start poller thread: {err}"))
            })?;

        Ok(EppoClient {
            configuration_store,
            evaluator,
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
        subject_key: Str,
        subject_attributes: Attributes,
        expected_type: Option<VariationType>,
        default: Py<PyAny>,
    ) -> PyResult<PyObject> {
        let result = self.evaluator.get_assignment(
            &flag_key,
            &subject_key.into(),
            &subject_attributes.into(),
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
        subject_key: Str,
        subject_attributes: Attributes,
        expected_type: Option<VariationType>,
        default: Py<PyAny>,
    ) -> PyResult<EvaluationResult> {
        let (result, event) = self.evaluator.get_assignment_details(
            &flag_key,
            &subject_key.into(),
            &subject_attributes.into(),
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
    pub fn log_assignment_event(&self, py: Python, event: AssignmentEvent) -> PyResult<()> {
        let event = event.try_to_pyobject(py)?;
        self.assignment_logger
            .call_method1(py, intern!(py, "log_assignment"), (event,))?;
        Ok(())
    }

    /// Try to log bandit event using `self.assignment_logger`.
    pub fn log_bandit_event(&self, py: Python, event: BanditEvent) -> PyResult<()> {
        let event = event.try_to_pyobject(py)?;
        self.assignment_logger
            .call_method1(py, intern!(py, "log_bandit_action"), (event,))?;
        Ok(())
    }

    pub fn shutdown(&self) {
        if let Some(poller) = &self.poller_thread {
            // Using `.stop()` instead of `.shutdown()` here because we don't need to wait for the
            // poller thread to exit.
            poller.stop();
        }
    }
}

impl Drop for EppoClient {
    fn drop(&mut self) {
        self.shutdown();
    }
}
