use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Attributes, Configuration};

use super::{
    eval::AllocationNonMatchReason, eval_visitor::*, Assignment, FlagEvaluationError, Split, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalFlagDetails {
    pub flag_key: String,
    pub subject_key: String,
    pub subject_attributes: Attributes,
    pub timestamp: DateTime<Utc>,
    /// Details of configuration used for evaluation. None if configuration hasn't been fetched yet.
    pub configuration_details: Option<ConfigurationDetails>,
    pub result: Result<Assignment, FlagEvaluationError>,
    /// Key of the selected variation.
    pub variation_key: Option<String>,
    /// Value of the selected variation. Could be `None` if no variation is selected, or selected
    /// value is absent in configuration (configuration error).
    pub variation_value: Option<Value>,
    pub allocations: Vec<EvalAllocationDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationDetails {
    /// Timestamp when configuration was fetched by the SDK.
    pub fetched_at: DateTime<Utc>,
    /// Timestamp when configuration was published by the server.
    pub published_at: DateTime<Utc>,
    /// Environment the configuration belongs to.
    pub environment_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalAllocationDetails {
    pub key: String,
    pub result: EvalAllocationResult,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
    now: DateTime<Utc>,
    configuration_details: Option<ConfigurationDetails>,

    result: Option<Result<Assignment, FlagEvaluationError>>,

    variation_key: Option<String>,
    variation_value: Option<Value>,

    /// List of allocation keys. Used to sort `allocation_eval_results`.
    allocation_keys_order: Vec<String>,
    allocation_eval_results: HashMap<String, EvalAllocationDetails>,
}

pub(crate) struct EvalAllocationDetailsBuilder<'a> {
    result: &'a mut EvalAllocationDetails,
    variation_key: &'a mut Option<String>,
}

impl EvalFlagDetailsBuilder {
    pub fn new(
        flag_key: String,
        subject_key: String,
        subject_attributes: Attributes,
        now: DateTime<Utc>,
    ) -> EvalFlagDetailsBuilder {
        EvalFlagDetailsBuilder {
            flag_key,
            subject_key,
            subject_attributes,
            now,
            configuration_details: None,
            result: None,
            variation_key: None,
            variation_value: None,
            allocation_keys_order: Vec::new(),
            allocation_eval_results: HashMap::new(),
        }
    }

    pub fn build(mut self) -> EvalFlagDetails {
        EvalFlagDetails {
            flag_key: self.flag_key,
            subject_key: self.subject_key,
            subject_attributes: self.subject_attributes,
            timestamp: self.now,
            configuration_details: self.configuration_details,
            result: self.result.expect(
                "EvalFlagDetailsBuilder.build() should only be called after evaluation is complete",
            ),
            variation_key: self.variation_key,
            variation_value: self.variation_value,
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
        EvalAllocationDetailsBuilder {
            result,
            variation_key: &mut self.variation_key,
        }
    }

    fn on_configuration(&mut self, configuration: &Configuration) {
        self.configuration_details = Some(ConfigurationDetails {
            fetched_at: configuration.fetched_at,
            published_at: configuration.flags.created_at,
            environment_name: configuration.flags.environment.name.clone(),
        })
    }

    fn on_flag_configuration(&mut self, flag: &super::Flag) {
        self.allocation_keys_order.truncate(0);
        self.allocation_keys_order
            .extend(flag.allocations.iter().map(|it| &it.key).cloned());
    }

    fn on_variation(&mut self, variation: &super::Variation) {
        self.variation_value = Some(variation.value.clone());
    }

    fn on_result(&mut self, result: &Result<super::Assignment, super::FlagEvaluationError>) {
        self.result = Some(result.clone());
    }
}

impl<'a> EvalAllocationVisitor for EvalAllocationDetailsBuilder<'a> {
    fn on_result(&mut self, result: Result<&Split, AllocationNonMatchReason>) {
        *self.variation_key = result.ok().map(|split| split.variation_key.clone());

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
