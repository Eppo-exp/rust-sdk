use std::{collections::HashMap, sync::Arc};

use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::ArcStr;

/// `Subject` is a bundle of subject attributes and a key.
#[derive(Debug)]
pub(crate) struct Subject {
    /// Subject key encoded as attribute value. Known to be `AttributeValue::ArcString`. This is
    /// done to allow returning subject key as an attribute when rule references "id".
    key: AttributeValue,
    attributes: Arc<Attributes>,
}

impl Subject {
    pub fn new(key: ArcStr, attributes: Arc<Attributes>) -> Subject {
        Subject {
            key: AttributeValue::ArcString(key),
            attributes,
        }
    }

    pub fn key(&self) -> &ArcStr {
        let AttributeValue::ArcString(s) = &self.key else {
            unreachable!("Subject::key is always encoded as AttributeValue::ArcString()");
        };
        s
    }

    pub fn get_attribute(&self, name: &str) -> Option<&AttributeValue> {
        let value = self.attributes.get(name);
        if value.is_some() {
            return value;
        }

        if name == "id" {
            return Some(&self.key);
        }

        None
    }
}

/// Type alias for a HashMap representing key-value pairs of attributes.
///
/// Keys are strings representing attribute names.
///
/// # Examples
/// ```
/// # use eppo_core::{Attributes, AttributeValue};
/// let attributes = [
///     ("age".to_owned(), 30.0.into()),
///     ("is_premium_member".to_owned(), true.into()),
///     ("username".to_owned(), "john_doe".into()),
/// ].into_iter().collect::<Attributes>();
/// ```
pub type Attributes = HashMap<String, AttributeValue>;

/// Enum representing possible values of an attribute for a subject.
///
/// Conveniently implements `From` conversions for `String`, `&str`, `f64`, and `bool` types.
///
/// Examples:
/// ```
/// # use eppo_core::AttributeValue;
/// let string_attr: AttributeValue = "example".into();
/// let number_attr: AttributeValue = 42.0.into();
/// let bool_attr: AttributeValue = true.into();
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, From, Clone)]
#[serde(untagged)]
pub enum AttributeValue {
    ArcString(ArcStr),
    /// A string value.
    String(String),
    /// A numerical value.
    Number(f64),
    /// A boolean value.
    Boolean(bool),
    /// A null value or absence of value.
    Null,
}

impl AttributeValue {
    pub fn as_str(&self) -> Option<&str> {
        if let AttributeValue::String(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }
}

impl From<&str> for AttributeValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

#[cfg(feature = "pyo3")]
mod pyo3_impl {
    use pyo3::{exceptions::PyTypeError, prelude::*, types::*};

    use super::*;

    impl<'py> FromPyObject<'py> for AttributeValue {
        fn extract_bound(value: &Bound<'py, PyAny>) -> PyResult<AttributeValue> {
            if let Ok(s) = value.downcast::<PyString>() {
                return Ok(AttributeValue::String(s.extract()?));
            }
            // In Python, Bool inherits from Int, so it must be checked first here.
            if let Ok(s) = value.downcast::<PyBool>() {
                return Ok(AttributeValue::Boolean(s.extract()?));
            }
            if let Ok(s) = value.downcast::<PyFloat>() {
                return Ok(AttributeValue::Number(s.extract()?));
            }
            if let Ok(s) = value.downcast::<PyInt>() {
                return Ok(AttributeValue::Number(s.extract()?));
            }
            if let Ok(_) = value.downcast::<PyNone>() {
                return Ok(AttributeValue::Null);
            }
            Err(PyTypeError::new_err(
                "invalid type for subject attribute value",
            ))
        }
    }
}
