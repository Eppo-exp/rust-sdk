use serde::{Deserialize, Serialize};

use crate::{events::AssignmentEvent, ArcStr};

/// Result of assignment evaluation.
#[derive(Debug, Serialize, Clone)]
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
    String(ArcStr),
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
    /// let value = AssignmentValue::String("example".into());
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
    /// let value = AssignmentValue::String("example".into());
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
    /// let value = AssignmentValue::String("example".into());
    /// assert_eq!(value.to_string(), Some("example".into()));
    /// ```
    pub fn to_string(self) -> Option<ArcStr> {
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

#[cfg(feature = "pyo3")]
mod pyo3_impl {
    use pyo3::prelude::*;

    use crate::pyo3::TryToPyObject;

    use super::*;

    impl TryToPyObject for AssignmentValue {
        fn try_to_pyobject(&self, py: Python) -> PyResult<PyObject> {
            let obj = match self {
                AssignmentValue::String(s) => s.to_object(py),
                AssignmentValue::Integer(i) => i.to_object(py),
                AssignmentValue::Numeric(n) => n.to_object(py),
                AssignmentValue::Boolean(b) => b.to_object(py),
                AssignmentValue::Json(j) => match serde_pyobject::to_pyobject(py, j) {
                    Ok(it) => it.unbind(),
                    Err(err) => return Err(err.0),
                },
            };
            Ok(obj)
        }
    }
}
