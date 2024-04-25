use std::collections::HashMap;

use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::Config;

/// A client for Eppo API.
///
/// In order to create a client instance, first create [`Config`].
///
/// # Examples
/// ```
/// # use eppo::{EppoClient, Config};
/// EppoClient::new(Config::from_api_key("api-key"));
/// ```
pub struct EppoClient<'a> {
    config: Config<'a>,
}

impl<'a> EppoClient<'a> {
    /// Create a new `EppoClient` using the specified configuration.
    ///
    /// ```
    /// # use eppo::{Config, EppoClient};
    /// let client = EppoClient::new(Config::from_api_key("api-key"));
    /// ```
    pub fn new(config: Config<'a>) -> Self {
        EppoClient { config }
    }

    /// Get variation assignment for the given subject.
    pub fn get_assignment(
        &self,
        _flag_key: &str,
        _subject_key: &str,
        _subject_attributes: &SubjectAttributes,
    ) -> Option<AssignmentValue> {
        None
    }
}

pub type SubjectAttributes = HashMap<String, AttributeValue>;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, From)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
}
impl From<&str> for AttributeValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum AssignmentValue {
    String(String),
    Integer(i64),
    Numeric(f64),
    Boolean(bool),
    Json(serde_json::Value),
}

impl AssignmentValue {
    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            AssignmentValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_integer(&self) -> bool {
        self.as_integer().is_some()
    }
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            AssignmentValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn is_numeric(&self) -> bool {
        self.as_numeric().is_some()
    }
    pub fn as_numeric(&self) -> Option<f64> {
        match self {
            Self::Numeric(n) => Some(*n),
            _ => None,
        }
    }

    pub fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            AssignmentValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn is_json(&self) -> bool {
        self.as_json().is_some()
    }
    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Json(v) => Some(v),
            _ => None,
        }
    }
}
