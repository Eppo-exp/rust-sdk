//! Some string type helpers.
//!
//! Moved into a separate module, so we could experiment with different representations.

use std::sync::Arc;

/// `ArcStr` is a string that can be cloned cheaply.
pub type ArcStr = Arc<str>;
