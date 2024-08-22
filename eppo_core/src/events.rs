use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{eval::eval_details::EvaluationDetails, Attributes};

/// Events that can be emitted during evaluation of assignment or bandit. They need to be logged to
/// analytics storage and fed back to Eppo for analysis.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Events {
    pub assignment: Option<AssignmentEvent>,
    pub bandit: Option<BanditEvent>,
}

/// Represents an event capturing the assignment of a feature flag to a subject and its logging
/// details.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentEvent {
    /// The key of the feature flag being assigned.
    pub feature_flag: String,
    /// The key of the allocation that the subject was assigned to.
    pub allocation: String,
    /// The key of the experiment associated with the assignment.
    pub experiment: String,
    /// The specific variation assigned to the subject.
    pub variation: String,
    /// The key identifying the subject receiving the assignment.
    pub subject: String,
    /// Custom attributes of the subject relevant to the assignment.
    pub subject_attributes: Attributes,
    /// The timestamp indicating when the assignment event occurred.
    pub timestamp: String,
    /// Additional metadata such as SDK language and version.
    pub meta_data: HashMap<String, String>,
    /// Additional user-defined logging fields for capturing extra information related to the
    /// assignment.
    #[serde(flatten)]
    pub extra_logging: HashMap<String, String>,
    /// Evaluation details that could help with debugging the assigment. Only populated when
    /// details-version of the `get_assigment` was called.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluation_details: Option<EvaluationDetails>,
}

/// Bandit evaluation event that needs to be logged to analytics storage.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BanditEvent {
    pub flag_key: String,
    pub bandit_key: String,
    pub subject: String,
    pub action: String,
    pub action_probability: f64,
    pub optimality_gap: f64,
    pub model_version: String,
    pub timestamp: String,
    pub subject_numeric_attributes: HashMap<String, f64>,
    pub subject_categorical_attributes: HashMap<String, String>,
    pub action_numeric_attributes: HashMap<String, f64>,
    pub action_categorical_attributes: HashMap<String, String>,
    pub meta_data: HashMap<String, String>,
}

impl AssignmentEvent {
    pub fn add_sdk_metadata(&mut self, name: String, version: String) {
        self.meta_data.insert("sdkName".to_owned(), name);
        self.meta_data.insert("sdkVersion".to_owned(), version);
    }
}

impl BanditEvent {
    pub fn add_sdk_metadata(&mut self, name: String, version: String) {
        self.meta_data.insert("sdkName".to_owned(), name);
        self.meta_data.insert("sdkVersion".to_owned(), version);
    }
}

#[cfg(feature = "pyo3")]
mod pyo3_impl {
    use pyo3::{PyObject, PyResult, Python};

    use crate::pyo3::TryToPyObject;

    use super::{AssignmentEvent, BanditEvent};

    impl TryToPyObject for AssignmentEvent {
        fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject> {
            serde_pyobject::to_pyobject(py, self)
                .map(|it| it.unbind())
                .map_err(|err| err.0)
        }
    }

    impl TryToPyObject for BanditEvent {
        fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject> {
            serde_pyobject::to_pyobject(py, self)
                .map(|it| it.unbind())
                .map_err(|err| err.0)
        }
    }
}
