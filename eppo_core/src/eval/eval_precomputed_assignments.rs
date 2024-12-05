use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::eval::get_assignment;
use crate::ufc::Assignment;
use crate::ufc::{AssignmentFormat, Environment};
use crate::{Attributes, Configuration, EvaluationError, Str};

#[derive(Debug)]
pub struct PrecomputedConfiguration {
    pub created_at: DateTime<Utc>,
    pub format: AssignmentFormat,
    pub environment: Environment,
    pub flags: HashMap<String, Result<Assignment, EvaluationError>>,
}

impl PrecomputedConfiguration {
    pub fn empty(environment_name: &str) -> Self {
        Self {
            created_at: Utc::now(),
            format: AssignmentFormat::Precomputed,
            environment: Environment {
                name: environment_name.into(),
            },
            flags: HashMap::new(),
        }
    }
}

pub fn get_precomputed_assignments(
    configuration: Option<&Configuration>,
    subject_key: &Str,
    subject_attributes: &Arc<Attributes>,
    early_exit: bool,
    now: DateTime<Utc>,
) -> PrecomputedConfiguration {
    if let Some(config) = configuration {
        let mut flags = HashMap::new();

        for key in config.flags.compiled.flags.keys() {
            match get_assignment(
                Some(config),
                key,
                &subject_key,
                &subject_attributes,
                None,
                now,
            ) {
                Ok(Some(assignment)) => {
                    flags.insert(key.clone(), Ok(assignment));
                }
                Ok(None) => continue,
                Err(e) => {
                    eprintln!("Failed to evaluate assignment for key {}: {:?}", key, e);
                    if early_exit {
                        break;
                    }
                }
            }
        }

        log::trace!(target: "eppo",
                      subject_key,
                      assignments:serde = flags;
                      "evaluated precomputed assignments");
        PrecomputedConfiguration {
            created_at: now,
            format: AssignmentFormat::Precomputed,
            environment: {
                Environment {
                    name: config.flags.compiled.environment.name.clone(),
                }
            },
            flags,
        }
    } else {
        log::warn!(target: "eppo",
                                 subject_key;
                                 "evaluating a flag before Eppo configuration has been fetched");
        PrecomputedConfiguration::empty("unknown")
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::{
        eval::get_precomputed_assignments,
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

        let subject_key = "test-subject-1".into();
        let subject_attributes = Arc::new(Attributes::new());
        let now = Utc::now();

        // Get precomputed assignments
        let precomputed = get_precomputed_assignments(
            Some(&configuration),
            &subject_key,
            &subject_attributes,
            false,
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

        let subject_key = "test-subject-1".into();
        let subject_attributes = Arc::new(Attributes::new());
        let now = Utc::now();

        // Get assignments with early exit
        let precomputed_with_early_exit = get_precomputed_assignments(
            Some(&configuration),
            &subject_key,
            &subject_attributes,
            true,
            now,
        );

        // Verify we have fewer entries due to early exit
        assert!(
            precomputed_with_early_exit.flags.len() < num_good_flags,
            "Early exit should stop processing on first error"
        );
    }
}
