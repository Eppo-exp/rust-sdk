use eppo_core::ufc::{AssignmentValue, UniversalFlagConfig};
use eppo_core::SdkMetadata;
use fastly::http::StatusCode;
use fastly::kv_store::KVStoreError;
use fastly::{Error, KVStore, Request, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct RequestBody {
    subject_key: String,
    subject_attributes: HashMap<String, serde_json::Value>,
    #[serde(rename = "banditActions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    bandit_actions: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize)]
struct AssignmentsResponse {
    assignments: HashMap<String, AssignmentValue>,
    timestamp: i64,
}

const KV_STORE_NAME: &str = "edge-assignment-kv-store";
const SDK_KEY_QUERY_PARAM: &str = "sdk_key";

fn kv_store_key(api_key: &str) -> String {
    format!("ufc-by-sdk-key-{}", api_key)
}

pub fn handle_assignments(mut req: Request) -> Result<Response, Error> {
    // Extract the API key first before we consume the request
    let api_key = match req.get_query_parameter(SDK_KEY_QUERY_PARAM) {
        Some(key) if !key.is_empty() => key.to_string(), // Convert to owned String
        _ => {
            return Ok(Response::from_status(StatusCode::BAD_REQUEST)
                .with_body_text_plain("Missing required query parameter: sdk_key"));
        }
    };

    // Now we can consume the request body
    let body: RequestBody = match serde_json::from_slice::<RequestBody>(&req.take_body_bytes()) {
        Ok(body) => {
            if body.subject_key.is_empty() {
                return Ok(Response::from_status(StatusCode::BAD_REQUEST)
                    .with_body_text_plain("subject_key is required and cannot be empty"));
            }
            body
        }
        Err(e) => {
            let error_message = if e.to_string().contains("subject_key") {
                "subject_key is required in the request body"
            } else {
                "Invalid request body format"
            };
            return Ok(
                Response::from_status(StatusCode::BAD_REQUEST).with_body_text_plain(error_message)
            );
        }
    };

    let subject_key = body.subject_key;
    let subject_attributes = body.subject_attributes;
    let bandit_actions = body.bandit_actions;

    // Construct an KVStore instance which is connected to the KV Store named `my-store`
    // [Documentation for the KVStore open method can be found here](https://docs.rs/fastly/latest/fastly/struct.KVStore.html#method.open)
    let kv_store = KVStore::open(KV_STORE_NAME).map(|store| store.expect("KVStore exists"))?;

    let mut kv_store_item = match kv_store.lookup(&kv_store_key(&api_key)) {
        Ok(item) => item,
        Err(e) => {
            let (status, message) = match e {
                // Return unauthorized if the key does not exist.
                // Our protocol lets the client know that the SDK key has not had a UFC
                // configuration pre-computed for it in the KV Store.
                KVStoreError::ItemNotFound => (
                    StatusCode::UNAUTHORIZED,
                    "SDK key not found in KV store".to_string(),
                ),
                _ => {
                    //fastly::log::error!("KV Store error: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Unexpected KV Store error".to_string(),
                    )
                }
            };

            return Ok(Response::from_status(status).with_body_text_plain(&message));
        }
    };

    // Parse the response from the KV store
    let kv_store_item_body = kv_store_item.take_body();
    let ufc_config = match UniversalFlagConfig::from_json(
        SdkMetadata {
            name: "fastly-edge-assignments",
            version: "0.1.0",
        },
        kv_store_item_body.into_bytes(),
    ) {
        Ok(config) => config,
        Err(e) => {
            //fastly::log::error!("Failed to parse UFC config: {:?}", e);
            return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                .with_body_text_plain("Invalid configuration format in KV store"));
        }
    };

    let assignments_response = AssignmentsResponse {
        // todo: push this to eppo_core
        assignments: HashMap::new(),
        // assignments: ufc_config
        //     .compiled
        //     .flags
        //     .keys()
        //     .map(|flag_key| (flag_key.clone(), AssignmentValue::String("test".into())))
        //     .collect(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    // Create an HTTP OK response with the assignments
    let response = match Response::from_status(StatusCode::OK).with_body_json(&assignments_response)
    {
        Ok(response) => response,
        Err(e) => {
            // fastly::log::error!("Failed to serialize response: {:?}", e);
            return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                .with_body_text_plain("Failed to serialize response"));
        }
    };
    Ok(response)
}
