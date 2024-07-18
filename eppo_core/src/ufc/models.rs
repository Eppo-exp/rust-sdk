use std::collections::HashMap;

use derive_more::From;
use serde::{Deserialize, Serialize};

use super::AssignmentValue;

#[allow(missing_docs)]
pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Universal Flag Configuration. This the response format from the UFC endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UniversalFlagConfig {
    /// When configuration was last updated.
    pub created_at: Timestamp,
    /// Environment this configuration belongs to.
    pub environment: Environment,
    /// Flags configuration.
    ///
    /// Value is wrapped in `TryParse` so that if we fail to parse one flag (e.g., new server
    /// format), we can still serve other flags.
    pub flags: HashMap<String, TryParse<Flag>>,
    /// `bandits` field connects string feature flags to bandits. Actual bandits configuration is
    /// served separately.
    #[serde(default)]
    pub bandits: HashMap<String, Vec<BanditVariation>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    /// Name of the environment.
    pub name: String,
}

/// `TryParse` allows the subfield to fail parsing without failing the parsing of the whole
/// structure.
///
/// This can be helpful to isolate errors in a subtree. e.g., if configuration for one flag parses,
/// the rest of the flags are still usable.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum TryParse<T> {
    /// Successfully parsed.
    Parsed(T),
    /// Parsing failed.
    ParseFailed(serde_json::Value),
}
impl<T> From<TryParse<T>> for Result<T, serde_json::Value> {
    fn from(value: TryParse<T>) -> Self {
        match value {
            TryParse::Parsed(v) => Ok(v),
            TryParse::ParseFailed(v) => Err(v),
        }
    }
}
impl<T> From<TryParse<T>> for Option<T> {
    fn from(value: TryParse<T>) -> Self {
        match value {
            TryParse::Parsed(v) => Some(v),
            TryParse::ParseFailed(_) => None,
        }
    }
}
impl<'a, T> From<&'a TryParse<T>> for Option<&'a T> {
    fn from(value: &TryParse<T>) -> Option<&T> {
        match value {
            TryParse::Parsed(v) => Some(v),
            TryParse::ParseFailed(_) => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Flag {
    pub key: String,
    pub enabled: bool,
    pub variation_type: VariationType,
    pub variations: HashMap<String, Variation>,
    pub allocations: Vec<Allocation>,
    pub total_shards: u64,
}

/// Type of the variation.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(missing_docs)]
pub enum VariationType {
    String,
    Integer,
    Numeric,
    Boolean,
    Json,
}

/// Subset of [`serde_json::Value`].
///
/// Unlike [`AssignmentValue`], `Value` is untagged, so we don't know the exact type until we
/// combine it with [`VariationType`] from the flag level.
#[derive(Debug, Serialize, Deserialize, PartialEq, From, Clone)]
#[serde(untagged)]
pub enum Value {
    /// Boolean maps to [`AssignmentValue::Boolean`].
    Boolean(bool),
    /// Number maps to either [`AssignmentValue::Integer`] or [`AssignmentValue::Numeric`].
    Number(f64),
    /// String maps to either [`AssignmentValue::String`] or [`AssignmentValue::Json`].
    String(String),
}

impl Value {
    /// Try to convert `Value` to [`AssignmentValue`] under the given [`VariationType`].
    pub(crate) fn to_assignment_value(&self, ty: VariationType) -> Option<AssignmentValue> {
        Some(match ty {
            VariationType::String => AssignmentValue::String(self.as_string()?.to_owned()),
            VariationType::Integer => AssignmentValue::Integer(self.as_integer()?),
            VariationType::Numeric => AssignmentValue::Numeric(self.as_number()?),
            VariationType::Boolean => AssignmentValue::Boolean(self.as_boolean()?),
            VariationType::Json => AssignmentValue::Json(self.to_json()?),
        })
    }

    fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            _ => None,
        }
    }

    fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    fn as_integer(&self) -> Option<i64> {
        let f = self.as_number()?;
        let i = f as i64;
        if i as f64 == f {
            Some(i)
        } else {
            None
        }
    }

    fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    fn to_json(&self) -> Option<serde_json::Value> {
        let s = self.as_string()?;
        serde_json::from_str(s).ok()?
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Variation {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Allocation {
    pub key: String,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub start_at: Option<Timestamp>,
    #[serde(default)]
    pub end_at: Option<Timestamp>,
    pub splits: Vec<Split>,
    #[serde(default = "default_do_log")]
    pub do_log: bool,
}

fn default_do_log() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, From, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Rule {
    pub conditions: Vec<Condition>,
}

/// `Condition` is a check that given user `attribute` matches the condition `value` under the given
/// `operator`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Condition {
    pub operator: ConditionOperator,
    pub attribute: String,
    pub value: ConditionValue,
}

/// Possible condition types.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConditionOperator {
    /// Matches regex. Condition value must be a regex string.
    Matches,
    /// Regex does not match. Condition value must be a regex string.
    NotMatches,
    /// Greater than or equal. Attribute and condition value must either be numbers or semver
    /// string.
    Gte,
    /// Greater than. Attribute and condition value must either be numbers or semver string.
    Gt,
    /// Less than or equal. Attribute and condition value must either be numbers or semver string.
    Lte,
    /// Less than. Attribute and condition value must either be numbers or semver string.
    Lt,
    /// One of values. Condition value must be a list of strings. Match is case-sensitive.
    OneOf,
    /// Not one of values. Condition value must be a list of strings. Match is case-sensitive.
    ///
    /// Null/absent attributes fail this condition automatically. (i.e., `null NOT_ONE_OF ["hello"]`
    /// is `false`)
    NotOneOf,
    /// Null check.
    ///
    /// Condition value must be a boolean. If it's `true`, this is a null check. If it's `false`,
    /// this is a not null check.
    IsNull,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(missing_docs)]
pub enum ConditionValue {
    Single(Value),
    // Only string arrays are currently supported.
    Multiple(Vec<String>),
}

impl<T: Into<Value>> From<T> for ConditionValue {
    fn from(value: T) -> Self {
        Self::Single(value.into())
    }
}
impl From<Vec<String>> for ConditionValue {
    fn from(value: Vec<String>) -> Self {
        Self::Multiple(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Split {
    pub shards: Vec<Shard>,
    pub variation_key: String,
    #[serde(default)]
    pub extra_logging: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Shard {
    pub salt: String,
    pub ranges: Vec<ShardRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct ShardRange {
    pub start: u64,
    pub end: u64,
}
impl ShardRange {
    pub(crate) fn contains(&self, v: u64) -> bool {
        self.start <= v && v < self.end
    }
}

/// `BanditVariation` associates a variation in feature flag with a bandit.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BanditVariation {
    pub key: String,
    /// Key of the flag.
    pub flag_key: String,
    /// Today it's the same as `variation_value`.
    pub variation_key: String,
    /// String variation value.
    pub variation_value: String,
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use super::{TryParse, UniversalFlagConfig};

    #[test]
    fn parse_flags_v1() {
        let f = File::open("../sdk-test-data/ufc/flags-v1.json")
            .expect("Failed to open ../sdk-test-data/ufc/flags-v1.json");
        let _ufc: UniversalFlagConfig = serde_json::from_reader(BufReader::new(f)).unwrap();
    }

    #[test]
    fn parse_partially_if_unexpected() {
        let ufc: UniversalFlagConfig = serde_json::from_str(
            &r#"
              {
                "createdAt": "2024-07-18T00:00:00Z",
                "environment": {"name": "test"},
                "flags": {
                  "success": {
                    "key": "success",
                    "enabled": true,
                    "variationType": "BOOLEAN",
                    "variations": {},
                    "allocations": [],
                    "totalShards": 10000
                  },
                  "fail_parsing": {
                    "key": "fail_parsing",
                    "enabled": true,
                    "variationType": "NEW_TYPE",
                    "variations": {},
                    "allocations": [],
                    "totalShards": 10000
                  }
                }
              }
            "#,
        )
        .unwrap();
        assert!(
            matches!(ufc.flags.get("success").unwrap(), TryParse::Parsed(_)),
            "{:?} should match TryParse::Parsed(_)",
            ufc.flags.get("success").unwrap()
        );
        assert!(
            matches!(
                ufc.flags.get("fail_parsing").unwrap(),
                TryParse::ParseFailed(_)
            ),
            "{:?} should match TryParse::ParseFailed(_)",
            ufc.flags.get("fail_parsing").unwrap()
        );
    }
}
