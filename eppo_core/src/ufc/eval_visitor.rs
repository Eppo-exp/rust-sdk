use crate::Configuration;

use super::{
    eval::AllocationNonMatchReason, Allocation, Assignment, Flag, FlagEvaluationError, Split,
    Variation,
};

pub(super) trait EvalVisitor {
    // Type-foo here basically means that AllocationVisitor may hold references to EvalFlagVisitor
    // but should not outlive it.
    type AllocationVisitor<'a>: EvalAllocationVisitor + 'a
    where
        Self: 'a;

    /// Called when (if) evaluation gets configuration.
    #[allow(unused_variables)]
    #[inline]
    fn on_configuration(&mut self, configuration: &Configuration) {}

    #[allow(unused_variables)]
    #[inline]
    fn on_flag_configuration(&mut self, flag: &Flag) {}

    #[allow(unused_variables)]
    #[inline]
    fn on_variation(&mut self, variation: &Variation) {}

    fn visit_allocation<'a>(&'a mut self, allocation: &Allocation) -> Self::AllocationVisitor<'a>;

    /// Called with evaluation result.
    #[allow(unused_variables)]
    #[inline]
    fn on_result(&mut self, result: &Result<Assignment, FlagEvaluationError>) {}
}

pub(super) trait EvalAllocationVisitor {
    #[allow(unused_variables)]
    #[inline]
    fn on_result(&mut self, result: Result<&Split, AllocationNonMatchReason>) {}
}

/// Dummy visitor that does nothing.
///
/// It is designed so that all calls to it are optimized away (zero-cost).
pub(super) struct NoopEvalVisitor;

impl EvalVisitor for NoopEvalVisitor {
    type AllocationVisitor<'a> = NoopEvalVisitor;

    #[inline]
    fn visit_allocation<'a>(&'a mut self, _allocation: &Allocation) -> Self::AllocationVisitor<'a> {
        NoopEvalVisitor
    }
}

impl EvalAllocationVisitor for NoopEvalVisitor {}
