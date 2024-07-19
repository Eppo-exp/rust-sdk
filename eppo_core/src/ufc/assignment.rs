use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Attributes;

use super::eval_details::EvalFlagDetails;

/// Result of assignment evaluation.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Assignment {
    /// Assignment value that should be returned to the user.
    pub value: AssignmentValue,
    /// Optional assignment event that should be logged to storage.
    pub event: Option<AssignmentEvent>,
}

/// Enum representing values assigned to a subject as a result of feature flag evaluation.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(tag = "type", content = "value", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssignmentValue {
    /// A string value.
    String(String),
    /// An integer value.
    Integer(i64),
    /// A numeric value (floating-point).
    Numeric(f64),
    /// A boolean value.
    Boolean(bool),
    /// Arbitrary JSON value.
    Json(serde_json::Value),
}

impl AssignmentValue {
    /// Checks if the assignment value is of type String.
    ///
    /// # Returns
    /// - `true` if the value is of type String, otherwise `false`.
    ///
    /// # Examples
    /// ```
    /// # use eppo_core::ufc::AssignmentValue;
    /// let value = AssignmentValue::String("example".to_owned());
    /// assert_eq!(value.is_string(), true);
    /// ```
    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }
    /// Returns the assignment value as a string if it is of type String.
    ///
    /// # Returns
    /// - The string value if the assignment value is of type String, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// # use eppo_core::ufc::AssignmentValue;
    /// let value = AssignmentValue::String("example".to_owned());
    /// assert_eq!(value.as_str(), Some("example"));
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        match self {
            AssignmentValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Extracts the assignment value as a string if it is of type String.
    ///
    /// # Returns
    /// - The string value if the assignment value is of type String, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// # use eppo_core::ufc::AssignmentValue;
    /// let value = AssignmentValue::String("example".to_owned());
    /// assert_eq!(value.to_string(), Some("example".to_owned()));
    /// ```
    pub fn to_string(self) -> Option<String> {
        match self {
            AssignmentValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Checks if the assignment value is of type Integer.
    ///
    /// # Returns
    /// - `true` if the value is of type Integer, otherwise `false`.
    ///
    /// # Examples
    /// ```
    /// # use eppo_core::ufc::AssignmentValue;
    /// let value = AssignmentValue::Integer(42);
    /// assert_eq!(value.is_integer(), true);
    /// ```
    pub fn is_integer(&self) -> bool {
        self.as_integer().is_some()
    }
    /// Returns the assignment value as an integer if it is of type Integer.
    ///
    /// # Returns
    /// - The integer value if the assignment value is of type Integer, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// # use eppo_core::ufc::AssignmentValue;
    /// let value = AssignmentValue::Integer(42);
    /// assert_eq!(value.as_integer(), Some(42));
    /// ```
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            AssignmentValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Checks if the assignment value is of type Numeric.
    ///
    /// # Returns
    /// - `true` if the value is of type Numeric, otherwise `false`.
    ///
    /// # Examples
    /// ```
    /// # use eppo_core::ufc::AssignmentValue;
    /// let value = AssignmentValue::Numeric(3.14);
    /// assert_eq!(value.is_numeric(), true);
    /// ```
    pub fn is_numeric(&self) -> bool {
        self.as_numeric().is_some()
    }
    /// Returns the assignment value as a numeric value if it is of type Numeric.
    ///
    /// # Returns
    /// - The numeric value if the assignment value is of type Numeric, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// # use eppo_core::ufc::AssignmentValue;
    /// let value = AssignmentValue::Numeric(3.14);
    /// assert_eq!(value.as_numeric(), Some(3.14));
    /// ```
    pub fn as_numeric(&self) -> Option<f64> {
        match self {
            Self::Numeric(n) => Some(*n),
            _ => None,
        }
    }

    /// Checks if the assignment value is of type Boolean.
    ///
    /// # Returns
    /// - `true` if the value is of type Boolean, otherwise `false`.
    ///
    /// # Examples
    /// ```
    /// # use eppo_core::ufc::AssignmentValue;
    /// let value = AssignmentValue::Boolean(true);
    /// assert_eq!(value.is_boolean(), true);
    /// ```
    pub fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }
    /// Returns the assignment value as a boolean if it is of type Boolean.
    ///
    /// # Returns
    /// - The boolean value if the assignment value is of type Boolean, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// # use eppo_core::ufc::AssignmentValue;
    /// let value = AssignmentValue::Boolean(true);
    /// assert_eq!(value.as_boolean(), Some(true));
    /// ```
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            AssignmentValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Checks if the assignment value is of type Json.
    ///
    /// # Returns
    /// - `true` if the value is of type Json, otherwise `false`.
    ///
    /// # Examples
    /// ```
    /// # use eppo_core::ufc::AssignmentValue;
    /// use serde_json::json;
    ///
    /// let value = AssignmentValue::Json(json!({ "key": "value" }));
    /// assert_eq!(value.is_json(), true);
    /// ```
    pub fn is_json(&self) -> bool {
        self.as_json().is_some()
    }
    /// Returns the assignment value as a JSON value if it is of type Json.
    ///
    /// # Returns
    /// - The JSON value if the assignment value is of type Json, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// # use eppo_core::ufc::AssignmentValue;
    /// use serde_json::json;
    ///
    /// let value = AssignmentValue::Json(json!({ "key": "value" }));
    /// assert_eq!(value.as_json(), Some(&json!({ "key": "value" })));
    /// ```
    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Json(v) => Some(v),
            _ => None,
        }
    }
    /// Extracts the assignment value as a JSON value if it is of type Json.
    ///
    /// # Returns
    /// - The JSON value if the assignment value is of type Json, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// # use eppo_core::ufc::AssignmentValue;
    /// use serde_json::json;
    ///
    /// let value = AssignmentValue::Json(json!({ "key": "value" }));
    /// assert_eq!(value.to_json(), Some(json!({ "key": "value" })));
    /// ```
    pub fn to_json(self) -> Option<serde_json::Value> {
        match self {
            Self::Json(v) => Some(v),
            _ => None,
        }
    }
}

/// Represents an event capturing the assignment of a feature flag to a subject and its logging
/// details.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentEvent {
    /// The key of the feature flag being assigned.
    pub feature_flag: String,
    /// The key of the allocation that the subject was assigned to.
    pub allocation: String,
    /// The key of the experiment associated with the assignment.
    pub experiment: String,
    /// The specific variation assigned to the subject.
    pub variation: String,
    /// The key identifying the subject receiving the assignment.
    pub subject: String,
    /// Custom attributes of the subject relevant to the assignment.
    pub subject_attributes: Attributes,
    /// The timestamp indicating when the assignment event occurred.
    pub timestamp: String,
    /// Additional metadata such as SDK language and version.
    pub meta_data: HashMap<String, String>,
    /// Additional user-defined logging fields for capturing extra information related to the
    /// assignment.
    #[serde(flatten)]
    pub extra_logging: HashMap<String, String>,
    /// Evaluation details that could help with debugging the assigment. Only populated when
    /// details-version of the `get_assigment` was called.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluation_details: Option<EvalFlagDetails>,
}
