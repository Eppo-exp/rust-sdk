use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::SubjectAttributes;

/// Represents an event capturing the assignment of a feature flag to a subject and its logging
/// details.
#[derive(Debug, Serialize, Deserialize)]
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
    pub subject_attributes: SubjectAttributes,
    /// The timestamp indicating when the assignment event occurred.
    pub timestamp: String,
    /// Additional metadata such as SDK language and version.
    pub meta_data: HashMap<String, String>,
    /// Additional user-defined logging fields for capturing extra information related to the
    /// assignment.
    #[serde(flatten)]
    pub extra_logging: HashMap<String, String>,
}

/// A trait for logging assignment events to your storage system. Implementations should handle
/// persisting assignment events for analytics and tracking purposes.
pub trait AssignmentLogger {
    /// Logs the assignment event to the storage system.
    ///
    /// # Arguments
    ///
    /// * `event` - An [`AssignmentEvent`] to be logged.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eppo::{AssignmentLogger, AssignmentEvent};
    /// struct MyAssignmentLogger;
    ///
    /// impl AssignmentLogger for MyAssignmentLogger {
    ///     fn log_assignment(&self, event: AssignmentEvent) {
    ///         // Implement assignment logging logic here
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This method should not return errors and should not panic.
    /// Errors that occur during logging should be handled internally within the implementation.
    ///
    /// # Notes
    ///
    /// This method is called before returning assignment to the caller, so it is important that
    /// `log_assignment` does not block the calling thread to prevent performance implications and
    /// delays in returning assignments.
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
