use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::eval::get_assignment;
use crate::precomputed::{PrecomputedAssignment, PrecomputedBandit, PrecomputedConfiguration};
use crate::ufc::{ConfigurationFormat, ValueWire, VariationType};
use crate::{Configuration, ContextAttributes, Str};

pub fn get_precomputed_configuration(
    configuration: Option<&Configuration>,
    subject_key: &Str,
    subject_attributes: &Arc<ContextAttributes>,
    flag_actions: &HashMap<
        /* flag_key: */ Str,
        HashMap</* action_key: */ Str, ContextAttributes>,
    >,
    now: DateTime<Utc>,
) -> PrecomputedConfiguration {
    let Some(configuration) = configuration else {
        log::warn!(target: "eppo",
                   subject_key;
                   "evaluating a flag before Eppo configuration has been fetched");
        return PrecomputedConfiguration {
            obfuscated: serde_bool::False,
            format: ConfigurationFormat::Precomputed,
            created_at: now,
            environment: None,
            flags: HashMap::new(),
            bandits: HashMap::new(),
        };
    };

    let generic_attributes = Arc::new(subject_attributes.to_generic_attributes());

    let flags = configuration
        .flags
        .compiled
        .flags
        .keys()
        .filter_map(|flag_key| {
            get_assignment(
                Some(configuration),
                flag_key,
                &subject_key,
                &generic_attributes,
                None,
                now,
            )
            .unwrap_or_else(|err| {
                log::warn!(
                    target: "eppo",
                    subject_key,
                    flag_key,
                    err:?;
                    "Failed to evaluate assignment"
                );
                None
            })
            .map(|assignment| ((flag_key.clone(), PrecomputedAssignment::from(assignment))))
        })
        .collect::<HashMap<_, _>>();

    let bandits = configuration
        .bandits
        .as_ref()
        .map(|bandits| {
            flag_actions
                .iter()
                .filter_map(|(flag_key, actions)| {
                    let flag = flags.get(flag_key)?;

                    // Skip non-string variations as they can't be bandits.
                    if flag.variation_type != VariationType::String {
                        return None;
                    }

                    let flag_bandits: HashMap</* variation_key: */ Str, PrecomputedBandit> =
                        if let Some(ValueWire::String(precomputed_variation_value)) = flags
                            .get(flag_key)
                            .map(|assignment| &assignment.variation_value)
                        {
                            // If precomputing flag resolved to a value, we only need to evaluate a
                            // single bandit.
                            let bandit_key = &configuration
                                .flags
                                .compiled
                                .flag_to_bandit_associations
                                .get(flag_key)?
                                .get(precomputed_variation_value)?
                                .key;
                            let bandit_model = bandits.bandits.get(bandit_key)?;

                            let bandit_evaluation = bandit_model
                                .model_data
                                .evaluate(flag_key, subject_key, subject_attributes, actions)
                                .ok()?;

                            let selected_action = &actions[&bandit_evaluation.action_key];
                            let precomputed_bandit = PrecomputedBandit {
                                bandit_key: bandit_key.clone(),
                                action: bandit_evaluation.action_key,
                                action_probability: bandit_evaluation.action_weight,
                                optimality_gap: bandit_evaluation.optimality_gap,
                                model_version: bandit_model.model_version.clone(),
                                action_numeric_attributes: selected_action.numeric.clone(),
                                action_categorical_attributes: selected_action.categorical.clone(),
                            };

                            [(precomputed_variation_value.clone(), precomputed_bandit)]
                                .into_iter()
                                .collect()
                        } else {
                            // If precomputed flag did not resolve to a value, we need to precompute all
                            // bandits for the flag in case the user supplies a bandit variation as
                            // default variation.
                            configuration
                                .flags
                                .compiled
                                .flag_to_bandit_associations
                                .get(flag_key)?
                                .iter()
                                .filter_map(|(variation_value, bandit_variation)| {
                                    let bandit_key = &bandit_variation.key;
                                    let bandit_model = bandits.bandits.get(bandit_key)?;

                                    let bandit_evaluation = bandit_model
                                        .model_data
                                        .evaluate(
                                            flag_key,
                                            subject_key,
                                            subject_attributes,
                                            actions,
                                        )
                                        .ok()?;

                                    let selected_action = &actions[&bandit_evaluation.action_key];
                                    let precomputed_bandit = PrecomputedBandit {
                                        bandit_key: bandit_key.clone(),
                                        action: bandit_evaluation.action_key,
                                        action_probability: bandit_evaluation.action_weight,
                                        optimality_gap: bandit_evaluation.optimality_gap,
                                        model_version: bandit_model.model_version.clone(),
                                        action_numeric_attributes: selected_action.numeric.clone(),
                                        action_categorical_attributes: selected_action
                                            .categorical
                                            .clone(),
                                    };

                                    Some((variation_value.clone(), precomputed_bandit))
                                })
                                .collect()
                        };

                    Some((flag_key.clone(), flag_bandits))
                })
                .collect()
        })
        .unwrap_or_default();

    let result = PrecomputedConfiguration {
        obfuscated: serde_bool::False,
        created_at: now,
        format: ConfigurationFormat::Precomputed,
        environment: Some(configuration.flags.compiled.environment.clone()),
        flags,
        bandits,
    };

    log::trace!(
        target: "eppo",
        subject_key,
        configuration:serde = result;
        "evaluated precomputed assignments");

    result
}

#[cfg(test)]
mod tests {
    use crate::Str;
    use chrono::Utc;
    use std::collections::HashMap;

    use crate::{
        eval::get_precomputed_configuration, ufc::UniversalFlagConfig, Configuration,
        ContextAttributes, SdkMetadata,
    };
    use std::{fs, sync::Arc};

    #[test]
    fn test_precomputed_assignment_basic() {
        let _ = env_logger::builder().is_test(true).try_init();

        let configuration = {
            // Load test configuration
            let ufc_config = UniversalFlagConfig::from_json(
                SdkMetadata {
                    name: "test",
                    version: "0.1.0",
                },
                fs::read("../sdk-test-data/ufc/flags-v1.json").unwrap(),
            )
            .unwrap();
            Configuration::from_server_response(ufc_config, None)
        };

        let subject_key = "test-subject-1".into();
        let subject_attributes = Default::default();
        let actions = Default::default();
        let now = Utc::now();

        // Get precomputed assignments
        let precomputed = get_precomputed_configuration(
            Some(&configuration),
            &subject_key,
            &subject_attributes,
            &actions,
            now,
        );

        assert!(
            !precomputed.flags.is_empty(),
            "Should have precomputed flags"
        );

        // Each flag in the configuration should have an entry
        for flag_key in precomputed.flags.keys() {
            assert!(
                precomputed.flags.contains_key(flag_key),
                "Should have precomputed assignment for flag {}",
                flag_key
            );
        }

        // Uncomment next section to dump configuration to console.
        // eprintln!(
        //     "{}",
        //     serde_json::to_string_pretty(&precomputed.obfuscate()).unwrap()
        // );
        // assert!(false);
    }

    #[test]
    fn test_precomputed_assignment_bandits() {
        let _ = env_logger::builder().is_test(true).try_init();

        let configuration = {
            // Load test configuration
            let ufc_config = UniversalFlagConfig::from_json(
                SdkMetadata {
                    name: "test",
                    version: "0.1.0",
                },
                fs::read("../sdk-test-data/ufc/bandit-flags-v1.json").unwrap(),
            )
            .unwrap();
            let bandits_config = serde_json::from_slice(
                &fs::read("../sdk-test-data/ufc/bandit-models-v1.json").unwrap(),
            )
            .unwrap();
            Configuration::from_server_response(ufc_config, Some(bandits_config))
        };

        let subject_key = "test-subject-1".into();
        let subject_attributes = Default::default();
        let mut actions = HashMap::new();

        // Add actions for car bandit flag
        let car_actions: HashMap<Str, ContextAttributes> = [
            ("dodge".into(), Default::default()),
            ("mercedes".into(), Default::default()),
            (
                "toyota".into(),
                ContextAttributes {
                    numeric: Arc::new([("speed".into(), (1000.0).into())].into_iter().collect()),
                    categorical: Default::default(),
                },
            ),
        ]
        .into_iter()
        .collect();
        actions.insert("car_bandit_flag".into(), car_actions);

        let now = Utc::now();

        // Get precomputed assignments
        let precomputed = get_precomputed_configuration(
            Some(&configuration),
            &subject_key,
            &subject_attributes,
            &actions,
            now,
        );

        assert!(
            !precomputed.flags.is_empty(),
            "Should have precomputed flags"
        );

        // Each flag in the configuration should have an entry
        for flag_key in precomputed.flags.keys() {
            assert!(
                precomputed.flags.contains_key(flag_key),
                "Should have precomputed assignment for flag {}",
                flag_key
            );
        }

        // Uncomment next section to dump configuration to console.
        // eprintln!(
        //     "{}",
        //     serde_json::to_string_pretty(&precomputed.obfuscate()).unwrap()
        // );
        // assert!(false);
    }

    #[test]
    fn test_precomputed_assignment_with_and_without_actions() {
        let _ = env_logger::builder().is_test(true).try_init();

        let configuration = {
            let ufc_config = UniversalFlagConfig::from_json(
                SdkMetadata {
                    name: "test",
                    version: "0.1.0",
                },
                fs::read("../sdk-test-data/ufc/bandit-flags-v1.json").unwrap(),
            )
            .unwrap();
            let bandits_config = serde_json::from_slice(
                &fs::read("../sdk-test-data/ufc/bandit-models-v1.json").unwrap(),
            )
            .unwrap();
            Configuration::from_server_response(ufc_config, Some(bandits_config))
        };

        let subject_key = "test-subject-1".into();
        let subject_attributes = Arc::new(ContextAttributes {
            numeric: Default::default(),
            categorical: Arc::new([("country".into(), "UK".into())].into_iter().collect()),
        });
        let now = Utc::now();

        // Case 1: No actions provided
        let precomputed_no_actions = get_precomputed_configuration(
            Some(&configuration),
            &subject_key,
            &subject_attributes,
            &HashMap::new(),
            now,
        );
        // Validate Case 1
        assert!(
            precomputed_no_actions.bandits.is_empty(),
            "Should have no bandit data when no actions provided"
        );

        // Case 2: Only car bandit actions provided
        let mut car_only_actions = HashMap::new();
        let car_actions: HashMap<Str, ContextAttributes> = [(
            "toyota".into(),
            ContextAttributes {
                numeric: Arc::new([("speed".into(), (1000.0).into())].into_iter().collect()),
                categorical: Default::default(),
            },
        )]
        .into_iter()
        .collect();
        car_only_actions.insert("car_bandit_flag".into(), car_actions);

        let precomputed_car_only = get_precomputed_configuration(
            Some(&configuration),
            &subject_key,
            &subject_attributes,
            &car_only_actions,
            now,
        );
        // Validate Case 2
        assert!(
            precomputed_car_only.bandits.contains_key("car_bandit_flag"),
            "Should have car bandit data when car actions provided"
        );
        assert!(
            !precomputed_car_only
                .bandits
                .contains_key("banner_bandit_flag"),
            "Should not have banner bandit data when only car actions provided"
        );
        assert!(
            !precomputed_car_only
                .bandits
                .contains_key("banner_bandit_flag_uk_only"),
            "Should not have UK banner bandit data when only car actions provided"
        );

        // Case 3: Only banner bandit actions provided
        let mut banner_only_actions = HashMap::new();
        let banner_actions: HashMap<Str, ContextAttributes> = [
            ("nike".into(), Default::default()),
            ("adidas".into(), Default::default()),
        ]
        .into_iter()
        .collect();
        banner_only_actions.insert("banner_bandit_flag".into(), banner_actions.clone());
        banner_only_actions.insert("banner_bandit_flag_uk_only".into(), banner_actions);

        let precomputed_banner_only = get_precomputed_configuration(
            Some(&configuration),
            &subject_key,
            &subject_attributes,
            &banner_only_actions,
            now,
        );
        // Validate Case 3
        assert!(
            !precomputed_banner_only
                .bandits
                .contains_key("car_bandit_flag"),
            "Should not have car bandit data when only banner actions provided"
        );
        assert!(
            precomputed_banner_only
                .bandits
                .contains_key("banner_bandit_flag"),
            "Should have banner bandit data when banner actions provided"
        );
        assert!(
            precomputed_banner_only
                .bandits
                .contains_key("banner_bandit_flag_uk_only"),
            "Should have UK banner bandit data when banner actions provided"
        );
    }
}
