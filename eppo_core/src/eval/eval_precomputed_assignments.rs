use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::eval::get_assignment;
use crate::ufc::{Assignment, AssignmentFormat, Environment, ValueWire, VariationType};
use crate::{Attributes, Configuration, Str};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrecomputedConfiguration {
    created_at: DateTime<Utc>,
    /// `format` is always `AssignmentFormat::Precomputed`.
    format: AssignmentFormat,
    // Environment might be missing if configuration was absent during evaluation.
    environment: Option<Environment>,
    flags: HashMap<String, PrecomputedAssignment>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrecomputedAssignment {
    variation_type: VariationType,
    variation_value: ValueWire,

    do_log: bool,
    // If `do_log` is false, the client doesn’t need the field below.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    allocation_key: Option<Str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    variation_key: Option<Str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    extra_logging: Option<HashMap<String, String>>,
}

impl From<Assignment> for PrecomputedAssignment {
    fn from(assignment: Assignment) -> PrecomputedAssignment {
        match assignment.event {
            Some(event) => PrecomputedAssignment {
                variation_type: assignment.value.variation_type(),
                variation_value: assignment.value.variation_value(),
                do_log: true,
                allocation_key: Some(event.base.allocation.clone()),
                variation_key: Some(event.base.variation.clone()),
                extra_logging: Some(event.base.extra_logging.clone()),
            },
            None => PrecomputedAssignment {
                variation_type: assignment.value.variation_type(),
                variation_value: assignment.value.variation_value(),
                do_log: false,
                allocation_key: None,
                variation_key: None,
                extra_logging: None,
            },
        }
    }
}

pub fn get_precomputed_assignments(
    configuration: Option<&Configuration>,
    subject_key: &Str,
    subject_attributes: &Arc<Attributes>,
    now: DateTime<Utc>,
) -> PrecomputedConfiguration {
    let Some(configuration) = configuration else {
        log::warn!(target: "eppo",
                   subject_key;
                   "evaluating a flag before Eppo configuration has been fetched");
        return PrecomputedConfiguration {
            created_at: now,
            format: AssignmentFormat::Precomputed,
            environment: None,
            flags: HashMap::new(),
        };
    };

    let mut flags = HashMap::new();

    for key in configuration.flags.compiled.flags.keys() {
        match get_assignment(
            Some(configuration),
            key,
            &subject_key,
            &subject_attributes,
            None,
            now,
        ) {
            Ok(Some(assignment)) => {
                flags.insert(key.clone(), assignment.into());
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("Failed to evaluate assignment for key {}: {:?}", key, e);
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
        environment: Some(configuration.flags.compiled.environment.clone()),
        flags,
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::{
        eval::get_precomputed_assignments, ufc::UniversalFlagConfig, Attributes, Configuration,
        SdkMetadata,
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
}
