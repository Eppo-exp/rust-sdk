use crate::ufc::{Assignment, AssignmentFormat, Environment, VariationType};
use crate::{Attributes, Configuration, Str};
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
    pub allocation_key: Str,
    pub variation_key: Str,
    pub variation_type: VariationType,
    pub variation_value: serde_json::Value,
    /// Additional user-defined logging fields for capturing extra information related to the
    /// assignment.
    #[serde(flatten)]
    pub extra_logging: HashMap<String, String>,
    pub do_log: bool,
}

impl FlagAssignment {
    pub fn try_from_assignment(assignment: Assignment) -> Option<Self> {
        // WARNING! There is a problem here. The event is only populated for splits
        // that have `do_log` set to true in the wire format. This means that
        // all the ones present here are logged, but any splits that are not
        // logged are not present here.
        //
        // This is a problem for us because we want to be able to return
        // precomputed assignments for any split, logged or not, since we
        // want to be able to return them for all flags.
        //
        // We need to fix this.
        assignment.event.as_ref().map(|event| Self {
            allocation_key: event.base.allocation.clone(),
            variation_key: event.base.variation.clone(),
            variation_type: assignment.value.variation_type(),
            variation_value: assignment.value.variation_value(),
            extra_logging: event
                .base
                .extra_logging
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            do_log: true,
        })
    }
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
    pub fn from_configuration(
        configuration: Arc<Configuration>,
        flags: HashMap<String, FlagAssignment>,
    ) -> Self {
        Self {
            created_at: chrono::Utc::now(),
            format: AssignmentFormat::Precomputed,
            environment: {
                Environment {
                    name: configuration.flags.compiled.environment.name.clone(),
                }
            },
            flags,
        }
    }
}
