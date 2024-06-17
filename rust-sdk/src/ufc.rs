use std::collections::HashMap;

use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::{client::AssignmentValue, rules::Rule};

/// Universal Flag Configuration.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniversalFlagConfig {
    // Value is wrapped in `TryParse` so that if we fail to parse one flag (e.g., new server
    // format), we can still serve other flags.
    pub flags: HashMap<String, TryParse<Flag>>,
}

/// `TryParse` allows the subfield to fail parsing without failing the parsing of the whole
/// structure.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TryParse<T> {
    Parsed(T),
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Flag {
    pub key: String,
    pub enabled: bool,
    pub variation_type: VariationType,
    pub variations: HashMap<String, Variation>,
    pub allocations: Vec<Allocation>,
    #[serde(default = "default_total_shards")]
    pub total_shards: u64,
}

fn default_total_shards() -> u64 {
    10_000
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
/// combine it with [`VariationType`].
#[derive(Debug, Serialize, Deserialize, PartialEq, From)]
#[serde(untagged)]
pub enum Value {
    Boolean(bool),
    /// Number maps to either [`AssignmentValue::Integer`] or [`AssignmentValue::Numeric`].
    Number(f64),
    /// String maps to either [`AssignmentValue::String`] or [`AssignmentValue::Json`].
    String(String),
}

impl Value {
    pub fn to_assignment_value(&self, ty: VariationType) -> Option<AssignmentValue> {
        Some(match ty {
            VariationType::String => AssignmentValue::String(self.as_string()?.to_owned()),
            VariationType::Integer => AssignmentValue::Integer(self.as_integer()?),
            VariationType::Numeric => AssignmentValue::Numeric(self.as_number()?),
            VariationType::Boolean => AssignmentValue::Boolean(self.as_boolean()?),
            VariationType::Json => AssignmentValue::Json(self.as_json()?),
        })
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
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

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_json(&self) -> Option<serde_json::Value> {
        let s = self.as_string()?;
        serde_json::from_str(s).ok()?
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variation {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

pub type Timestamp = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Split {
    pub shards: Vec<Shard>,
    pub variation_key: String,
    #[serde(default = "HashMap::new")]
    pub extra_logging: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shard {
    pub salt: String,
    pub ranges: Vec<Range>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub start: u64,
    pub end: u64,
}
impl Range {
    pub fn contains(&self, v: u64) -> bool {
        self.start <= v && v < self.end
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use super::{TryParse, UniversalFlagConfig};

    #[test]
    fn parse_flags_v1() {
        let f = File::open("tests/data/ufc/flags-v1.json")
            .expect("Failed to open tests/data/ufc/flags-v1.json");
        let _ufc: UniversalFlagConfig = serde_json::from_reader(BufReader::new(f)).unwrap();
    }

    #[test]
    fn parse_partially_if_unexpected() {
        let ufc: UniversalFlagConfig = serde_json::from_str(
            &r#"
              {
                "flags": {
                  "success": {
                    "key": "success",
                    "enabled": true,
                    "variationType": "BOOLEAN",
                    "variations": {},
                    "allocations": []
                  },
                  "fail_parsing": {
                    "key": "fail_parsing",
                    "enabled": true,
                    "variationType": "NEW_TYPE",
                    "variations": {},
                    "allocations": []
                  }
                }
              }
            "#,
        )
        .unwrap();
        assert!(matches!(
            ufc.flags.get("success").unwrap(),
            TryParse::Parsed(_)
        ));
        assert!(matches!(
            ufc.flags.get("fail_parsing").unwrap(),
            TryParse::ParseFailed(_)
        ));
    }
}
