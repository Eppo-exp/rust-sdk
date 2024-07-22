use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    error::{EvaluationError, EvaluationFailure},
    AttributeValue, Attributes,
};

use super::{Condition, Shard, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FlagEvaluationCode {
    /// An allocation configured for this flag was matched for any reason.
    Match,
    /// Flag does not exist or is not enabled for the environment in use.
    FlagUnrecognizedOrDisabled,
    /// Flag is not enabled for the environment in use.
    FlagDisabled,
    /// Default allocation is matched and is also serving NULL, resulting in the default value being
    /// assigned.
    DefaultAllocationNull,
    /// Variation value does not match the specified type for the function called.
    TypeMismatch,
    /// Configuration received from the server is invalid for the SDK. This should normally never
    /// happen and is likely a signal that you should update SDK.
    UnexpectedConfigurationError,
    /// `get_bandit_action` was called without supplying actions.
    NoActionsSuppliedForBandit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationResultWithDetails<T> {
    pub variation: Option<T>,
    pub action: Option<String>,
    pub evaluation_details: EvaluationDetails,
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

/// Details about feature flag evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationDetails {
    pub flag_key: String,
    pub subject_key: String,
    pub subject_attributes: Attributes,
    /// Timestamp when the flag was evaluated.
    pub timestamp: DateTime<Utc>,

    /// Timestamp when configuration was fetched by the SDK. None if configuration hasn't been
    /// fetched yet.
    pub config_fetched_at: Option<DateTime<Utc>>,
    /// Timestamp when configuration was published by the server. None if configuration hasn't been
    /// fetched yet.
    pub config_published_at: Option<DateTime<Utc>>,
    /// Environment the configuration belongs to. None if configuration hasn't been fetched yet.
    pub environment_name: Option<String>,

    pub flag_evaluation_code: FlagEvaluationCode,

    /// Key of the selected variation.
    pub variation_key: Option<String>,
    /// Value of the selected variation. Could be `None` if no variation is selected, or selected
    /// value is absent in configuration (configuration error).
    pub variation_value: Option<Value>,

    pub bandit_key: Option<String>,
    pub bandit_action: Option<String>,

    /// Evaluation details for all allocations.
    pub allocations: Vec<AllocationEvaluationDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationEvaluationDetails {
    pub key: String,
    /// Order position of the allocation as seen in the Web UI.
    pub order_position: usize,
    pub allocation_evaluation_code: AllocationEvaluationCode,
    pub evaluated_rules: Vec<RuleEvaluationDetails>,
    pub evaluated_splits: Vec<SplitEvaluationDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AllocationEvaluationCode {
    /// The allocation was not evaluated because previous allocation matched.
    Unevaluated,
    /// The subject matched all conditions and this allocation was selected.
    Match,
    /// Evaluation happened before required start date for this allocation.
    BeforeStartDate,
    /// Evaluation happened after required end date for this allocation.
    AfterEndDate,
    /// Subject failed all allocation rules.
    FailingRule,
    /// Subject matched all rules but missed due to traffic exposure.
    TrafficExposureMiss,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleEvaluationDetails {
    pub matched: bool,
    pub conditions: Vec<ConditionEvaluationDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionEvaluationDetails {
    pub condition: Condition,
    pub attribute_value: Option<AttributeValue>,
    pub matched: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitEvaluationDetails {
    pub variation_key: String,
    pub matched: bool,
    pub shards: Vec<ShardEvaluationDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardEvaluationDetails {
    pub matched: bool,
    pub shard: Shard,
    pub shard_value: u64,
}

impl From<Option<EvaluationFailure>> for FlagEvaluationCode {
    fn from(value: Option<EvaluationFailure>) -> Self {
        value.map(|it| it.into()).unwrap_or(Self::Match)
    }
}

impl From<EvaluationFailure> for FlagEvaluationCode {
    fn from(value: EvaluationFailure) -> Self {
        match value {
            EvaluationFailure::ConfigurationMissing => Self::FlagUnrecognizedOrDisabled,
            EvaluationFailure::FlagUnrecognizedOrDisabled => Self::FlagUnrecognizedOrDisabled,
            EvaluationFailure::FlagDisabled => Self::FlagUnrecognizedOrDisabled,
            EvaluationFailure::DefaultAllocationNull => Self::DefaultAllocationNull,
            EvaluationFailure::Error(err) => err.into(),
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
