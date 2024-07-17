use super::{Assignment, FlagEvaluationError, UniversalFlagConfig};

pub(crate) trait EvalVisitor {
    /// Called when (if) evaluation gets configuration.
    #[allow(unused_variables)]
    #[inline]
    fn on_configuration(&mut self, configuration: &UniversalFlagConfig) {}

    /// Called with evaluation result.
    #[allow(unused_variables)]
    #[inline]
    fn on_result(&mut self, result: &Result<Assignment, FlagEvaluationError>) {}
}

/// Dummy visitor that does nothing.
///
/// It is designed so that all calls to it are optimized away (zero-cost).
pub(crate) struct NoopEvalVisitor;

impl EvalVisitor for NoopEvalVisitor {}
