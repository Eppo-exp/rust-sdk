use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub uuid: String,
    pub timestamp: i64,
    pub event_type: String,
    pub payload: HashMap<String, serde_json::Value>,
}
