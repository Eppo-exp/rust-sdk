use std::collections::HashMap;

use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::rules::Rule;

/// Universal Flag Configuration.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ufc {
    flags: HashMap<String, TryParse<Flag>>,
}

/// `TryParse` allows the subfield to fail parsing without failing the parsing of the whole
/// structure.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TryParse<T> {
    Parsed(T),
    ParseFailed(serde_json::Value),
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
    key: String,
    enabled: bool,
    variation_type: VariationType,
    variations: HashMap<String, Variation>,
    allocations: Vec<Allocation>,
    #[serde(default = "default_total_shards")]
    total_shards: u64,
}

fn default_total_shards() -> u64 {
    10_000
}

#[derive(Debug, Serialize, Deserialize)]
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

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variation {
    key: String,
    value: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Allocation {
    key: String,
    #[serde(default)]
    rules: Vec<Rule>,
    #[serde(default)]
    start_at: Option<Timestamp>,
    #[serde(default)]
    end_at: Option<Timestamp>,
    splits: Vec<Split>,
    #[serde(default = "default_do_log")]
    do_log: bool,
}

fn default_do_log() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Timestamp(String);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Split {
    shards: Vec<Shard>,
    variation_key: String,
    #[serde(default = "HashMap::new")]
    extra_logging: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shard {
    salt: String,
    ranges: Vec<Range>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    start: u64,
    end: u64,
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use crate::ufc::{Flag, TryParse};

    use super::Ufc;

    #[test]
    fn parse_flags_v1() {
        let f = File::open("tests/data/ufc/flags-v1.json")
            .expect("Failed to open tests/data/ufc/flags-v1.json");
        let _ufc: Ufc = serde_json::from_reader(BufReader::new(f)).unwrap();
    }

    #[test]
    fn parse_partially_if_unexpected() {
        let ufc: Ufc = serde_json::from_str(
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
