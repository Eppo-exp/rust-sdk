use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{error::EvaluationFailure, AttributeValue, Attributes, Configuration};

use super::{
    eval::AllocationNonMatchReason, eval_details::*, eval_visitor::*, Assignment, Condition, Rule,
    Shard, Split, Value,
};

pub(crate) struct EvalFlagDetailsBuilder {
    flag_key: String,
    subject_key: String,
    subject_attributes: Attributes,
    now: DateTime<Utc>,

    configuration_fetched_at: Option<DateTime<Utc>>,
    configuration_published_at: Option<DateTime<Utc>>,
    environment_name: Option<String>,

    evaluation_failure: Option<EvaluationFailure>,

    variation_key: Option<String>,
    variation_value: Option<Value>,

    /// List of allocation keys. Used to sort `allocation_eval_results`.
    allocation_keys_order: Vec<String>,
    allocation_eval_results: HashMap<String, AllocationEvaluationDetails>,
}

pub(crate) struct EvalAllocationDetailsBuilder<'a> {
    allocation_details: &'a mut AllocationEvaluationDetails,
    variation_key: &'a mut Option<String>,
}

pub(crate) struct EvalRuleDetailsBuilder<'a> {
    rule_details: &'a mut RuleEvaluationDetails,
}

pub(crate) struct EvalSplitDetailsBuilder<'a> {
    split_details: &'a mut SplitEvaluationDetails,
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
            configuration_fetched_at: None,
            configuration_published_at: None,
            environment_name: None,
            evaluation_failure: None,
            variation_key: None,
            variation_value: None,
            allocation_keys_order: Vec::new(),
            allocation_eval_results: HashMap::new(),
        }
    }

    pub fn build(mut self) -> EvaluationDetails {
        EvaluationDetails {
            flag_key: self.flag_key,
            subject_key: self.subject_key,
            subject_attributes: self.subject_attributes,
            timestamp: self.now,
            config_fetched_at: self.configuration_fetched_at,
            config_published_at: self.configuration_published_at,
            environment_name: self.environment_name,
            flag_evaluation_code: self.evaluation_failure.into(),
            variation_key: self.variation_key,
            variation_value: self.variation_value,
            bandit_key: None,
            bandit_action: None,
            allocations: self
                .allocation_keys_order
                .into_iter()
                .enumerate()
                .map(|(i, key)| match self.allocation_eval_results.remove(&key) {
                    Some(details) => details,
                    None => AllocationEvaluationDetails {
                        key,
                        order_position: i + 1,
                        allocation_evaluation_code: AllocationEvaluationCode::Unevaluated,
                        evaluated_rules: Vec::new(),
                        evaluated_splits: Vec::new(),
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
        let order_position = self.allocation_eval_results.len() + 1;
        let result = self
            .allocation_eval_results
            .entry(allocation.key.clone())
            .or_insert(AllocationEvaluationDetails {
                key: allocation.key.clone(),
                order_position,
                allocation_evaluation_code: AllocationEvaluationCode::Unevaluated,
                evaluated_rules: Vec::new(),
                evaluated_splits: Vec::new(),
            });
        EvalAllocationDetailsBuilder {
            allocation_details: result,
            variation_key: &mut self.variation_key,
        }
    }

    fn on_configuration(&mut self, configuration: &Configuration) {
        self.configuration_fetched_at = Some(configuration.fetched_at);
        self.configuration_published_at = Some(configuration.flags.created_at);
        self.environment_name = Some(configuration.flags.environment.name.clone());
    }

    fn on_flag_configuration(&mut self, flag: &super::Flag) {
        self.allocation_keys_order.truncate(0);
        self.allocation_keys_order
            .extend(flag.allocations.iter().map(|it| &it.key).cloned());
    }

    fn on_variation(&mut self, variation: &super::Variation) {
        self.variation_value = Some(variation.value.clone());
    }

    fn on_result(&mut self, result: &Result<Assignment, EvaluationFailure>) {
        self.evaluation_failure = result.as_ref().err().copied();
    }
}

impl<'b> EvalAllocationVisitor for EvalAllocationDetailsBuilder<'b> {
    type RuleVisitor<'a> = EvalRuleDetailsBuilder<'a>
    where
        Self: 'a;

    type SplitVisitor<'a> = EvalSplitDetailsBuilder<'a>
    where
        Self: 'a;

    fn visit_rule<'a>(&'a mut self, _rule: &Rule) -> EvalRuleDetailsBuilder<'a> {
        self.allocation_details
            .evaluated_rules
            .push(RuleEvaluationDetails {
                matched: false,
                conditions: Vec::new(),
            });
        EvalRuleDetailsBuilder {
            rule_details: self
                .allocation_details
                .evaluated_rules
                .last_mut()
                .expect("we just inserted an element, so there must be last"),
        }
    }

    fn visit_split<'a>(&'a mut self, split: &Split) -> Self::SplitVisitor<'a> {
        self.allocation_details
            .evaluated_splits
            .push(SplitEvaluationDetails {
                matched: false,
                variation_key: split.variation_key.clone(),
                shards: Vec::new(),
            });
        EvalSplitDetailsBuilder {
            split_details: self
                .allocation_details
                .evaluated_splits
                .last_mut()
                .expect("we just inserted an element, so there must be last"),
        }
    }

    fn on_result(&mut self, result: Result<&Split, AllocationNonMatchReason>) {
        *self.variation_key = result.ok().map(|split| split.variation_key.clone());

        self.allocation_details.allocation_evaluation_code = match result {
            Ok(_) => AllocationEvaluationCode::Match,
            Err(AllocationNonMatchReason::BeforeStartDate) => {
                AllocationEvaluationCode::BeforeStartDate
            }
            Err(AllocationNonMatchReason::AfterEndDate) => AllocationEvaluationCode::AfterEndDate,
            Err(AllocationNonMatchReason::FailingRule) => AllocationEvaluationCode::FailingRule,
            Err(AllocationNonMatchReason::TrafficExposureMiss) => {
                AllocationEvaluationCode::TrafficExposureMiss
            }
        };
    }
}

impl<'a> EvalRuleVisitor for EvalRuleDetailsBuilder<'a> {
    fn on_condition_eval(
        &mut self,
        condition: &Condition,
        attribute_value: Option<&AttributeValue>,
        result: bool,
    ) {
        self.rule_details
            .conditions
            .push(ConditionEvaluationDetails {
                matched: result,
                condition: condition.clone(),
                attribute_value: attribute_value.cloned(),
            });
    }

    fn on_result(&mut self, result: bool) {
        self.rule_details.matched = result;
    }
}

impl<'a> EvalSplitVisitor for EvalSplitDetailsBuilder<'a> {
    fn on_shard_eval(&mut self, shard: &Shard, shard_value: u64, matches: bool) {
        self.split_details.shards.push(ShardEvaluationDetails {
            matched: matches,
            shard: shard.clone(),
            shard_value,
        });
    }

    fn on_result(&mut self, matches: bool) {
        self.split_details.matched = matches;
    }
}
