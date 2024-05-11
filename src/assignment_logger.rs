use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::SubjectAttributes;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentEvent {
    pub feature_flag: String,
    pub allocation: String,
    pub experiment: String,
    pub variation: String,
    pub subject: String,
    pub subject_attributes: SubjectAttributes,
    pub timestamp: String,
    pub meta_data: HashMap<String, String>,
    #[serde(flatten)]
    pub extra_logging: HashMap<String, String>,
}

pub trait AssignmentLogger {
    fn log_assignment(&self, event: AssignmentEvent);
}

pub(crate) struct NoopAssignmentLogger;
impl AssignmentLogger for NoopAssignmentLogger {
    fn log_assignment(&self, _event: AssignmentEvent) {}
}

impl<T: Fn(AssignmentEvent)> AssignmentLogger for T {
    fn log_assignment(&self, event: AssignmentEvent) {
        self(event);
    }
}
