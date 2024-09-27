//! Some string type helpers.
//!
//! Moved into a separate module, so we could experiment with different representations.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// `ArcStr` is a string that can be cloned cheaply.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ArcStr(Arc<str>);

impl std::fmt::Display for ArcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<T: Into<Arc<str>>> From<T> for ArcStr {
    fn from(value: T) -> ArcStr {
        ArcStr(value.into())
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
