use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{AttributeValue, Attributes};

/// `ContextAttributes` are subject or action attributes split by their semantics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextAttributes {
    /// Numeric attributes are quantitative (e.g., real numbers) and define a scale.
    ///
    /// Not all numbers are numeric attributes. If a number is used to represent an enumeration or
    /// on/off values, it is a categorical attribute.
    pub numeric: HashMap<String, f64>,
    /// Categorical attributes are attributes that have a finite set of values that are not directly
    /// comparable (i.e., enumeration).
    pub categorical: HashMap<String, String>,
}

impl From<Attributes> for ContextAttributes {
    fn from(value: Attributes) -> Self {
        ContextAttributes::from_iter(value)
    }
}

impl<K, V> FromIterator<(K, V)> for ContextAttributes
where
    K: ToOwned<Owned = String>,
    V: ToOwned<Owned = AttributeValue>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        iter.into_iter()
            .fold(ContextAttributes::default(), |mut acc, (key, value)| {
                match value.to_owned() {
                    AttributeValue::String(value) => {
                        acc.categorical.insert(key.to_owned(), value);
                    }
                    AttributeValue::Number(value) => {
                        acc.numeric.insert(key.to_owned(), value);
                    }
                    AttributeValue::Boolean(value) => {
                        // TBD: shall we ignore boolean attributes instead?
                        //
                        // One argument for including it here is that this basically guarantees that
                        // assignment evaluation inside bandit evaluation works the same way as if
                        // `get_assignment()` was called with generic `Attributes`.
                        //
                        // We can go a step further and remove `AttributeValue::Boolean` altogether,
                        // forcing it to be converted to a string before any evaluation.
                        acc.categorical.insert(key.to_owned(), value.to_string());
                    }
                    AttributeValue::Null => {
                        // Nulls are missing values and are ignored.
                    }
                }
                acc
            })
    }
}

impl ContextAttributes {
    /// Convert contextual attributes to generic `Attributes`.
    pub fn to_generic_attributes(&self) -> Attributes {
        let mut result = HashMap::with_capacity(self.numeric.len() + self.categorical.capacity());
        for (key, value) in self.numeric.iter() {
            result.insert(key.clone(), AttributeValue::Number(*value));
        }
        for (key, value) in self.categorical.iter() {
            result.insert(key.clone(), AttributeValue::String(value.clone()));
        }
        result
    }
}
