use std::collections::HashMap;

use chrono::Utc;

use crate::{
    client::AssignmentValue,
    sharder::Sharder,
    ufc::{Allocation, Flag, Shard, Split, Timestamp},
    AssignmentEvent, SubjectAttributes,
};

impl Flag {
    #[allow(dead_code)]
    pub fn eval<'a>(
        self: &'a Flag,
        subject_key: &str,
        subject_attributes: &SubjectAttributes,
        sharder: &impl Sharder,
    ) -> Option<(AssignmentValue, Option<AssignmentEvent>)> {
        if !self.enabled {
            return None;
        }

        let now = Utc::now();

        // Augmenting subject_attributes with id, so that subject_key can be used in the rules.
        let augmented_subject_attributes = {
            let mut sa = subject_attributes.clone();
            sa.entry("id".into()).or_insert_with(|| subject_key.into());
            sa
        };

        let (allocation, split) = self.allocations.iter().find_map(|allocation| {
            allocation
                .get_matching_split(
                    subject_key,
                    &augmented_subject_attributes,
                    sharder,
                    self.total_shards,
                    now,
                )
                .map(|split| (allocation, split))
        })?;

        let variation = self.variations.get(&split.variation_key)?;
        let assignment_value = variation.value.to_assignment_value(self.variation_type)?;
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

        Some((assignment_value, event))
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
        let forbidden =
            self.start_at.is_some_and(|t| now < t) || self.end_at.is_some_and(|t| now > t);
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
