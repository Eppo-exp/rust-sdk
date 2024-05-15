use std::collections::HashMap;

use chrono::Utc;

use crate::{
    client::AssignmentValue,
    sharder::Sharder,
    ufc::{Allocation, Flag, Shard, Split, Timestamp, TryParse, UniversalFlagConfig},
    AssignmentEvent, Error, Result, SubjectAttributes,
};

impl UniversalFlagConfig {
    pub fn eval_flag(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &SubjectAttributes,
        sharder: &impl Sharder,
    ) -> Result<Option<(AssignmentValue, Option<AssignmentEvent>)>> {
        let flag = self.flags.get(flag_key).ok_or(Error::FlagNotFound)?;

        match flag {
            TryParse::Parsed(flag) => flag.eval(subject_key, subject_attributes, sharder),
            TryParse::ParseFailed(_) => Err(Error::ConfigurationParseError),
        }
    }
}

impl Flag {
    pub fn eval(
        &self,
        subject_key: &str,
        subject_attributes: &SubjectAttributes,
        sharder: &impl Sharder,
    ) -> Result<Option<(AssignmentValue, Option<AssignmentEvent>)>> {
        if !self.enabled {
            return Ok(None);
        }

        let now = Utc::now();

        // Augmenting subject_attributes with id, so that subject_key can be used in the rules.
        let augmented_subject_attributes = {
            let mut sa = subject_attributes.clone();
            sa.entry("id".into()).or_insert_with(|| subject_key.into());
            sa
        };

        let Some((allocation, split)) = self.allocations.iter().find_map(|allocation| {
            allocation
                .get_matching_split(
                    subject_key,
                    &augmented_subject_attributes,
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
            Error::ConfigurationError
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
                Error::ConfigurationError
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
                meta_data: HashMap::from([
                    ("sdkLanguage".to_owned(), "rust".to_owned()),
                    (
                        "sdkVersion".to_owned(),
                        env!("CARGO_PKG_VERSION").to_owned(),
                    ),
                ]),
                extra_logging: split.extra_logging.clone(),
            })
        } else {
            None
        };

        Ok(Some((assignment_value, event)))
    }
}

impl Allocation {
    pub fn get_matching_split(
        &self,
        subject_key: &str,
        augmented_subject_attributes: &SubjectAttributes,
        sharder: &impl Sharder,
        total_shards: u64,
        now: Timestamp,
    ) -> Option<&Split> {
        if self.is_allowed_by_time(now) && self.is_allowed_by_rules(augmented_subject_attributes) {
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

    fn is_allowed_by_rules(&self, augmented_subject_attributes: &SubjectAttributes) -> bool {
        self.rules.is_empty()
            || self
                .rules
                .iter()
                .any(|rule| rule.eval(augmented_subject_attributes))
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
        sharder::Md5Sharder,
        ufc::{TryParse, UniversalFlagConfig, Value, VariationType},
        SubjectAttributes,
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
        subject_attributes: SubjectAttributes,
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
        let config: UniversalFlagConfig =
            serde_json::from_reader(File::open("tests/data/ufc/flags-v1.json").unwrap()).unwrap();

        for entry in fs::read_dir("tests/data/ufc/tests/").unwrap() {
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
                        &Md5Sharder,
                    )
                    .unwrap_or(None);

                let result_assingment = result
                    .as_ref()
                    .map(|(value, _event)| value)
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
