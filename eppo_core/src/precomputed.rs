use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::obfuscation::{Base64Str, Md5HashedStr};
use crate::timestamp::Timestamp;
use crate::ufc::{Assignment, ConfigurationFormat, Environment, ValueWire, VariationType};
use crate::{CategoricalAttribute, NumericAttribute, Str};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrecomputedConfiguration {
    pub(crate) obfuscated: serde_bool::False,
    pub(crate) created_at: Timestamp,
    /// `format` is always `AssignmentFormat::Precomputed`.
    pub(crate) format: ConfigurationFormat,
    // Environment might be missing if configuration was absent during evaluation.
    pub(crate) environment: Option<Environment>,
    pub(crate) flags: HashMap</* flag_key: */ Str, PrecomputedAssignment>,
    pub(crate) bandits:
        HashMap</* flag_key: */ Str, HashMap</* variation_value: */ Str, PrecomputedBandit>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PrecomputedAssignment {
    pub(crate) variation_type: VariationType,
    pub(crate) variation_value: ValueWire,

    pub(crate) do_log: bool,
    // If `do_log` is false, the client doesn’t need the field below.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) allocation_key: Option<Str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) variation_key: Option<Str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) extra_logging: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PrecomputedBandit {
    pub(crate) bandit_key: Str,
    pub(crate) action: Str,
    pub(crate) action_probability: f64,
    pub(crate) optimality_gap: f64,
    pub(crate) model_version: Str,
    pub(crate) action_numeric_attributes: Arc<HashMap<Str, NumericAttribute>>,
    pub(crate) action_categorical_attributes: Arc<HashMap<Str, CategoricalAttribute>>,
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

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObfuscatedPrecomputedConfiguration {
    obfuscated: serde_bool::True,
    /// `format` is always `AssignmentFormat::Precomputed`.
    format: ConfigurationFormat,
    /// Salt used for hashing md5-encoded strings.
    salt: Str,
    created_at: Timestamp,
    // Environment might be missing if configuration was absent during evaluation.
    environment: Option<Environment>,
    flags: HashMap<Md5HashedStr, ObfuscatedPrecomputedAssignment>,
    bandits: HashMap<
        /* flag_key: */ Md5HashedStr,
        HashMap</* variation_value: */ Md5HashedStr, ObfuscatedPrecomputedBandit>,
    >,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObfuscatedPrecomputedAssignment {
    variation_type: VariationType,
    variation_value: Base64Str,
    do_log: bool,
    // If `do_log` is false, the client doesn’t need the fields below.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    allocation_key: Option<Base64Str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    variation_key: Option<Base64Str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    extra_logging: Option<HashMap<Base64Str, Base64Str>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObfuscatedPrecomputedBandit {
    bandit_key: Base64Str,
    action: Base64Str,
    action_probability: f64,
    optimality_gap: f64,
    model_version: Base64Str,
    action_numeric_attributes: HashMap<Base64Str, Base64Str>,
    action_categorical_attributes: HashMap<Base64Str, Base64Str>,
}

impl PrecomputedConfiguration {
    pub fn obfuscate(self) -> ObfuscatedPrecomputedConfiguration {
        self.into()
    }
}

impl From<PrecomputedConfiguration> for ObfuscatedPrecomputedConfiguration {
    fn from(config: PrecomputedConfiguration) -> Self {
        let salt: Str = {
            let bytes = rand::thread_rng().gen::<[u8; 16]>();
            base64::prelude::BASE64_STANDARD_NO_PAD
                .encode(&bytes)
                .into()
        };
        ObfuscatedPrecomputedConfiguration {
            obfuscated: serde_bool::True,
            format: ConfigurationFormat::Precomputed,
            created_at: config.created_at,
            environment: config.environment,
            flags: config
                .flags
                .into_iter()
                .map(|(k, v)| {
                    (
                        Md5HashedStr::new(salt.as_bytes(), k.as_bytes()),
                        ObfuscatedPrecomputedAssignment::from(v),
                    )
                })
                .collect(),
            bandits: config
                .bandits
                .into_iter()
                .map(|(k, v)| {
                    (
                        Md5HashedStr::new(salt.as_bytes(), k.as_bytes()),
                        v.into_iter()
                            .map(|(k, v)| {
                                (Md5HashedStr::new(salt.as_bytes(), k.as_bytes()), v.into())
                            })
                            .collect(),
                    )
                })
                .collect(),
            salt,
        }
    }
}

impl From<PrecomputedAssignment> for ObfuscatedPrecomputedAssignment {
    fn from(value: PrecomputedAssignment) -> Self {
        ObfuscatedPrecomputedAssignment {
            variation_type: value.variation_type,
            variation_value: Base64Str::from(value.variation_value),
            do_log: value.do_log,
            allocation_key: value.allocation_key.map(Base64Str),
            variation_key: value.variation_key.map(Base64Str),
            extra_logging: value.extra_logging.map(|it| {
                it.into_iter()
                    .map(|(k, v)| (Base64Str(Str::from(k)), Base64Str(Str::from(v))))
                    .collect()
            }),
        }
    }
}

impl From<PrecomputedBandit> for ObfuscatedPrecomputedBandit {
    fn from(value: PrecomputedBandit) -> Self {
        ObfuscatedPrecomputedBandit {
            bandit_key: value.bandit_key.into(),
            action: value.action.into(),
            action_probability: value.action_probability,
            optimality_gap: value.optimality_gap,
            model_version: value.model_version.into(),
            action_numeric_attributes: value
                .action_numeric_attributes
                .iter()
                .map(|(k, v)| (k.clone().into(), v.to_f64().to_string().into()))
                .collect(),
            action_categorical_attributes: value
                .action_categorical_attributes
                .iter()
                .map(|(k, v)| (k.clone().into(), v.to_str().into()))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precomputed_obfuscation() {
        let configuration = PrecomputedConfiguration {
            obfuscated: serde_bool::False,
            format: ConfigurationFormat::Precomputed,
            created_at: crate::timestamp::now(),
            environment: Some(Environment {
                name: "Test".into(),
            }),
            flags: [(
                "test-flag".into(),
                PrecomputedAssignment {
                    variation_type: VariationType::String,
                    variation_value: ValueWire::String("hello, world!".into()),
                    do_log: true,
                    allocation_key: Some("allocation-key".into()),
                    variation_key: Some("variation-key".into()),
                    extra_logging: Some(
                        [("hello".to_owned(), "world".to_owned())]
                            .into_iter()
                            .collect(),
                    ),
                },
            )]
            .into_iter()
            .collect(),
            bandits: HashMap::new(),
        };

        let obfuscated = configuration.obfuscate();
        let flag_key = Md5HashedStr::new(obfuscated.salt.as_bytes(), b"test-flag");
        let flag = obfuscated.flags.get(&flag_key);

        assert!(flag.is_some());
        assert_eq!(
            serde_json::to_string(flag.unwrap()).unwrap(),
            r#"{"variationType":"STRING","variationValue":"aGVsbG8sIHdvcmxkIQ==","doLog":true,"allocationKey":"YWxsb2NhdGlvbi1rZXk=","variationKey":"dmFyaWF0aW9uLWtleQ==","extraLogging":{"aGVsbG8=":"d29ybGQ="}}"#
        );
    }
}
