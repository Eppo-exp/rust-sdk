use chrono::{DateTime, Utc};

use crate::{sharder::get_md5_shard, Attributes, Configuration};

use super::{
    eval_details::{EvalFlagDetails, EvalFlagDetailsBuilder},
    eval_visitor::{
        EvalAllocationVisitor, EvalRuleVisitor, EvalSplitVisitor, EvalVisitor, NoopEvalVisitor,
    },
    Allocation, Assignment, AssignmentEvent, Flag, FlagEvaluationError, Shard, Split, Timestamp,
    TryParse, UniversalFlagConfig, VariationType,
};

/// Evaluate the specified feature flag for the given subject and return assigned variation and
/// an optional assignment event for logging.
pub fn get_assignment(
    configuration: Option<&Configuration>,
    flag_key: &str,
    subject_key: &str,
    subject_attributes: &Attributes,
    expected_type: Option<VariationType>,
) -> Result<Option<Assignment>, FlagEvaluationError> {
    let now = Utc::now();
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
    subject_key: &str,
    subject_attributes: &Attributes,
    expected_type: Option<VariationType>,
) -> (
    Result<Option<Assignment>, FlagEvaluationError>,
    EvalFlagDetails,
) {
    let now = Utc::now();
    let mut builder = EvalFlagDetailsBuilder::new(
        flag_key.to_owned(),
        subject_key.to_owned(),
        subject_attributes.to_owned(),
        now,
    );
    let result = get_assignment_with_visitor(
        configuration,
        &mut builder,
        flag_key,
        subject_key,
        subject_attributes,
        expected_type,
        now,
    );
    let details = builder.build();
    (result, details)
}

fn get_assignment_with_visitor<V: EvalVisitor>(
    configuration: Option<&Configuration>,
    visitor: &mut V,
    flag_key: &str,
    subject_key: &str,
    subject_attributes: &Attributes,
    expected_type: Option<VariationType>,
    now: DateTime<Utc>,
) -> Result<Option<Assignment>, FlagEvaluationError> {
    let result = get_assignment_inner(
        configuration,
        visitor,
        flag_key,
        subject_key,
        subject_attributes,
        expected_type,
        now,
    );

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

        Err(FlagEvaluationError::ConfigurationMissing) => {
            log::warn!(target: "eppo",
                           flag_key,
                           subject_key;
                           "evaluating a flag before Eppo configuration has been fetched");
            Ok(None)
        }

        // These are considered normal conditions and usually don't need extra attention, so we
        // remap them to Ok(None) before returning to the user.
        Err(err) if err.is_normal() => {
            log::trace!(target: "eppo",
                           flag_key,
                           subject_key;
                           "returning default assignment because of: {err}");
            Ok(None)
        }

        Err(err) => {
            log::warn!(target: "eppo",
                       flag_key,
                       subject_key;
                       "error occurred while evaluating a flag: {err}",
            );
            Err(err)
        }
    }
}

fn get_assignment_inner<V: EvalVisitor>(
    configuration: Option<&Configuration>,
    visitor: &mut V,
    flag_key: &str,
    subject_key: &str,
    subject_attributes: &Attributes,
    expected_type: Option<VariationType>,
    now: DateTime<Utc>,
) -> Result<Assignment, FlagEvaluationError> {
    let Some(config) = configuration else {
        return Err(FlagEvaluationError::ConfigurationMissing);
    };

    visitor.on_configuration(config);

    config.flags.eval_flag(
        visitor,
        &flag_key,
        &subject_key,
        &subject_attributes,
        expected_type,
        now,
    )
}

impl UniversalFlagConfig {
    /// Evaluate the flag for the given subject, expecting `expected_type` type.
    fn eval_flag<V: EvalVisitor>(
        &self,
        visitor: &mut V,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
        expected_type: Option<VariationType>,
        now: DateTime<Utc>,
    ) -> Result<Assignment, FlagEvaluationError> {
        let flag = self.get_flag(flag_key)?;

        visitor.on_flag_configuration(flag);

        if let Some(ty) = expected_type {
            flag.verify_type(ty)?;
        }

        flag.eval(visitor, subject_key, subject_attributes, now)
    }

    fn get_flag<'a>(&'a self, flag_key: &str) -> Result<&'a Flag, FlagEvaluationError> {
        let flag = self
            .flags
            .get(flag_key)
            .ok_or(FlagEvaluationError::FlagNotFound)?;

        match flag {
            TryParse::Parsed(flag) => Ok(flag),
            TryParse::ParseFailed(_) => Err(FlagEvaluationError::ConfigurationParseError),
        }
    }
}

impl Flag {
    fn verify_type(&self, ty: VariationType) -> Result<(), FlagEvaluationError> {
        if self.variation_type == ty {
            Ok(())
        } else {
            Err(FlagEvaluationError::InvalidType {
                expected: ty,
                found: self.variation_type,
            })
        }
    }

    fn eval<V: EvalVisitor>(
        &self,
        visitor: &mut V,
        subject_key: &str,
        subject_attributes: &Attributes,
        now: DateTime<Utc>,
    ) -> Result<Assignment, FlagEvaluationError> {
        if !self.enabled {
            return Err(FlagEvaluationError::FlagDisabled);
        }

        // Augmenting subject_attributes with id, so that subject_key can be used in the rules.
        let subject_attributes_with_id = {
            let mut sa = subject_attributes.clone();
            sa.entry("id".into()).or_insert_with(|| subject_key.into());
            sa
        };

        let Some((allocation, split)) = self.allocations.iter().find_map(|allocation| {
            let mut visitor = visitor.visit_allocation(allocation);
            let result = allocation.get_matching_split(
                &mut visitor,
                subject_key,
                &subject_attributes_with_id,
                self.total_shards,
                now,
            );
            visitor.on_result(result);
            result.ok().map(|split| (allocation, split))
        }) else {
            return Err(FlagEvaluationError::NoAllocation);
        };

        let variation = self.variations.get(&split.variation_key).ok_or_else(|| {
            log::warn!(target: "eppo",
                       flag_key:display = self.key,
                       subject_key,
                       variation_key:display = split.variation_key;
                       "internal: unable to find variation");
            FlagEvaluationError::ConfigurationError
        })?;

        visitor.on_variation(variation);

        let assignment_value = variation
            .value
            .to_assignment_value(self.variation_type)
            .ok_or_else(|| {
                log::warn!(target: "eppo",
                           flag_key:display = self.key,
                           subject_key,
                           variation_key:display = split.variation_key;
                           "internal: unable to convert Value to AssignmentValue");
                FlagEvaluationError::ConfigurationError
            })?;

        let event = allocation.do_log.then(|| AssignmentEvent {
            feature_flag: self.key.clone(),
            allocation: allocation.key.clone(),
            experiment: format!("{}-{}", self.key, allocation.key),
            variation: variation.key.clone(),
            subject: subject_key.to_owned(),
            subject_attributes: subject_attributes.clone(),
            timestamp: now.to_rfc3339(),
            meta_data: [(
                "eppoCoreVersion".to_owned(),
                env!("CARGO_PKG_VERSION").to_owned(),
            )]
            .into(),
            extra_logging: split.extra_logging.clone(),
        });

        Ok(Assignment {
            value: assignment_value,
            event,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum AllocationNonMatchReason {
    BeforeStartDate,
    AfterEndDate,
    FailingRules,
    TrafficExposureMiss,
}

impl Allocation {
    fn get_matching_split<V: EvalAllocationVisitor>(
        &self,
        visitor: &mut V,
        subject_key: &str,
        subject_attributes_with_id: &Attributes,
        total_shards: u64,
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
                let result = rule.eval(&mut visitor, subject_attributes_with_id);
                visitor.on_result(result);
                result
            });
        if !is_allowed_by_rules {
            return Err(AllocationNonMatchReason::FailingRules);
        }

        self.splits
            .iter()
            .find(|split| {
                let mut visitor = visitor.visit_split(split);
                let matches = split.matches(&mut visitor, subject_key, total_shards);
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
    fn matches<V: EvalSplitVisitor>(
        &self,
        visitor: &mut V,
        subject_key: &str,
        total_shards: u64,
    ) -> bool {
        self.shards
            .iter()
            .all(|shard| shard.matches(visitor, subject_key, total_shards))
    }
}

impl Shard {
    /// Return `true` if `subject_key` matches the given shard.
    fn matches<V: EvalSplitVisitor>(
        &self,
        visitor: &mut V,
        subject_key: &str,
        total_shards: u64,
    ) -> bool {
        let h = get_md5_shard(&[self.salt.as_str(), "-", subject_key], total_shards);
        let matches = self.ranges.iter().any(|range| range.contains(h));
        visitor.on_shard_eval(self, h, matches);
        matches
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};

    use serde::{Deserialize, Serialize};

    use crate::{
        ufc::{get_assignment, TryParse, UniversalFlagConfig, Value, VariationType},
        Attributes, Configuration,
    };

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestFile {
        flag: String,
        variation_type: VariationType,
        default_value: TryParse<Value>,
        subjects: Vec<TestSubject>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestSubject {
        subject_key: String,
        subject_attributes: Attributes,
        assignment: TryParse<Value>,
    }

    // Test files have different representation of Value for JSON. Whereas server returns a string
    // that has to be further parsed, test files embed the JSON object directly.
    //
    // Therefore, if we failed to parse "assignment" field as one of the values, we fallback to
    // AttributeValue::Json.
    fn to_value(try_parse: TryParse<Value>) -> Value {
        match try_parse {
            TryParse::Parsed(v) => v,
            TryParse::ParseFailed(json) => Value::String(serde_json::to_string(&json).unwrap()),
        }
    }

    #[test]
    fn evaluation_sdk_test_data() {
        let _ = env_logger::builder().is_test(true).try_init();

        let config: UniversalFlagConfig =
            serde_json::from_reader(File::open("../sdk-test-data/ufc/flags-v1.json").unwrap())
                .unwrap();
        let config = Configuration::from_server_response(config, None);

        for entry in fs::read_dir("../sdk-test-data/ufc/tests/").unwrap() {
            let entry = entry.unwrap();
            println!("Processing test file: {:?}", entry.path());

            let f = File::open(entry.path()).unwrap();
            let test_file: TestFile = serde_json::from_reader(f).unwrap();

            let default_assignment = to_value(test_file.default_value)
                .to_assignment_value(test_file.variation_type)
                .unwrap();

            for subject in test_file.subjects {
                print!("test subject {:?} ... ", subject.subject_key);
                let result = get_assignment(
                    Some(&config),
                    &test_file.flag,
                    &subject.subject_key,
                    &subject.subject_attributes,
                    Some(test_file.variation_type),
                )
                .unwrap_or(None);

                let result_assingment = result
                    .as_ref()
                    .map(|assignment| &assignment.value)
                    .unwrap_or(&default_assignment);
                let expected_assignment = to_value(subject.assignment)
                    .to_assignment_value(test_file.variation_type)
                    .unwrap();

                assert_eq!(result_assingment, &expected_assignment);
                println!("ok");
            }
        }
    }
}
