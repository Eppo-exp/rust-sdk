use std::collections::HashMap;

use derive_more::From;
use serde::{Deserialize, Serialize};

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
    /// A string value.
    String(String),
    /// A numerical value.
    Number(f64),
    /// A boolean value.
    Boolean(bool),
    /// A null value or absence of value.
    Null,
}
impl From<&str> for AttributeValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}
