use crate::ufc::{AssignmentFormat, Environment, VariationType};
use crate::{Attributes, Str};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// Request
#[derive(Debug, Deserialize)]
pub struct PrecomputedAssignmentsServiceRequestBody {
    pub subject_key: String,
    pub subject_attributes: Arc<Attributes>,
    // TODO: Add bandit actions
    // #[serde(rename = "banditActions")]
    // #[serde(skip_serializing_if = "Option::is_none")]
    // bandit_actions: Option<HashMap<String, serde_json::Value>>,
}

// Response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlagAssignment {
    pub allocation_key: String,
    pub variation_key: String,
    pub variation_type: VariationType,
    pub variation_value: serde_json::Value,
    /// Additional user-defined logging fields for capturing extra information related to the
    /// assignment.
    #[serde(flatten)]
    pub extra_logging: HashMap<String, String>,
    pub do_log: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrecomputedAssignmentsServiceResponse {
    created_at: chrono::DateTime<chrono::Utc>,
    format: AssignmentFormat,
    environment: Environment,
    flags: HashMap<String, FlagAssignment>,
}

impl PrecomputedAssignmentsServiceResponse {
    pub fn new(environment_name: Str, flags: HashMap<String, FlagAssignment>) -> Self {
        Self {
            created_at: chrono::Utc::now(),
            format: AssignmentFormat::Precomputed,
            environment: {
                Environment {
                    name: environment_name,
                }
            },
            flags,
        }
    }
}
