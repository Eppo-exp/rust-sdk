use std::collections::HashMap;

use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

use crate::sharder::Md5Sharder;
use crate::sharder::Sharder;
use crate::ufc::Assignment;
use crate::ufc::AssignmentEvent;
use crate::ufc::AssignmentValue;
use crate::ufc::VariationType;
use crate::Configuration;
use crate::ContextAttributes;

use super::event::BanditEvent;
use super::BanditCategoricalAttributeCoefficient;
use super::BanditModelData;
use super::BanditNumericAttributeCoefficient;

#[derive(Debug)]
struct BanditEvaluationDetails {
    pub flag_key: String,
    pub subject_key: String,
    pub subject_attributes: ContextAttributes,
    /// Selected action.
    pub action_key: String,
    /// Attributes of the selected action.
    pub action_attributes: ContextAttributes,
    /// Score of the selected action.
    pub action_score: f64,
    pub action_weight: f64,
    pub gamma: f64,
    /// Distance between best and selected actions' scores.
    pub optimality_gap: f64,
}

struct Action<'a> {
    key: &'a str,
    attributes: &'a ContextAttributes,
}

/// Result of evaluating a bandit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanditResult {
    /// Selected variation from the feature flag.
    variation: String,
    /// Selected action if any.
    action: Option<String>,
    /// Flag assignment event that needs to be logged to analytics storage.
    assignment_event: Option<AssignmentEvent>,
    /// Bandit assignment event that needs to be logged to analytics storage.
    bandit_event: Option<BanditEvent>,
}

impl Configuration {
    /// Evaluate the specified string feature flag for the given subject. If resulting variation is
    /// a bandit, evaluate the bandit to return the action.
    pub fn get_bandit_action(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &ContextAttributes,
        actions: &HashMap<String, ContextAttributes>,
        default_variation: &str,
    ) -> BanditResult {
        let assignment = self
            .get_assignment(
                flag_key,
                subject_key,
                &subject_attributes.to_generic_attributes(),
                Some(VariationType::String),
            )
            .unwrap_or_default()
            .unwrap_or_else(|| Assignment {
                value: AssignmentValue::String(default_variation.to_owned()),
                event: None,
            });

        let variation = assignment
            .value
            .to_string()
            .expect("flag assignment in bandit evaluation is always a string");

        let Some(bandit_key) = self.get_bandit_key(flag_key, &variation) else {
            // It's not a bandit variation, just return it.
            return BanditResult {
                variation,
                action: None,
                assignment_event: assignment.event,
                bandit_event: None,
            };
        };

        let Some(bandit) = self.get_bandit(bandit_key) else {
            // We've evaluated a flag that resulted in a bandit but now we cannot find the bandit
            // configuration and we cannot proceed.
            //
            // This should normally never happen as it means that there's a mismatch between the
            // general UFC config and bandits config.
            //
            // Abort evaluation and return default variant, ignoring `assignment.event` logging.
            log::warn!(target: "eppo", bandit_key; "unable to find bandit configuration");
            return BanditResult {
                variation: default_variation.to_owned(),
                action: None,
                assignment_event: None,
                bandit_event: None,
            };
        };

        let Some(evaluation) =
            bandit
                .model_data
                .evaluate(flag_key, subject_key, subject_attributes, actions)
        else {
            // We've evaluated a flag but now bandit evaluation failed. (Likely to user supplying
            // empty actions, or NaN attributes.)
            //
            // Abort evaluation and return default variant, ignoring `assignment.event` logging.
            return BanditResult {
                variation: default_variation.to_owned(),
                action: None,
                assignment_event: None,
                bandit_event: None,
            };
        };

        let bandit_event = BanditEvent {
            flag_key: flag_key.to_owned(),
            bandit_key: bandit_key.to_owned(),
            subject: subject_key.to_owned(),
            action: evaluation.action_key.clone(),
            action_probability: evaluation.action_weight,
            optimality_gap: evaluation.optimality_gap,
            model_version: bandit.model_version.clone(),
            timestamp: Utc::now().to_rfc3339(),
            subject_numeric_attributes: evaluation.subject_attributes.numeric,
            subject_categorical_attributes: evaluation.subject_attributes.categorical,
            action_numeric_attributes: evaluation.action_attributes.numeric,
            action_categorical_attributes: evaluation.action_attributes.categorical,
            meta_data: [(
                "eppoCoreVersion".to_owned(),
                env!("CARGO_PKG_VERSION").to_owned(),
            )]
            .into(),
        };

        return BanditResult {
            variation,
            action: Some(evaluation.action_key),
            assignment_event: assignment.event,
            bandit_event: Some(bandit_event),
        };
    }
}

impl BanditModelData {
    fn evaluate(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &ContextAttributes,
        actions: &HashMap<String, ContextAttributes>,
    ) -> Option<BanditEvaluationDetails> {
        // total_shards is not configurable at the moment.
        const TOTAL_SHARDS: u64 = 10_000;

        let scores = actions
            .iter()
            .map(|(key, attributes)| {
                (
                    key.as_str(),
                    self.score_action(Action { key, attributes }, subject_attributes),
                )
            })
            .collect::<HashMap<_, _>>();

        let best = scores
            .iter()
            .max_by(|a, b| f64::total_cmp(a.1, b.1))
            .map(|(k, v)| (*k, *v))?;

        let weights = self.weigh_actions(&scores, best);

        // Pseudo-random deterministic shuffle of actions. Shuffling is unique per subject, so when
        // weights change slightly, large swatches of subjects are not reassign from one action to
        // the same other action (instead, if subject is pushed away from an action, it will get
        // assigned to a pseudo-random other action).
        let shuffled_actions = {
            let mut shuffled_actions = actions.keys().map(|x| x.as_str()).collect::<Vec<_>>();
            // Sort actions by their shard value. Use action key as tie breaker.
            shuffled_actions.sort_by_cached_key(|&action_key| {
                let hash = Md5Sharder.get_shard(
                    format!("{flag_key}-{subject_key}-{action_key}"),
                    TOTAL_SHARDS,
                );
                (hash, action_key)
            });
            shuffled_actions
        };

        let selection_hash =
            (Md5Sharder.get_shard(format!("{flag_key}-{subject_key}"), TOTAL_SHARDS) as f64)
                / (TOTAL_SHARDS as f64);

        let selected_action = {
            let mut cumulative_weight = 0.0;
            *shuffled_actions
                .iter()
                .find(|&action_key| {
                    cumulative_weight += weights[action_key];
                    cumulative_weight > selection_hash
                })
                .or_else(|| shuffled_actions.last())?
        };

        let optimality_gap = best.1 - scores[selected_action];

        Some(BanditEvaluationDetails {
            flag_key: flag_key.to_owned(),
            subject_key: subject_key.to_owned(),
            subject_attributes: subject_attributes.to_owned(),
            action_key: selected_action.to_owned(),
            action_attributes: actions[selected_action].to_owned(),
            action_score: scores[selected_action],
            action_weight: weights[selected_action],
            gamma: self.gamma,
            optimality_gap,
        })
    }

    /// Weigh actions depending on their scores. Higher-scored actions receive more weight, except
    /// best action which receive the remainder weight.
    fn weigh_actions<'a>(
        &self,
        scores: &HashMap<&'a str, f64>,
        (best_action, best_score): (&'a str, f64),
    ) -> HashMap<&'a str, f64> {
        let mut weights = HashMap::<&str, f64>::new();

        let n_actions = scores.len() as f64;

        let mut remainder_weight = 1.0;
        for (action, score) in scores {
            if *action != best_action {
                let min_probability = self.action_probability_floor / n_actions;
                let weight =
                    min_probability.max(1.0 / (n_actions + self.gamma * (best_score - score)));

                weights.insert(action, weight);
                remainder_weight -= weight;
            }
        }

        weights.insert(best_action, f64::max(remainder_weight, 0.0));

        weights
    }

    fn score_action(&self, action: Action, subject_attributes: &ContextAttributes) -> f64 {
        let Some(coefficients) = self.coefficients.get(action.key) else {
            return self.default_action_score;
        };

        coefficients.intercept
            + score_attributes(
                &action.attributes,
                &coefficients.action_numeric_coefficients,
                &coefficients.action_categorical_coefficients,
            )
            + score_attributes(
                subject_attributes,
                &coefficients.subject_numeric_coefficients,
                &coefficients.subject_categorical_coefficients,
            )
    }
}

fn score_attributes(
    attributes: &ContextAttributes,
    numeric_coefficients: &[BanditNumericAttributeCoefficient],
    categorical_coefficients: &[BanditCategoricalAttributeCoefficient],
) -> f64 {
    numeric_coefficients
        .into_iter()
        .map(|coef| {
            attributes
                .numeric
                .get(&coef.attribute_key)
                // fend against infinite/NaN attributes as they poison the calculation down the line
                .filter(|n| n.is_finite())
                .map(|value| value * coef.coefficient)
                .unwrap_or(coef.missing_value_coefficient)
        })
        .chain(categorical_coefficients.into_iter().map(|coef| {
            attributes
                .categorical
                .get(&coef.attribute_key)
                .and_then(|value| coef.value_coefficients.get(value))
                .copied()
                .unwrap_or(coef.missing_value_coefficient)
        }))
        .sum()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fs::{read_dir, File},
    };

    use serde::{Deserialize, Serialize};

    use crate::{Configuration, ContextAttributes};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestFile {
        flag: String,
        default_value: String,
        subjects: Vec<TestSubject>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestSubject {
        subject_key: String,
        subject_attributes: TestContextAttributes,
        actions: Vec<TestAction>,
        assignment: TestAssignment,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestContextAttributes {
        numeric_attributes: HashMap<String, f64>,
        categorical_attributes: HashMap<String, String>,
    }
    impl From<TestContextAttributes> for ContextAttributes {
        fn from(value: TestContextAttributes) -> ContextAttributes {
            ContextAttributes {
                numeric: value.numeric_attributes,
                categorical: value.categorical_attributes,
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestAction {
        action_key: String,
        #[serde(flatten)]
        attributes: TestContextAttributes,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    struct TestAssignment {
        variation: String,
        action: Option<String>,
    }

    #[test]
    fn sdk_test_data() {
        let config = serde_json::from_reader(
            File::open("../sdk-test-data/ufc/bandit-flags-v1.json").unwrap(),
        )
        .unwrap();
        let bandits = serde_json::from_reader(
            File::open("../sdk-test-data/ufc/bandit-models-v1.json").unwrap(),
        )
        .unwrap();

        let config = Configuration::new(Some(config), Some(bandits));

        for entry in read_dir("../sdk-test-data/ufc/bandit-tests/").unwrap() {
            let entry = entry.unwrap();
            println!("Processing test file: {:?}", entry.path());

            if entry
                .file_name()
                .into_string()
                .unwrap()
                .ends_with(".dynamic-typing.json")
            {
                // Not applicable to Rust as it's strongly statically typed.
                continue;
            }

            let test: TestFile = serde_json::from_reader(File::open(entry.path()).unwrap())
                .expect("cannot parse test file");

            for subject in test.subjects {
                print!("test subject {:?}... ", subject.subject_key);

                let actions = subject
                    .actions
                    .into_iter()
                    .map(|x| (x.action_key, x.attributes.into()))
                    .collect();

                let result = config.get_bandit_action(
                    &test.flag,
                    &subject.subject_key,
                    &subject.subject_attributes.into(),
                    &actions,
                    &test.default_value,
                );

                assert_eq!(
                    TestAssignment {
                        variation: result.variation,
                        action: result.action
                    },
                    subject.assignment
                );

                println!("ok")
            }
        }
    }
}
