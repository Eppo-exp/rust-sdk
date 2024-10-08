use crate::AssignmentEvent;

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
    /// ```no_run
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
