use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{AttributeValue, Attributes};

use super::{AssignmentValue, Condition, FlagEvaluationError, Shard, Value};

/// Details about feature flag evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalFlagDetails {
    pub flag_key: String,
    pub subject_key: String,
    pub subject_attributes: Attributes,
    /// Timestamp when the flag was evaluated.
    pub timestamp: DateTime<Utc>,
    /// Details of configuration used for evaluation. None if configuration hasn't been fetched yet.
    pub configuration_details: Option<ConfigurationDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<AssignmentValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<FlagEvaluationError>,
    /// Key of the selected variation.
    pub variation_key: Option<String>,
    /// Value of the selected variation. Could be `None` if no variation is selected, or selected
    /// value is absent in configuration (configuration error).
    pub variation_value: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allocations: Vec<EvalAllocationDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationDetails {
    /// Timestamp when configuration was fetched by the SDK.
    pub fetched_at: DateTime<Utc>,
    /// Timestamp when configuration was published by the server.
    pub published_at: DateTime<Utc>,
    /// Environment the configuration belongs to.
    pub environment_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalAllocationDetails {
    pub key: String,
    pub result: EvalAllocationResult,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evaluated_rules: Vec<EvalRuleDetails>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evaluated_splits: Vec<EvalSplitDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvalAllocationResult {
    /// The allocation was not evaluated because previous allocation matched.
    Unevaluated,
    /// The subject matched all conditions and this allocation was selected.
    Matched,
    /// Evaluation happened before required start date for this allocation.
    BeforeStartDate,
    /// Evaluation happened after required end date for this allocation.
    AfterEndDate,
    /// Subject failed all allocation rules.
    FailingRules,
    /// Subject matched all rules but missed due to traffic exposure.
    TrafficExposureMiss,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalRuleDetails {
    pub matched: bool,
    pub conditions: Vec<EvalConditionDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalConditionDetails {
    pub condition: Condition,
    pub attribute_value: Option<AttributeValue>,
    pub matched: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalSplitDetails {
    pub variation_key: String,
    pub matched: bool,
    pub shards: Vec<EvalShardDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalShardDetails {
    pub matched: bool,
    pub shard: Shard,
    pub shard_value: u64,
}
