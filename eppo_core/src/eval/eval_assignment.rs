use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    error::{EvaluationError, EvaluationFailure},
    events::AssignmentEvent,
    ufc::{
        Allocation, Assignment, AssignmentValue, CompiledFlagsConfig, Flag, Shard, Split,
        Timestamp, VariationType,
    },
    Attributes, Configuration, Str,
};

use super::{
    eval_details::EvaluationResultWithDetails,
    eval_details_builder::EvalDetailsBuilder,
    eval_visitor::{
        EvalAllocationVisitor, EvalAssignmentVisitor, EvalRuleVisitor, EvalSplitVisitor,
        NoopEvalVisitor,
    },
    subject::Subject,
};

/// Evaluate the specified feature flag for the given subject and return assigned variation and
/// an optional assignment event for logging.
pub fn get_assignment(
    configuration: Option<&Configuration>,
    flag_key: &str,
    subject_key: &Str,
    subject_attributes: &Arc<Attributes>,
    expected_type: Option<VariationType>,
    now: DateTime<Utc>,
) -> Result<Option<Assignment>, EvaluationError> {
    get_assignment_with_visitor(
        configuration,
        &mut NoopEvalVisitor,
        flag_key,
        subject_key,
        subject_attributes,
        expected_type,
        now,
    )
}

/// Evaluate the specified feature flag for the given subject and return evaluation details.
pub fn get_assignment_details(
    configuration: Option<&Configuration>,
    flag_key: &str,
    subject_key: &Str,
    subject_attributes: &Arc<Attributes>,
    expected_type: Option<VariationType>,
    now: DateTime<Utc>,
) -> (
    EvaluationResultWithDetails<AssignmentValue>,
    Option<AssignmentEvent>,
) {
    let mut details_builder = EvalDetailsBuilder::new(
        flag_key.to_owned(),
        subject_key.to_owned(),
        subject_attributes.to_owned(),
        now,
    );
    let result = get_assignment_with_visitor(
        configuration,
        &mut details_builder,
        flag_key,
        subject_key,
        subject_attributes,
        expected_type,
        now,
    );

    let (value, mut event) = match result.unwrap_or_default() {
        Some(Assignment { value, event }) => (Some(value), event),
        None => (None, None),
    };

    let evaluation_details = Arc::new(details_builder.build());

    if let Some(event) = &mut event {
        event.evaluation_details = Some(evaluation_details.clone());
    }

    let result_with_details = EvaluationResultWithDetails {
        variation: value,
        action: None,
        evaluation_details,
    };

    (result_with_details, event)
}

// Exposed for use in bandit evaluation.
pub(super) fn get_assignment_with_visitor<V: EvalAssignmentVisitor>(
    configuration: Option<&Configuration>,
    visitor: &mut V,
    flag_key: &str,
    subject_key: &Str,
    subject_attributes: &Arc<Attributes>,
    expected_type: Option<VariationType>,
    now: DateTime<Utc>,
) -> Result<Option<Assignment>, EvaluationError> {
    let result = if let Some(config) = configuration {
        visitor.on_configuration(config);

        config.flags.compiled.eval_flag(
            visitor,
            &flag_key,
            &subject_key,
            &subject_attributes,
            expected_type,
            now,
        )
    } else {
        Err(EvaluationFailure::ConfigurationMissing)
    };

    visitor.on_result(&result);

    match result {
        Ok(assignment) => {
            log::trace!(target: "eppo",
                    flag_key,
                    subject_key,
                    assignment:serde = assignment.value;
                    "evaluated a flag");
            Ok(Some(assignment))
        }

        Err(EvaluationFailure::ConfigurationMissing) => {
            log::warn!(target: "eppo",
                           flag_key,
                           subject_key;
                           "evaluating a flag before Eppo configuration has been fetched");
            Ok(None)
        }

        Err(EvaluationFailure::Error(err)) => {
            log::warn!(target: "eppo",
                       flag_key,
                       subject_key;
                       "error occurred while evaluating a flag: {err}",
            );
            Err(err)
        }

        // Non-Error failures are considered normal conditions and usually don't need extra
        // attention, so we remap them to Ok(None) before returning to the user.
        Err(err) => {
            log::trace!(target: "eppo",
                           flag_key,
                           subject_key;
                           "returning default assignment because of: {err}");
            Ok(None)
        }
    }
}

impl CompiledFlagsConfig {
    /// Evaluate the flag for the given subject, expecting `expected_type` type.
    fn eval_flag<V: EvalAssignmentVisitor>(
        &self,
        visitor: &mut V,
        flag_key: &str,
        subject_key: &Str,
        subject_attributes: &Arc<Attributes>,
        expected_type: Option<VariationType>,
        now: DateTime<Utc>,
    ) -> Result<Assignment, EvaluationFailure> {
        let flag = self.get_flag(flag_key)?;

        visitor.on_flag_configuration(flag);

        if let Some(ty) = expected_type {
            flag.verify_type(ty)?;
        }

        flag.eval(visitor, subject_key, subject_attributes, now)
    }

    fn get_flag(&self, flag_key: &str) -> Result<&Flag, EvaluationFailure> {
        let flag = self
            .flags
            .get(flag_key)
            .ok_or(EvaluationFailure::FlagUnrecognizedOrDisabled)?
            .as_ref()
            .map_err(Clone::clone)?;
        Ok(flag)
    }
}

impl Flag {
    fn verify_type(&self, ty: VariationType) -> Result<(), EvaluationFailure> {
        if self.variation_type == ty {
            Ok(())
        } else {
            Err(EvaluationFailure::Error(EvaluationError::TypeMismatch {
                expected: ty,
                found: self.variation_type,
            }))
        }
    }

    fn eval<V: EvalAssignmentVisitor>(
        &self,
        visitor: &mut V,
        subject_key: &Str,
        subject_attributes: &Arc<Attributes>,
        now: DateTime<Utc>,
    ) -> Result<Assignment, EvaluationFailure> {
        let subject = Subject::new(subject_key.clone(), subject_attributes.clone());

        let Some(split) = self.allocations.iter().find_map(|allocation| {
            let mut visitor = visitor.visit_allocation(allocation);
            let result = allocation.get_matching_split(&mut visitor, &subject, now);
            visitor.on_result(result);
            result.ok()
        }) else {
            return Err(EvaluationFailure::DefaultAllocationNull);
        };

        let (value, event_base) = split.result.clone()?;

        Ok(Assignment {
            value,
            event: event_base.map(|base| AssignmentEvent {
                base,
                subject: subject_key.clone(),
                subject_attributes: subject_attributes.clone(),
                timestamp: now,
                evaluation_details: None,
            }),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum AllocationNonMatchReason {
    BeforeStartDate,
    AfterEndDate,
    FailingRule,
    TrafficExposureMiss,
}

impl Allocation {
    fn get_matching_split<V: EvalAllocationVisitor>(
        &self,
        visitor: &mut V,
        subject: &Subject,
        now: Timestamp,
    ) -> Result<&Split, AllocationNonMatchReason> {
        if self.start_at.is_some_and(|t| now < t) {
            return Err(AllocationNonMatchReason::BeforeStartDate);
        }
        if self.end_at.is_some_and(|t| now > t) {
            return Err(AllocationNonMatchReason::AfterEndDate);
        }

        let is_allowed_by_rules = self.rules.is_empty()
            || self.rules.iter().any(|rule| {
                let mut visitor = visitor.visit_rule(rule);
                let result = rule.eval(&mut visitor, subject);
                visitor.on_result(result);
                result
            });
        if !is_allowed_by_rules {
            return Err(AllocationNonMatchReason::FailingRule);
        }

        self.splits
            .iter()
            .find(|split| {
                let mut visitor = visitor.visit_split(split);
                let matches = split.matches(&mut visitor, subject.key());
                visitor.on_result(matches);
                matches
            })
            .ok_or(AllocationNonMatchReason::TrafficExposureMiss)
    }
}

impl Split {
    /// Return `true` if `subject_key` matches the given split.
    ///
    /// To match a split, subject must match all underlying shards.
    fn matches<V: EvalSplitVisitor>(&self, visitor: &mut V, subject_key: &str) -> bool {
        self.shards
            .iter()
            .all(|shard| shard.matches(visitor, subject_key))
    }
}

impl Shard {
    /// Return `true` if `subject_key` matches the given shard.
    fn matches<V: EvalSplitVisitor>(&self, visitor: &mut V, subject_key: &str) -> bool {
        let h = self.sharder.shard(&[subject_key]);

        let matches = self.ranges.iter().any(|range| range.contains(h));
        visitor.on_shard_eval(self, h, matches);
        matches
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        sync::Arc,
    };

    use chrono::Utc;
    use serde::{Deserialize, Serialize};

    use crate::{
        eval::{
            eval_details::{
                AllocationEvaluationCode, AllocationEvaluationDetails, FlagEvaluationCode,
            },
            get_assignment, get_assignment_details,
        },
        ufc::{RuleWire, UniversalFlagConfig, ValueWire, VariationType},
        Attributes, Configuration, SdkMetadata, Str,
    };

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestFile {
        flag: String,
        variation_type: VariationType,
        default_value: DefaultValue,
        subjects: Vec<TestSubject>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    enum DefaultValue {
        Value(ValueWire),
        Json(serde_json::Value),
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestSubject {
        subject_key: Str,
        subject_attributes: Arc<Attributes>,
        assignment: DefaultValue,
        evaluation_details: TruncatedEvaluationDetails,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TruncatedEvaluationDetails {
        /// Environment the configuration belongs to. None if configuration hasn't been fetched yet.
        environment_name: Option<Str>,

        flag_evaluation_code: TruncatedFlagEvaluationCode,
        flag_evaluation_description: String,

        /// Key of the selected variation.
        variation_key: Option<Str>,
        /// Value of the selected variation. Could be `None` if no variation is selected, or selected
        /// value is absent in configuration (configuration error).
        variation_value: serde_json::Value,

        bandit_key: Option<Str>,
        bandit_action: Option<Str>,

        matched_rule: Option<RuleWire>,
        matched_allocation: Option<TruncatedAllocationEvaluationDetails>,
        unmatched_allocations: Vec<TruncatedAllocationEvaluationDetails>,
        unevaluated_allocations: Vec<TruncatedAllocationEvaluationDetails>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    enum TruncatedFlagEvaluationCode {
        // AssignmentError is not used in Rust because we know all possible errors.
        AssignmentError,
        #[serde(untagged)]
        Native(FlagEvaluationCode),
    }
    impl From<TruncatedFlagEvaluationCode> for FlagEvaluationCode {
        fn from(value: TruncatedFlagEvaluationCode) -> Self {
            match value {
                TruncatedFlagEvaluationCode::AssignmentError => {
                    FlagEvaluationCode::UnexpectedConfigurationError
                }
                TruncatedFlagEvaluationCode::Native(code) => code,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TruncatedAllocationEvaluationDetails {
        pub key: Str,
        /// Order position of the allocation as seen in the Web UI.
        pub order_position: usize,
        pub allocation_evaluation_code: AllocationEvaluationCode,
    }
    impl From<&AllocationEvaluationDetails> for TruncatedAllocationEvaluationDetails {
        fn from(value: &AllocationEvaluationDetails) -> Self {
            Self {
                key: value.key.clone(),
                order_position: value.order_position,
                allocation_evaluation_code: value.allocation_evaluation_code,
            }
        }
    }

    // Test files have different representation of Value for JSON. Whereas server returns a string
    // that has to be further parsed, test files embed the JSON object directly.
    //
    // Therefore, if we failed to parse "assignment" field as one of the values, we fallback to
    // AttributeValue::Json.
    fn to_value(value: DefaultValue) -> ValueWire {
        match value {
            DefaultValue::Value(v) => v,
            DefaultValue::Json(json) => {
                ValueWire::String(serde_json::to_string(&json).unwrap().into())
            }
        }
    }

    #[test]
    fn evaluation_sdk_test_data() {
        let _ = env_logger::builder().is_test(true).try_init();

        let config = UniversalFlagConfig::from_json(
            SdkMetadata {
                name: "test",
                version: "0.1.0",
            },
            std::fs::read("../sdk-test-data/ufc/flags-v1.json").unwrap(),
        )
        .unwrap();
        let config = Configuration::from_server_response(config, None);
        let now = Utc::now();

        for entry in fs::read_dir("../sdk-test-data/ufc/tests/").unwrap() {
            let entry = entry.unwrap();
            println!("Processing test file: {:?}", entry.path());

            let f = File::open(entry.path()).unwrap();
            let test_file: TestFile = serde_json::from_reader(f).unwrap();

            let default_assignment = to_value(test_file.default_value)
                .into_assignment_value(test_file.variation_type)
                .unwrap();

            for subject in test_file.subjects {
                print!("test subject {:?} ... ", subject.subject_key);
                let result = get_assignment(
                    Some(&config),
                    &test_file.flag,
                    &subject.subject_key,
                    &subject.subject_attributes,
                    Some(test_file.variation_type),
                    now,
                )
                .unwrap_or(None);

                let result_assingment = result
                    .as_ref()
                    .map(|assignment| &assignment.value)
                    .unwrap_or(&default_assignment);
                let expected_assignment = to_value(subject.assignment)
                    .into_assignment_value(test_file.variation_type)
                    .unwrap();

                assert_eq!(result_assingment, &expected_assignment);
                println!("ok");
            }
        }
    }

    #[test]
    fn evaluation_details_sdk_test_data() {
        let _ = env_logger::builder().is_test(true).try_init();

        let config = UniversalFlagConfig::from_json(
            SdkMetadata {
                name: "test",
                version: "0.1.0",
            },
            fs::read("../sdk-test-data/ufc/flags-v1.json").unwrap(),
        )
        .unwrap();
        let config = Configuration::from_server_response(config, None);
        let now = Utc::now();

        for entry in fs::read_dir("../sdk-test-data/ufc/tests/").unwrap() {
            let entry = entry.unwrap();
            println!("Processing test file: {:?}", entry.path());

            let f = File::open(entry.path()).unwrap();
            let test_file: TestFile = serde_json::from_reader(f).unwrap();

            for subject in test_file.subjects {
                print!("test subject {:?} ... ", subject.subject_key);
                let (result, _) = get_assignment_details(
                    Some(&config),
                    &test_file.flag,
                    &subject.subject_key,
                    &subject.subject_attributes,
                    Some(test_file.variation_type),
                    now,
                );

                let actual = result.evaluation_details;
                let expected = &subject.evaluation_details;

                assert_eq!(actual.environment_name, expected.environment_name);
                assert_eq!(
                    actual.flag_evaluation_code,
                    Some(FlagEvaluationCode::from(expected.flag_evaluation_code))
                );
                if expected.flag_evaluation_code != TruncatedFlagEvaluationCode::AssignmentError {
                    // Assignment errors never happen in Rust and we generate different description.
                    assert_eq!(
                        actual.flag_evaluation_description, expected.flag_evaluation_description,
                        "details: {actual:#?}"
                    );
                }
                assert_eq!(actual.bandit_key, expected.bandit_key);
                assert_eq!(actual.variation_key, expected.variation_key);
                // Truncated evaluation details erase variation value (integer vs. float vs. json)
                // type and simultaneously transform the value in a ways that is impossible to
                // reverse (parse json from string). So it is impossible to test against.
                //
                // assert_eq!(
                //     actual.variation_value,
                //     expected.variation_value
                // );

                let mut matched_allocation: Option<TruncatedAllocationEvaluationDetails> = None;
                let mut unmatched_allocations: Vec<TruncatedAllocationEvaluationDetails> =
                    Vec::new();
                let mut unevaluated_allocations: Vec<TruncatedAllocationEvaluationDetails> =
                    Vec::new();
                for allocation in &actual.allocations {
                    match allocation.allocation_evaluation_code {
                        AllocationEvaluationCode::Unevaluated => {
                            unevaluated_allocations.push(allocation.into())
                        }
                        AllocationEvaluationCode::Match => {
                            matched_allocation = Some(allocation.into())
                        }
                        AllocationEvaluationCode::BeforeStartTime
                        | AllocationEvaluationCode::AfterEndTime
                        | AllocationEvaluationCode::FailingRule
                        | AllocationEvaluationCode::TrafficExposureMiss => {
                            unmatched_allocations.push(allocation.into())
                        }
                    }
                }

                assert_eq!(matched_allocation, expected.matched_allocation);
                assert_eq!(unmatched_allocations, expected.unmatched_allocations);
                assert_eq!(unevaluated_allocations, expected.unevaluated_allocations);

                println!("ok");
            }
        }
    }
}
