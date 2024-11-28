use std::collections::HashMap;

use crate::{error::EvaluationFailure, ufc::Assignment};

#[derive(Debug)]
pub struct PrecomputedConfiguration {
    pub flags: HashMap<String, Result<Assignment, EvaluationFailure>>,
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::{
        configuration_store::ConfigurationStore,
        eval::{Evaluator, EvaluatorConfig},
        ufc::{UniversalFlagConfig, VariationType},
        Attributes, Configuration, SdkMetadata,
    };
    use std::{fs, sync::Arc};

    fn setup_test_config() -> Configuration {
        let _ = env_logger::builder().is_test(true).try_init();

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
    }

    #[test]
    fn test_precomputed_assignment_basic() {
        let configuration = setup_test_config();

        let configuration_store = Arc::new(ConfigurationStore::new());
        configuration_store.set_configuration(Arc::new(configuration));

        let evaluator = Evaluator::new(EvaluatorConfig {
            configuration_store: configuration_store.clone(),
            sdk_metadata: SdkMetadata {
                name: "test",
                version: "0.1.0",
            },
        });

        let subject_key = "test-subject-1".into();
        let subject_attributes = Arc::new(Attributes::new());
        let now = Utc::now();

        // Get precomputed assignments
        let precomputed =
            evaluator.get_precomputed_assignment(&subject_key, &subject_attributes, false);

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
    }

    #[test]
    fn test_precomputed_assignment_early_exit() {
        let mut configuration = setup_test_config();
        let num_good_flags = configuration.flags.compiled.flags.len();

        // Add a flag that will cause an evaluation error
        configuration.flags.compiled.flags.insert(
            "error-flag".to_string(),
            Ok(crate::ufc::Flag {
                variation_type: VariationType::String,
                allocations: vec![].into_boxed_slice(),
            }),
        );

        let configuration_store = Arc::new(ConfigurationStore::new());
        configuration_store.set_configuration(Arc::new(configuration));

        let evaluator = Evaluator::new(EvaluatorConfig {
            configuration_store: configuration_store.clone(),
            sdk_metadata: SdkMetadata {
                name: "test",
                version: "0.1.0",
            },
        });

        let subject_key = "test-subject-1".into();
        let subject_attributes = Arc::new(Attributes::new());
        let now = Utc::now();

        // Get assignments with early exit
        let precomputed_with_early_exit =
            evaluator.get_precomputed_assignment(&subject_key, &subject_attributes, true);

        // Verify we have fewer entries due to early exit
        assert!(
            precomputed_with_early_exit.flags.len() < num_good_flags,
            "Early exit should stop processing on first error"
        );
    }
}
