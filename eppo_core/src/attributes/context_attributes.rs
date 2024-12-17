use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::Str;

use super::{
    AttributeValue, AttributeValueImpl, Attributes, CategoricalAttribute, NumericAttribute,
};

/// `ContextAttributes` are subject or action attributes split by their semantics.
// TODO(oleksii): I think we should hide fields of this type and maybe the whole type itself. Now
// with `Attributes` being able to faithfully represent numeric and categorical attributes, there's
// little reason for users of eppo_core to know about `ContextAttributes`, so it makes sense to hide
// it and make it an internal type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "pyo3", pyo3::pyclass(module = "eppo_client"))]
pub struct ContextAttributes {
    /// Numeric attributes are quantitative (e.g., real numbers) and define a scale.
    ///
    /// Not all numbers are numeric attributes. If a number is used to represent an enumeration or
    /// on/off values, it is a categorical attribute.
    #[serde(alias = "numericAttributes")]
    pub numeric: Arc<HashMap<Str, NumericAttribute>>,
    /// Categorical attributes are attributes that have a finite set of values that are not directly
    /// comparable (i.e., enumeration).
    #[serde(alias = "categoricalAttributes")]
    pub categorical: Arc<HashMap<Str, CategoricalAttribute>>,
}

impl From<Attributes> for ContextAttributes {
    fn from(value: Attributes) -> Self {
        ContextAttributes::from_iter(value)
    }
}

impl<K, V> FromIterator<(K, V)> for ContextAttributes
where
    K: Into<Str>,
    V: Into<AttributeValue>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let (categorical, numeric) = iter.into_iter().fold(
            (HashMap::new(), HashMap::new()),
            |(mut categorical, mut numeric), (key, value)| {
                match value.into() {
                    AttributeValue(AttributeValueImpl::Categorical(value)) => {
                        categorical.insert(key.into(), value);
                    }
                    AttributeValue(AttributeValueImpl::Numeric(value)) => {
                        numeric.insert(key.into(), value);
                    }
                    AttributeValue(AttributeValueImpl::Null) => {
                        // Nulls are missing values and are ignored.
                    }
                }
                (categorical, numeric)
            },
        );
        ContextAttributes {
            numeric: Arc::new(numeric),
            categorical: Arc::new(categorical),
        }
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
    use std::{collections::HashMap, sync::Arc};

    use pyo3::prelude::*;

    use crate::{Attributes, CategoricalAttribute, NumericAttribute, Str};

    use super::ContextAttributes;

    #[pymethods]
    impl ContextAttributes {
        #[new]
        fn new(
            numeric_attributes: HashMap<Str, NumericAttribute>,
            categorical_attributes: HashMap<Str, CategoricalAttribute>,
        ) -> ContextAttributes {
            ContextAttributes {
                numeric: Arc::new(numeric_attributes),
                categorical: Arc::new(categorical_attributes),
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
