use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::eval::get_assignment;
use crate::precomputed::PrecomputedConfiguration;
use crate::ufc::ConfigurationFormat;
use crate::{Attributes, Configuration, Str};

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
            obfuscated: serde_bool::False,
            format: ConfigurationFormat::Precomputed,
            created_at: now,
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
        obfuscated: serde_bool::False,
        created_at: now,
        format: ConfigurationFormat::Precomputed,
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
