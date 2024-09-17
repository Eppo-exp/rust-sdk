use pyo3::prelude::*;
use pyo3::types::PyDict;

#[derive(Debug, Clone)]
#[pyclass(frozen, subclass, module = "eppo_client")]
pub struct AssignmentLogger {}

#[pymethods]
impl AssignmentLogger {
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    #[allow(unused_variables)]
    fn new(args: &Bound<'_, PyAny>, kwargs: Option<&Bound<'_, PyAny>>) -> AssignmentLogger {
        AssignmentLogger {}
    }

    #[allow(unused_variables)]
    fn log_assignment(slf: Bound<Self>, event: Bound<PyDict>) {}

    #[allow(unused_variables)]
    fn log_bandit_action(slf: Bound<Self>, event: Bound<PyDict>) {}
}
