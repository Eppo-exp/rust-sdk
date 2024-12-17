use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    error::{EvaluationError, EvaluationFailure},
    ufc::{AssignmentValue, ConditionWire, Shard},
    AttributeValue, Attributes, Str,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FlagEvaluationCode {
    /// An allocation configured for this flag was matched for any reason.
    Match,
    /// Configuration has not been fetched yet.
    ConfigurationMissing,
    /// Flag does not exist or is not enabled for the environment in use.
    FlagUnrecognizedOrDisabled,
    /// Default allocation is matched and is also serving NULL, resulting in the default value being
    /// assigned.
    DefaultAllocationNull,
    /// Variation value does not match the specified type for the function called.
    TypeMismatch,
    /// Configuration received from the server is invalid for the SDK. This should normally never
    /// happen and is likely a signal that you should update SDK.
    UnexpectedConfigurationError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BanditEvaluationCode {
    /// Found a bandit action.
    Match,
    /// Configuration has not been fetched yet.
    ConfigurationMissing,
    /// Configuration received from the server is invalid for the SDK. This should normally never
    /// happen and is likely a signal that you should update SDK.
    UnexpectedConfigurationError,
    /// Assignment evaluated to a non-bandit variation.
    NonBanditVariation,
    /// `get_bandit_action` was called without supplying actions.
    NoActionsSuppliedForBandit,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationResultWithDetails<T> {
    pub variation: Option<T>,
    pub action: Option<String>,
    pub evaluation_details: Arc<EvaluationDetails>,
}

impl<T> EvaluationResultWithDetails<T> {
    /// Map `EvaluationResultWithDetails.variation` using the `f` function.
    pub fn map<T2, F: FnOnce(T) -> T2>(self, f: F) -> EvaluationResultWithDetails<T2> {
        EvaluationResultWithDetails {
            variation: self.variation.map(f),
            action: self.action,
            evaluation_details: self.evaluation_details,
        }
    }
}

/// Details about feature flag or bandit evaluation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationDetails {
    pub flag_key: String,
    pub subject_key: Str,
    pub subject_attributes: Arc<Attributes>,
    /// Timestamp when the flag was evaluated.
    pub timestamp: DateTime<Utc>,

    /// Timestamp when configuration was fetched by the SDK. None if configuration hasn't been
    /// fetched yet.
    pub config_fetched_at: Option<DateTime<Utc>>,
    /// Timestamp when configuration was published by the server. None if configuration hasn't been
    /// fetched yet.
    pub config_published_at: Option<DateTime<Utc>>,
    /// Environment the configuration belongs to. None if configuration hasn't been fetched yet.
    pub environment_name: Option<Str>,

    pub bandit_evaluation_code: Option<BanditEvaluationCode>,
    pub flag_evaluation_code: Option<FlagEvaluationCode>,
    pub flag_evaluation_description: String,

    /// Key of the selected variation.
    pub variation_key: Option<Str>,
    /// Value of the selected variation. Could be `None` if no variation is selected, or selected
    /// value is absent in configuration (configuration error).
    pub variation_value: Option<AssignmentValue>,

    pub bandit_key: Option<Str>,
    pub bandit_action: Option<Str>,

    /// Evaluation details for all allocations.
    pub allocations: Vec<AllocationEvaluationDetails>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationEvaluationDetails {
    pub key: Str,
    /// Order position of the allocation as seen in the Web UI.
    pub order_position: usize,
    pub allocation_evaluation_code: AllocationEvaluationCode,
    pub evaluated_rules: Vec<RuleEvaluationDetails>,
    pub evaluated_splits: Vec<SplitEvaluationDetails>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AllocationEvaluationCode {
    /// The allocation was not evaluated because previous allocation matched.
    Unevaluated,
    /// The subject matched all conditions and this allocation was selected.
    Match,
    /// Evaluation happened before required start time for this allocation.
    BeforeStartTime,
    /// Evaluation happened after required end time for this allocation.
    AfterEndTime,
    /// Subject failed all allocation rules.
    FailingRule,
    /// Subject matched all rules but missed due to traffic exposure.
    TrafficExposureMiss,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleEvaluationDetails {
    pub matched: bool,
    pub conditions: Vec<ConditionEvaluationDetails>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionEvaluationDetails {
    pub(crate) condition: ConditionWire,
    pub attribute_value: Option<AttributeValue>,
    pub matched: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitEvaluationDetails {
    pub variation_key: Str,
    pub matched: bool,
    pub shards: Vec<ShardEvaluationDetails>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardEvaluationDetails {
    pub matched: bool,
    pub shard: Shard,
    pub shard_value: u32,
}

impl From<Result<(), EvaluationFailure>> for FlagEvaluationCode {
    fn from(value: Result<(), EvaluationFailure>) -> Self {
        match value {
            Ok(()) => Self::Match,
            Err(err) => err.into(),
        }
    }
}

impl From<EvaluationFailure> for FlagEvaluationCode {
    fn from(value: EvaluationFailure) -> Self {
        match value {
            EvaluationFailure::ConfigurationMissing => Self::ConfigurationMissing,
            EvaluationFailure::FlagUnrecognizedOrDisabled => Self::FlagUnrecognizedOrDisabled,
            EvaluationFailure::FlagDisabled => Self::FlagUnrecognizedOrDisabled,
            EvaluationFailure::DefaultAllocationNull => Self::DefaultAllocationNull,
            EvaluationFailure::Error(err) => err.into(),
            EvaluationFailure::NonBanditVariation
            | EvaluationFailure::NoActionsSuppliedForBandit => {
                debug_assert!(
                    false,
                    "{value:?} should never be emitted by flag evaluation"
                );
                Self::UnexpectedConfigurationError
            }
        }
    }
}

impl From<EvaluationError> for FlagEvaluationCode {
    fn from(value: EvaluationError) -> Self {
        match value {
            EvaluationError::TypeMismatch { .. } => Self::TypeMismatch,
            EvaluationError::UnexpectedConfigurationParseError => {
                Self::UnexpectedConfigurationError
            }
            EvaluationError::UnexpectedConfigurationError => Self::UnexpectedConfigurationError,
        }
    }
}

impl From<Result<(), EvaluationFailure>> for BanditEvaluationCode {
    fn from(value: Result<(), EvaluationFailure>) -> Self {
        match value {
            Ok(()) => Self::Match,
            Err(err) => err.into(),
        }
    }
}

impl From<EvaluationFailure> for BanditEvaluationCode {
    fn from(value: EvaluationFailure) -> Self {
        match value {
            EvaluationFailure::Error(err) => err.into(),
            EvaluationFailure::ConfigurationMissing => Self::ConfigurationMissing,
            EvaluationFailure::FlagUnrecognizedOrDisabled
            | EvaluationFailure::FlagDisabled
            | EvaluationFailure::DefaultAllocationNull => {
                debug_assert!(
                    false,
                    "{value:?} should never be emitted by bandit evaluation"
                );
                Self::UnexpectedConfigurationError
            }
            EvaluationFailure::NonBanditVariation => Self::NonBanditVariation,
            EvaluationFailure::NoActionsSuppliedForBandit => Self::NoActionsSuppliedForBandit,
        }
    }
}

impl From<EvaluationError> for BanditEvaluationCode {
    fn from(value: EvaluationError) -> Self {
        match value {
            EvaluationError::TypeMismatch { .. } => {
                debug_assert!(
                    false,
                    "{value:?} should never be emitted by bandit evaluation"
                );
                Self::UnexpectedConfigurationError
            }
            EvaluationError::UnexpectedConfigurationError => Self::UnexpectedConfigurationError,
            EvaluationError::UnexpectedConfigurationParseError => {
                Self::UnexpectedConfigurationError
            }
        }
    }
}

#[cfg(feature = "pyo3")]
mod pyo3_impl {
    use pyo3::prelude::*;

    use crate::pyo3::TryToPyObject;

    use super::EvaluationDetails;

    impl TryToPyObject for EvaluationDetails {
        fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject> {
            serde_pyobject::to_pyobject(py, self)
                .map(|it| it.unbind())
                .map_err(|err| err.0)
        }
    }
}

#[cfg(feature = "magnus")]
mod magnus_impl {
    use magnus::{prelude::*, IntoValue};

    use super::{EvaluationDetails, EvaluationResultWithDetails};

    impl IntoValue for &EvaluationDetails {
        fn into_value_with(self, _handle: &magnus::Ruby) -> magnus::Value {
            serde_magnus::serialize(self)
                .expect("EvaluationDetails should always be serializable to Ruby")
        }
    }

    impl<T: IntoValue> IntoValue for EvaluationResultWithDetails<T> {
        fn into_value_with(self, handle: &magnus::Ruby) -> magnus::Value {
            let hash = handle.hash_new();
            let _ = hash.aset(handle.sym_new("variation"), self.variation);
            let _ = hash.aset(handle.sym_new("action"), self.action);
            let _ = hash.aset(
                handle.sym_new("evaluation_details"),
                self.evaluation_details.as_ref(),
            );
            hash.as_value()
        }
    }
}
