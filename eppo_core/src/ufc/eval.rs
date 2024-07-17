use chrono::Utc;

use crate::{
    sharder::{Md5Sharder, Sharder},
    Attributes, Configuration,
};

use super::{
    Allocation, Assignment, AssignmentEvent, Flag, FlagEvaluationError, Shard, Split, Timestamp,
    TryParse, UniversalFlagConfig, VariationType,
};

impl Configuration {
    /// Evaluate the specified feature flag for the given subject and return assigned variation and
    /// an optional assignment event for logging.
    pub fn get_assignment(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
        expected_type: Option<VariationType>,
    ) -> Result<Option<Assignment>, FlagEvaluationError> {
        let Some(ufc) = &self.flags else {
            log::warn!(target: "eppo", flag_key, subject_key; "evaluating a flag before Eppo configuration has been fetched");
            // We treat missing configuration (the poller has not fetched config) as a normal
            // scenario.
            return Ok(None);
        };

        let assignment =
            match ufc.eval_flag(&flag_key, &subject_key, &subject_attributes, expected_type) {
                Ok(result) => result,
                Err(err) => {
                    log::warn!(target: "eppo",
                               flag_key,
                               subject_key,
                               subject_attributes:serde;
                               "error occurred while evaluating a flag: {:?}", err,
                    );
                    return Err(err);
                }
            };

        log::trace!(target: "eppo",
                    flag_key,
                    subject_key,
                    subject_attributes:serde,
                    assignment:serde = assignment.as_ref().map(|Assignment{value, ..}| value);
                    "evaluated a flag");

        Ok(assignment)
    }
}

impl UniversalFlagConfig {
    /// Evaluate the flag for the given subject, expecting `expected_type` type.
    pub fn eval_flag(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
        expected_type: Option<VariationType>,
    ) -> Result<Option<Assignment>, FlagEvaluationError> {
        let flag = self.get_flag(flag_key)?;

        if let Some(ty) = expected_type {
            flag.verify_type(ty)?;
        }

        flag.eval(subject_key, subject_attributes, &Md5Sharder)
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

    fn eval(
        &self,
        subject_key: &str,
        subject_attributes: &Attributes,
        sharder: &impl Sharder,
    ) -> Result<Option<Assignment>, FlagEvaluationError> {
        if !self.enabled {
            return Ok(None);
        }

        let now = Utc::now();

        // Augmenting subject_attributes with id, so that subject_key can be used in the rules.
        let subject_attributes_with_id = {
            let mut sa = subject_attributes.clone();
            sa.entry("id".into()).or_insert_with(|| subject_key.into());
            sa
        };

        let Some((allocation, split)) = self.allocations.iter().find_map(|allocation| {
            allocation
                .get_matching_split(
                    subject_key,
                    &subject_attributes_with_id,
                    sharder,
                    self.total_shards,
                    now,
                )
                .map(|split| (allocation, split))
        }) else {
            return Ok(None);
        };

        let variation = self.variations.get(&split.variation_key).ok_or_else(|| {
            log::warn!(target: "eppo",
                       flag_key:display = self.key,
                       subject_key,
                       variation_key:display = split.variation_key;
                       "internal: unable to find variation");
            FlagEvaluationError::ConfigurationError
        })?;

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

        let event = if allocation.do_log {
            Some(AssignmentEvent {
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
            })
        } else {
            None
        };

        Ok(Some(Assignment {
            value: assignment_value,
            event,
        }))
    }
}

impl Allocation {
    fn get_matching_split(
        &self,
        subject_key: &str,
        subject_attributes_with_id: &Attributes,
        sharder: &impl Sharder,
        total_shards: u64,
        now: Timestamp,
    ) -> Option<&Split> {
        if self.is_allowed_by_time(now) && self.is_allowed_by_rules(subject_attributes_with_id) {
            self.splits
                .iter()
                .find(|split| split.matches(subject_key, sharder, total_shards))
        } else {
            None
        }
    }

    fn is_allowed_by_time(&self, now: Timestamp) -> bool {
        let forbidden = matches!(self.start_at, Some(t) if now < t)
            || matches!(self.end_at, Some(t) if now > t);
        !forbidden
    }

    fn is_allowed_by_rules(&self, subject_attributes_with_id: &Attributes) -> bool {
        self.rules.is_empty()
            || self
                .rules
                .iter()
                .any(|rule| rule.eval(subject_attributes_with_id))
    }
}

impl Split {
    /// Return `true` if `subject_key` matches the given split under the provided `sharder`.
    ///
    /// To match a split, subject must match all underlying shards.
    fn matches(&self, subject_key: &str, sharder: &impl Sharder, total_shards: u64) -> bool {
        self.shards
            .iter()
            .all(|shard| shard.matches(subject_key, sharder, total_shards))
    }
}

impl Shard {
    /// Return `true` if `subject_key` matches the given shard under the provided `sharder`.
    fn matches(&self, subject_key: &str, sharder: &impl Sharder, total_shards: u64) -> bool {
        let h = sharder.get_shard(&format!("{}-{}", self.salt, subject_key), total_shards);
        self.ranges.iter().any(|range| range.contains(h))
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};

    use serde::{Deserialize, Serialize};

    use crate::{
        ufc::{TryParse, UniversalFlagConfig, Value, VariationType},
        Attributes,
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
                let result = config
                    .eval_flag(
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
