use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::bandits::{
    BanditCategoricalAttributeCoefficient, BanditModelData, BanditNumericAttributeCoefficient,
};
use crate::error::EvaluationFailure;
use crate::events::{AssignmentEvent, BanditEvent};
use crate::sharder::get_md5_shard;
use crate::ufc::{Assignment, AssignmentValue, VariationType};
use crate::{Configuration, EvaluationError, Str};
use crate::{ContextAttributes, SdkMetadata};

use super::eval_assignment::get_assignment_with_visitor;
use super::eval_details::EvaluationDetails;
use super::eval_details_builder::EvalDetailsBuilder;
use super::eval_visitor::{EvalBanditVisitor, NoopEvalVisitor};

#[derive(Debug)]
pub(super) struct BanditEvaluationDetails {
    /// Selected action.
    pub(super) action_key: Str,
    pub(super) action_weight: f64,
    /// Distance between best and selected actions' scores.
    pub(super) optimality_gap: f64,
}

struct Action<'a> {
    key: &'a str,
    attributes: &'a ContextAttributes,
}

/// Result of evaluating a bandit.
#[derive(Debug, Clone, Serialize)]
pub struct BanditResult {
    /// Selected variation from the feature flag.
    pub variation: Str,
    /// Selected action if any.
    pub action: Option<Str>,
    /// Flag assignment event that needs to be logged to analytics storage.
    pub assignment_event: Option<AssignmentEvent>,
    /// Bandit assignment event that needs to be logged to analytics storage.
    pub bandit_event: Option<BanditEvent>,
}

/// Evaluate the specified string feature flag for the given subject. If resulting variation is
/// a bandit, evaluate the bandit to return the action.
pub fn get_bandit_action(
    configuration: Option<&Configuration>,
    flag_key: &str,
    subject_key: &Str,
    subject_attributes: &ContextAttributes,
    actions: &HashMap<Str, ContextAttributes>,
    default_variation: &Str,
    now: DateTime<Utc>,
    sdk_meta: &SdkMetadata,
) -> BanditResult {
    get_bandit_action_with_visitor(
        &mut NoopEvalVisitor,
        configuration,
        flag_key,
        subject_key,
        subject_attributes,
        actions,
        default_variation,
        now,
        sdk_meta,
    )
}

/// Evaluate the specified string feature flag for the given subject. If resulting variation is
/// a bandit, evaluate the bandit to return the action. In addition, return evaluation details.
pub fn get_bandit_action_details(
    configuration: Option<&Configuration>,
    flag_key: &str,
    subject_key: &Str,
    subject_attributes: &ContextAttributes,
    actions: &HashMap<Str, ContextAttributes>,
    default_variation: &Str,
    now: DateTime<Utc>,
    sdk_meta: &SdkMetadata,
) -> (BanditResult, EvaluationDetails) {
    let mut builder = EvalDetailsBuilder::new(
        flag_key.to_owned(),
        subject_key.to_owned(),
        subject_attributes.to_generic_attributes().into(),
        now,
    );
    let result = get_bandit_action_with_visitor(
        &mut builder,
        configuration,
        flag_key,
        subject_key,
        subject_attributes,
        actions,
        default_variation,
        now,
        sdk_meta,
    );
    let details = builder.build();
    (result, details)
}

/// Evaluate the specified string feature flag for the given subject. If resulting variation is
/// a bandit, evaluate the bandit to return the action.
fn get_bandit_action_with_visitor<V: EvalBanditVisitor>(
    visitor: &mut V,
    configuration: Option<&Configuration>,
    flag_key: &str,
    subject_key: &Str,
    subject_attributes: &ContextAttributes,
    actions: &HashMap<Str, ContextAttributes>,
    default_variation: &Str,
    now: DateTime<Utc>,
    sdk_meta: &SdkMetadata,
) -> BanditResult {
    let Some(configuration) = configuration else {
        let result = BanditResult {
            variation: default_variation.clone(),
            action: None,
            assignment_event: None,
            bandit_event: None,
        };
        visitor.on_result(Err(EvaluationFailure::ConfigurationMissing), &result);
        return result;
    };

    visitor.on_configuration(configuration);

    let assignment = get_assignment_with_visitor(
        Some(configuration),
        &mut visitor.visit_assignment(),
        flag_key,
        subject_key,
        &Arc::new(subject_attributes.to_generic_attributes()),
        Some(VariationType::String),
        now,
    )
    .unwrap_or_default()
    .unwrap_or_else(|| Assignment {
        value: AssignmentValue::String(default_variation.clone()),
        event: None,
    });

    let variation = assignment
        .value
        .to_string()
        .expect("flag assignment in bandit evaluation is always a string");

    let Some(bandit_key) = configuration.get_bandit_key(flag_key, &variation) else {
        // It's not a bandit variation, just return it.
        let result = BanditResult {
            variation,
            action: None,
            assignment_event: assignment.event,
            bandit_event: None,
        };
        visitor.on_result(Err(EvaluationFailure::NonBanditVariation), &result);
        return result;
    };

    visitor.on_bandit_key(bandit_key);

    let Some(bandit) = configuration.get_bandit(bandit_key) else {
        // We've evaluated a flag that resulted in a bandit but now we cannot find the bandit
        // configuration and we cannot proceed.
        //
        // This should normally never happen as it means that there's a mismatch between the
        // general UFC config and bandits config.
        log::warn!(target: "eppo", bandit_key; "unable to find bandit configuration");
        let result = BanditResult {
            variation,
            action: None,
            assignment_event: assignment.event,
            bandit_event: None,
        };
        visitor.on_result(
            Err(EvaluationFailure::Error(
                EvaluationError::UnexpectedConfigurationError,
            )),
            &result,
        );
        return result;
    };

    let evaluation =
        match bandit
            .model_data
            .evaluate(flag_key, subject_key, subject_attributes, actions)
        {
            Ok(evaluation) => evaluation,
            Err(err) => {
                // We've evaluated a flag but now bandit evaluation failed. (Likely to user supplying
                // empty actions, or NaN attributes.)
                //
                // Abort evaluation and return default variant.
                let result = BanditResult {
                    variation,
                    action: None,
                    assignment_event: assignment.event,
                    bandit_event: None,
                };
                visitor.on_result(Err(err), &result);
                return result;
            }
        };

    let action_attributes = &actions[&evaluation.action_key];
    let bandit_event = BanditEvent {
        flag_key: flag_key.into(),
        bandit_key: bandit_key.clone(),
        subject: subject_key.clone(),
        action: evaluation.action_key.clone(),
        action_probability: evaluation.action_weight,
        optimality_gap: evaluation.optimality_gap,
        model_version: bandit.model_version.clone(),
        timestamp: now.to_rfc3339(),
        subject_numeric_attributes: subject_attributes.numeric.clone(),
        subject_categorical_attributes: subject_attributes.categorical.clone(),
        action_numeric_attributes: action_attributes.numeric.clone(),
        action_categorical_attributes: action_attributes.categorical.clone(),
        meta_data: sdk_meta.into(),
    };

    let result = BanditResult {
        variation,
        action: Some(evaluation.action_key),
        assignment_event: assignment.event,
        bandit_event: Some(bandit_event),
    };
    visitor.on_result(Ok(()), &result);
    return result;
}

impl BanditModelData {
    // Exported to super, so we can use it in precomputed evaluation.
    pub(super) fn evaluate(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &ContextAttributes,
        actions: &HashMap<Str, ContextAttributes>,
    ) -> Result<BanditEvaluationDetails, EvaluationFailure> {
        // total_shards is not configurable at the moment.
        const TOTAL_SHARDS: u32 = 10_000;

        if actions.len() == 0 {
            return Err(EvaluationFailure::NoActionsSuppliedForBandit);
        }

        let scores = actions
            .iter()
            .map(|(key, attributes)| {
                (
                    key,
                    self.score_action(Action { key, attributes }, subject_attributes),
                )
            })
            .collect::<HashMap<_, _>>();

        let best = scores
            .iter()
            .max_by(|a, b| {
                f64::total_cmp(a.1, b.1).then_with(|| {
                    // In the case of multiple actions getting the same best score, we need to break
                    // the tie deterministically.
                    //
                    // Compare action names next.
                    //
                    // We're reversing the comparison, so that before-ordered name is considered
                    // higher and wins the best score.
                    Ord::cmp(a.0, b.0).reverse()
                })
            })
            .map(|(k, v)| (*k, *v))
            .ok_or_else(|| {
                debug_assert!(false, "scores should contain at least one action");
                EvaluationFailure::NoActionsSuppliedForBandit
            })?;

        let weights = self.weigh_actions(&scores, best);

        // Pseudo-random deterministic shuffle of actions. Shuffling is unique per subject, so when
        // weights change slightly, large swatches of subjects are not reassign from one action to
        // the same other action (instead, if subject is pushed away from an action, it will get
        // assigned to a pseudo-random other action).
        let shuffled_actions = {
            let mut shuffled_actions = actions.keys().collect::<Vec<_>>();
            // Sort actions by their shard value. Use action key as tie breaker.
            shuffled_actions.sort_by_cached_key(|&action_key| {
                let hash =
                    get_md5_shard(&[flag_key, "-", subject_key, "-", action_key], TOTAL_SHARDS);
                (hash, action_key)
            });
            shuffled_actions
        };

        let selection_hash = (get_md5_shard(&[flag_key, "-", subject_key], TOTAL_SHARDS) as f64)
            / (TOTAL_SHARDS as f64);

        let selected_action = {
            let mut cumulative_weight = 0.0;
            *shuffled_actions
                .iter()
                .find(|&action_key| {
                    cumulative_weight += weights[action_key];
                    cumulative_weight > selection_hash
                })
                .or_else(|| shuffled_actions.last())
                .ok_or_else(|| {
                    debug_assert!(false, "shuffled_actions should contain at least one action");
                    EvaluationFailure::NoActionsSuppliedForBandit
                })?
        };

        let optimality_gap = best.1 - scores[selected_action];

        Ok(BanditEvaluationDetails {
            action_key: selected_action.to_owned(),
            // action_attributes: actions[selected_action].to_owned(),
            action_weight: weights[selected_action],
            optimality_gap,
        })
    }

    /// Weigh actions depending on their scores. Higher-scored actions receive more weight, except
    /// best action which receive the remainder weight.
    fn weigh_actions<'a>(
        &self,
        scores: &HashMap<&'a Str, f64>,
        (best_action, best_score): (&'a Str, f64),
    ) -> HashMap<&'a Str, f64> {
        let mut weights = HashMap::<&Str, f64>::new();

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
                .get(coef.attribute_key.as_str())
                .cloned()
                .map(f64::from)
                // fend against infinite/NaN attributes as they poison the calculation down the line
                .filter(|n| n.is_finite())
                .map(|value| value * coef.coefficient)
                .unwrap_or(coef.missing_value_coefficient)
        })
        .chain(categorical_coefficients.into_iter().map(|coef| {
            attributes
                .categorical
                .get(coef.attribute_key.as_str())
                .and_then(|value| coef.value_coefficients.get(value.to_str().as_ref()))
                .copied()
                .unwrap_or(coef.missing_value_coefficient)
        }))
        .sum()
}

#[cfg(test)]
mod tests {
    use std::fs::{read_dir, File};

    use chrono::Utc;
    use serde::{Deserialize, Serialize};

    use crate::{
        eval::get_bandit_action, ufc::UniversalFlagConfig, Configuration, ContextAttributes,
        SdkMetadata, Str,
    };

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestFile {
        flag: String,
        default_value: Str,
        subjects: Vec<TestSubject>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestSubject {
        subject_key: Str,
        subject_attributes: ContextAttributes,
        actions: Vec<TestAction>,
        assignment: TestAssignment,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestAction {
        action_key: Str,
        #[serde(flatten)]
        attributes: ContextAttributes,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    struct TestAssignment {
        variation: Str,
        action: Option<Str>,
    }

    #[test]
    fn sdk_test_data() {
        let config = UniversalFlagConfig::from_json(
            SdkMetadata {
                name: "test",
                version: "0.1.0",
            },
            std::fs::read("../sdk-test-data/ufc/bandit-flags-v1.json").unwrap(),
        )
        .unwrap();
        let bandits = serde_json::from_reader(
            File::open("../sdk-test-data/ufc/bandit-models-v1.json").unwrap(),
        )
        .unwrap();

        let config = Configuration::from_server_response(config, Some(bandits));

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

                let result = get_bandit_action(
                    Some(&config),
                    &test.flag,
                    &subject.subject_key,
                    &subject.subject_attributes.into(),
                    &actions,
                    &test.default_value,
                    Utc::now(),
                    &SdkMetadata {
                        name: "test",
                        version: "0.1.0",
                    },
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
