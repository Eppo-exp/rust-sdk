use std::sync::Arc;

use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::{events::AssignmentEvent, Str};

use crate::ufc::VariationType;

use super::ValueWire;

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
///
/// # Serialization
///
/// When serialized to JSON, serialized as a two-field object with `type` and `value`. Type is one
/// of "STRING", "INTEGER", "NUMERIC", "BOOLEAN", or "JSON". Value is either string, number,
/// boolean, or arbitrary JSON value.
///
/// Example:
/// ```json
/// {"type":"JSON","value":{"hello":"world"}}
/// ```
#[derive(Debug, Clone)]
pub enum AssignmentValue {
    /// A string value.
    String(Str),
    /// An integer value.
    Integer(i64),
    /// A numeric value (floating-point).
    Numeric(f64),
    /// A boolean value.
    Boolean(bool),
    /// Arbitrary JSON value.
    Json {
        raw: Str,
        parsed: Arc<serde_json::Value>,
    },
}

impl Serialize for AssignmentValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AssignmentValue", 2)?;
        match self {
            AssignmentValue::String(s) => {
                state.serialize_field("type", "STRING")?;
                state.serialize_field("value", s)?;
            }
            AssignmentValue::Integer(i) => {
                state.serialize_field("type", "INTEGER")?;
                state.serialize_field("value", i)?;
            }
            AssignmentValue::Numeric(n) => {
                state.serialize_field("type", "NUMERIC")?;
                state.serialize_field("value", n)?;
            }
            AssignmentValue::Boolean(b) => {
                state.serialize_field("type", "BOOLEAN")?;
                state.serialize_field("value", b)?;
            }
            AssignmentValue::Json { raw: _, parsed } => {
                state.serialize_field("type", "JSON")?;
                state.serialize_field("value", parsed)?;
            }
        }
        state.end()
    }
}

impl PartialEq for AssignmentValue {
    // Compare ignoring Json::raw.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AssignmentValue::String(v1), AssignmentValue::String(v2)) => v1 == v2,
            (AssignmentValue::Integer(v1), AssignmentValue::Integer(v2)) => v1 == v2,
            (AssignmentValue::Numeric(v1), AssignmentValue::Numeric(v2)) => v1 == v2,
            (AssignmentValue::Boolean(v1), AssignmentValue::Boolean(v2)) => v1 == v2,
            (
                AssignmentValue::Json { parsed: v1, .. },
                AssignmentValue::Json { parsed: v2, .. },
            ) => v1 == v2,
            _ => false,
        }
    }
}

impl AssignmentValue {
    pub fn from_json(value: serde_json::Value) -> Result<AssignmentValue, serde_json::Error> {
        let raw = serde_json::to_string(&value)?;
        Ok(AssignmentValue::Json {
            raw: raw.into(),
            parsed: Arc::new(value),
        })
    }

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
    pub fn to_string(self) -> Option<Str> {
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
    /// let value = AssignmentValue::from_json(json!({ "key": "value" }).into()).unwrap();
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
    /// let value = AssignmentValue::from_json(json!({ "key": "value" }).into()).unwrap();
    /// assert_eq!(value.as_json(), Some(&json!({ "key": "value" })));
    /// ```
    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Json { raw: _, parsed } => Some(parsed),
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
    /// let value = AssignmentValue::from_json(json!({ "key": "value" }).into()).unwrap();
    /// assert_eq!(value.to_json(), Some(json!({ "key": "value" }).into()));
    /// ```
    pub fn to_json(self) -> Option<Arc<serde_json::Value>> {
        match self {
            Self::Json { raw: _, parsed } => Some(parsed),
            _ => None,
        }
    }

    /// Returns the type of the variation as a string.
    ///
    /// # Returns
    /// - A string representing the type of the variation ("STRING", "INTEGER", "NUMERIC", "BOOLEAN", or "JSON").
    ///
    /// # Examples
    /// ```
    /// # use eppo_core::ufc::AssignmentValue;
    /// # use eppo_core::ufc::VariationType;
    /// let value = AssignmentValue::String("example".into());
    /// assert_eq!(value.variation_type(), VariationType::String);
    /// ```
    pub fn variation_type(&self) -> VariationType {
        match self {
            AssignmentValue::String(_) => VariationType::String,
            AssignmentValue::Integer(_) => VariationType::Integer,
            AssignmentValue::Numeric(_) => VariationType::Numeric,
            AssignmentValue::Boolean(_) => VariationType::Boolean,
            AssignmentValue::Json { .. } => VariationType::Json,
        }
    }

    /// Returns the raw value of the variation.
    ///
    /// # Returns
    /// - A JSON Value containing the variation value.
    pub(crate) fn variation_value(&self) -> ValueWire {
        match self {
            AssignmentValue::String(s) => ValueWire::String(s.clone()),
            AssignmentValue::Integer(i) => ValueWire::Number(*i as f64),
            AssignmentValue::Numeric(n) => ValueWire::Number(*n),
            AssignmentValue::Boolean(b) => ValueWire::Boolean(*b),
            AssignmentValue::Json { raw, parsed: _ } => ValueWire::String(raw.clone()),
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
                AssignmentValue::Json { raw: _, parsed } => {
                    match serde_pyobject::to_pyobject(py, parsed) {
                        Ok(it) => it.unbind(),
                        Err(err) => return Err(err.0),
                    }
                }
            };
            Ok(obj)
        }
    }
}

#[cfg(feature = "magnus")]
mod magnus_impl {
    use magnus::prelude::*;
    use magnus::{IntoValue, Ruby};

    use super::*;

    impl IntoValue for AssignmentValue {
        fn into_value_with(self, handle: &Ruby) -> magnus::Value {
            match self {
                AssignmentValue::String(s) => s.into_value_with(handle),
                AssignmentValue::Integer(i) => i.into_value_with(handle),
                AssignmentValue::Numeric(n) => n.into_value_with(handle),
                AssignmentValue::Boolean(b) => b.into_value_with(handle),
                AssignmentValue::Json { raw: _, parsed } => serde_magnus::serialize(&parsed)
                    .expect("JSON value should always be serializable to Ruby"),
            }
        }
    }

    impl IntoValue for Assignment {
        fn into_value_with(self, handle: &Ruby) -> magnus::Value {
            let hash = handle.hash_new();
            let _ = hash.aset(handle.sym_new("value"), self.value);
            let _ = hash.aset(
                handle.sym_new("event"),
                serde_magnus::serialize::<_, magnus::Value>(&self.event)
                    .expect("AssignmentEvent should always be serializable to Ruby"),
            );
            hash.as_value()
        }
    }
}
