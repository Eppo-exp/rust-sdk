use std::sync::Arc;

use crate::ufc::UniversalFlagConfig;

#[derive(Default, Clone)]
pub struct Configuration {
    /// UFC configuration.
    pub ufc: Option<Arc<UniversalFlagConfig>>,
}
