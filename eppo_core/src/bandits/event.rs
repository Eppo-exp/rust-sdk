use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Bandit evaluation event that needs to be logged to analytics storage.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BanditEvent {
    pub flag_key: String,
    pub bandit_key: String,
    pub subject: String,
    pub action: String,
    pub action_probability: f64,
    pub optimality_gap: f64,
    pub model_version: String,
    pub timestamp: String,
    pub subject_numeric_attributes: HashMap<String, f64>,
    pub subject_categorical_attributes: HashMap<String, String>,
    pub action_numeric_attributes: HashMap<String, f64>,
    pub action_categorical_attributes: HashMap<String, String>,
    pub meta_data: HashMap<String, String>,
}
