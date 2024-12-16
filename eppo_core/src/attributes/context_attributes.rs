use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    AttributeValue, AttributeValueImpl, Attributes, CategoricalAttribute, NumericAttribute,
};

/// `ContextAttributes` are subject or action attributes split by their semantics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "pyo3", pyo3::pyclass(module = "eppo_client"))]
pub struct ContextAttributes {
    /// Numeric attributes are quantitative (e.g., real numbers) and define a scale.
    ///
    /// Not all numbers are numeric attributes. If a number is used to represent an enumeration or
    /// on/off values, it is a categorical attribute.
    #[serde(alias = "numericAttributes")]
    pub numeric: HashMap<String, NumericAttribute>,
    /// Categorical attributes are attributes that have a finite set of values that are not directly
    /// comparable (i.e., enumeration).
    #[serde(alias = "categoricalAttributes")]
    pub categorical: HashMap<String, CategoricalAttribute>,
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
                    AttributeValue(AttributeValueImpl::Categorical(value)) => {
                        acc.categorical.insert(key.to_owned(), value);
                    }
                    AttributeValue(AttributeValueImpl::Numeric(value)) => {
                        acc.numeric.insert(key.to_owned(), value);
                    }
                    AttributeValue(AttributeValueImpl::Null) => {
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
            result.insert(key.clone(), value.clone().into());
        }
        for (key, value) in self.categorical.iter() {
            result.insert(key.clone(), value.clone().into());
        }
        result
    }
}

#[cfg(feature = "pyo3")]
mod pyo3_impl {
    use std::collections::HashMap;

    use pyo3::prelude::*;

    use crate::{Attributes, CategoricalAttribute, NumericAttribute};

    use super::ContextAttributes;

    #[pymethods]
    impl ContextAttributes {
        #[new]
        fn new(
            numeric_attributes: HashMap<String, NumericAttribute>,
            categorical_attributes: HashMap<String, CategoricalAttribute>,
        ) -> ContextAttributes {
            ContextAttributes {
                numeric: numeric_attributes,
                categorical: categorical_attributes,
            }
        }

        /// Create an empty Attributes instance with no numeric or categorical attributes.
        ///
        /// Returns:
        ///     ContextAttributes: An instance of the ContextAttributes class with empty dictionaries
        ///         for numeric and categorical attributes.
        #[staticmethod]
        fn empty() -> ContextAttributes {
            ContextAttributes::default()
        }

        /// Create an ContextAttributes instance from a dictionary of attributes.

        /// Args:
        ///     attributes (Dict[str, Union[float, int, bool, str]]): A dictionary where keys are attribute names
        ///         and values are attribute values which can be of type float, int, bool, or str.

        /// Returns:
        ///     ContextAttributes: An instance of the ContextAttributes class
        ///         with numeric and categorical attributes separated.
        #[staticmethod]
        fn from_dict(attributes: Attributes) -> ContextAttributes {
            attributes.into()
        }

        /// Note that this copies internal attributes, so changes to returned value won't have
        /// effect. This may be mitigated by setting numeric attributes again.
        #[getter]
        fn get_numeric_attributes(&self, py: Python) -> PyObject {
            self.numeric.to_object(py)
        }

        /// Note that this copies internal attributes, so changes to returned value won't have
        /// effect. This may be mitigated by setting categorical attributes again.
        #[getter]
        fn get_categorical_attributes(&self, py: Python) -> PyObject {
            self.categorical.to_object(py)
        }
    }
}
