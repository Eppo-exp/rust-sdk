use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Attributes;

use super::{
    eval::AllocationNonMatchReason, eval_visitor::*, Assignment, FlagEvaluationError, Split,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalFlagDetails {
    pub flag_key: String,
    pub subject_key: String,
    pub subject_attributes: Attributes,
    pub result: Result<Assignment, FlagEvaluationError>,
    pub allocations: Vec<EvalAllocationDetails>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalAllocationDetails {
    pub key: String,
    pub result: EvalAllocationResult,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EvalAllocationResult {
    Unevaluated,
    Matched,
    BeforeStartDate,
    AfterEndDate,
    FailingRules,
    TrafficExposureMiss,
}

pub(crate) struct EvalFlagDetailsBuilder {
    flag_key: String,
    subject_key: String,
    subject_attributes: Attributes,

    result: Option<Result<Assignment, FlagEvaluationError>>,

    /// List of allocation keys. Used to sort `allocation_eval_results`.
    allocation_keys_order: Vec<String>,
    allocation_eval_results: HashMap<String, EvalAllocationDetails>,
}

pub(crate) struct EvalAllocationDetailsBuilder<'a> {
    result: &'a mut EvalAllocationDetails,
}

impl EvalFlagDetailsBuilder {
    pub fn new(
        flag_key: String,
        subject_key: String,
        subject_attributes: Attributes,
    ) -> EvalFlagDetailsBuilder {
        EvalFlagDetailsBuilder {
            flag_key,
            subject_key,
            subject_attributes,
            result: None,
            allocation_keys_order: Vec::new(),
            allocation_eval_results: HashMap::new(),
        }
    }

    pub fn build(mut self) -> EvalFlagDetails {
        EvalFlagDetails {
            flag_key: self.flag_key,
            subject_key: self.subject_key,
            subject_attributes: self.subject_attributes,
            result: self.result.expect(
                "EvalFlagDetailsBuilder.build() should only be called after evaluation is complete",
            ),
            allocations: self
                .allocation_keys_order
                .into_iter()
                .map(|key| match self.allocation_eval_results.remove(&key) {
                    Some(details) => details,
                    None => EvalAllocationDetails {
                        key,
                        result: EvalAllocationResult::Unevaluated,
                    },
                })
                .collect(),
        }
    }
}

impl EvalVisitor for EvalFlagDetailsBuilder {
    type AllocationVisitor<'a> = EvalAllocationDetailsBuilder<'a>;

    fn visit_allocation<'a>(
        &'a mut self,
        allocation: &super::Allocation,
    ) -> Self::AllocationVisitor<'a> {
        let result = self
            .allocation_eval_results
            .entry(allocation.key.clone())
            .or_insert(EvalAllocationDetails {
                key: allocation.key.clone(),
                result: EvalAllocationResult::Unevaluated,
            });
        EvalAllocationDetailsBuilder { result }
    }

    fn on_configuration(&mut self, _configuration: &super::UniversalFlagConfig) {}

    fn on_flag_configuration(&mut self, flag: &super::Flag) {
        self.allocation_keys_order.truncate(0);
        self.allocation_keys_order
            .extend(flag.allocations.iter().map(|it| &it.key).cloned());
    }

    fn on_result(&mut self, result: &Result<super::Assignment, super::FlagEvaluationError>) {
        self.result = Some(result.clone());
    }
}

impl<'a> EvalAllocationVisitor for EvalAllocationDetailsBuilder<'a> {
    fn on_result(&mut self, result: Result<&Split, AllocationNonMatchReason>) {
        self.result.result = match result {
            Ok(_) => EvalAllocationResult::Matched,
            Err(AllocationNonMatchReason::BeforeStartDate) => EvalAllocationResult::BeforeStartDate,
            Err(AllocationNonMatchReason::AfterEndDate) => EvalAllocationResult::AfterEndDate,
            Err(AllocationNonMatchReason::FailingRules) => EvalAllocationResult::FailingRules,
            Err(AllocationNonMatchReason::TrafficExposureMiss) => {
                EvalAllocationResult::TrafficExposureMiss
            }
        };
    }
}
