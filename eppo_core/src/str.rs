//! Some string type helpers.
//!
//! Moved into a separate module, so we could experiment with different representations.

use std::{borrow::Cow, sync::Arc};

use faststr::FastStr;

use serde::{Deserialize, Serialize};

/// `ArcStr` is a string that can be cloned cheaply.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ArcStr(FastStr);

impl ArcStr {
    pub fn new<S: AsRef<str>>(s: S) -> ArcStr {
        ArcStr(FastStr::new(s))
    }

    pub fn from_static_str(s: &'static str) -> ArcStr {
        ArcStr(FastStr::from_static_str(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for ArcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::fmt::Display for ArcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

macro_rules! impl_from_faststr {
    ($ty:ty) => {
        impl From<$ty> for ArcStr {
            fn from(value: $ty) -> ArcStr {
                ArcStr(value.into())
            }
        }
    };
}

impl_from_faststr!(Arc<str>);
impl_from_faststr!(Arc<String>);
impl_from_faststr!(String);
impl_from_faststr!(FastStr);

impl<'a> From<&'a str> for ArcStr {
    fn from(value: &'a str) -> ArcStr {
        ArcStr(FastStr::new(value))
    }
}

impl<'a> From<Cow<'a, str>> for ArcStr {
    fn from(value: Cow<'a, str>) -> ArcStr {
        match value {
            Cow::Borrowed(s) => s.into(),
            Cow::Owned(s) => s.into(),
        }
    }
}

impl AsRef<str> for ArcStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for ArcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl log::kv::ToValue for ArcStr {
    fn to_value(&self) -> log::kv::Value {
        log::kv::Value::from_display(self)
    }
}

#[cfg(feature = "pyo3")]
mod pyo3_impl {
    use std::borrow::Cow;

    use pyo3::prelude::*;
    use pyo3::types::PyString;

    use crate::ArcStr;

    impl<'py> FromPyObject<'py> for ArcStr {
        fn extract_bound(value: &Bound<'py, PyAny>) -> PyResult<Self> {
            Ok(ArcStr::from(value.extract::<Cow<str>>()?))
        }
    }

    impl ToPyObject for ArcStr {
        fn to_object(&self, py: Python<'_>) -> PyObject {
            PyString::new_bound(py, &self).into()
        }
    }
}
