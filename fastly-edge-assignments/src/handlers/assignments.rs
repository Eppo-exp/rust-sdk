use eppo_core::configuration_store::ConfigurationStore;
use eppo_core::eval::{Evaluator, EvaluatorConfig};
use eppo_core::ufc::UniversalFlagConfig;
use eppo_core::{Attributes, Configuration, SdkMetadata};
use fastly::http::StatusCode;
use fastly::kv_store::KVStoreError;
use fastly::{Error, KVStore, Request, Response};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct RequestBody {
    subject_key: String,
    subject_attributes: Arc<Attributes>,
    // TODO: Add bandit actions
    // #[serde(rename = "banditActions")]
    // #[serde(skip_serializing_if = "Option::is_none")]
    // bandit_actions: Option<HashMap<String, serde_json::Value>>,
}

// Response

#[derive(Debug, Serialize)]
#[serde(rename_all = "UPPERCASE")]
enum AssignmentFormat {
    Precomputed,
}

#[derive(Debug, Serialize)]
struct Environment {
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FlagAssignment {
    allocation_key: String,
    variation_key: String,
    variation_type: String,
    variation_value: serde_json::Value,
    extra_logging: HashMap<String, serde_json::Value>,
    do_log: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AssignmentsResponse {
    created_at: i64,
    format: AssignmentFormat,
    environment: Environment,
    flags: HashMap<String, FlagAssignment>,
}

const KV_STORE_NAME: &str = "edge-assignment-kv-store";
const SDK_KEY_QUERY_PARAM: &str = "apiKey"; // For legacy reasons this is named `apiKey`

const SDK_NAME: &str = "fastly-edge-assignments";
const SDK_VERSION: &str = "0.1.0";

fn kv_store_key(token_hash: &str) -> String {
    format!("ufc-by-sdk-key-token-hash-{}", token_hash)
}

fn token_hash(sdk_key: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sdk_key.as_bytes());
    base64_url::encode(&hasher.finalize())
}

pub fn handle_assignments(mut req: Request) -> Result<Response, Error> {
    // Extract the SDK key and generate a token hash matching the pre-defined encoding.
    let token_hash = match req.get_query_parameter(SDK_KEY_QUERY_PARAM) {
        Some(key) if !key.is_empty() => token_hash(key.to_string()),
        _ => {
            return Ok(
                Response::from_status(StatusCode::BAD_REQUEST).with_body_text_plain(&format!(
                    "Missing required query parameter: {}",
                    SDK_KEY_QUERY_PARAM
                )),
            );
        }
    };

    // Deserialize the request body into a struct
    let (subject_key, subject_attributes): (eppo_core::Str, Arc<Attributes>) =
        match serde_json::from_slice::<RequestBody>(&req.take_body_bytes()) {
            Ok(body) => {
                if body.subject_key.is_empty() {
                    return Ok(Response::from_status(StatusCode::BAD_REQUEST)
                        .with_body_text_plain("subject_key is required and cannot be empty"));
                }
                (
                    eppo_core::Str::from(body.subject_key),
                    body.subject_attributes,
                )
            }
            Err(e) => {
                let error_message = if e.to_string().contains("subject_key") {
                    "subject_key is required in the request body"
                } else {
                    "Invalid request body format"
                };
                return Ok(Response::from_status(StatusCode::BAD_REQUEST)
                    .with_body_text_plain(error_message));
            }
        };

    // Open the KV store
    let kv_store = KVStore::open(KV_STORE_NAME).map(|store| store.expect("KVStore exists"))?;

    let mut kv_store_item = match kv_store.lookup(&kv_store_key(&token_hash)) {
        Ok(item) => item,
        Err(e) => {
            let (status, message) = match e {
                KVStoreError::ItemNotFound => {
                    eprintln!("Missing configuration for SDK key: {}", token_hash);

                    // Return unauthorized if the key does not exist.
                    // Our protocol lets the client know that the SDK key has not had a UFC
                    // configuration pre-computed for it in the KV Store.
                    (StatusCode::UNAUTHORIZED, "Invalid SDK key.".to_string())
                }
                _ => {
                    eprintln!("KV Store error: {:?}", e);
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
            name: SDK_NAME,
            version: SDK_VERSION,
        },
        kv_store_item_body.into_bytes(),
    ) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to parse UFC config: {:?}", e);
            return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                .with_body_text_plain("Invalid configuration format in KV store"));
        }
    };

    let configuration = Configuration::from_server_response(ufc_config, None);
    let flag_keys = configuration.flag_keys();
    let configuration_store = ConfigurationStore::new();
    configuration_store.set_configuration(Arc::new(configuration));
    let evaluator = Evaluator::new(EvaluatorConfig {
        configuration_store: Arc::new(configuration_store),
        sdk_metadata: SdkMetadata {
            name: SDK_NAME,
            version: SDK_VERSION,
        },
    });

    let subject_assignments = flag_keys
        .iter()
        .filter_map(|key| {
            match evaluator.get_assignment(key, &subject_key, &subject_attributes, None) {
                Ok(Some(assignment)) => {
                    // Extract event data if available, otherwise skip this assignment
                    assignment.event.as_ref().map(|event| {
                        (
                            key.clone(),
                            FlagAssignment {
                                allocation_key: event.base.allocation.to_string(),
                                variation_key: event.base.variation.to_string(),
                                // TODO: We need to get the variation type from the UFC config.
                                variation_type: assignment.value.variation_type().to_string(),
                                variation_value: assignment.value.variation_value(),
                                extra_logging: event
                                    .base
                                    .extra_logging
                                    .iter()
                                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                                    .collect(),
                                do_log: true,
                            },
                        )
                    })
                }
                Ok(None) => None,
                Err(e) => {
                    eprintln!("Failed to evaluate assignment for key {}: {:?}", key, e);
                    None
                }
            }
        })
        .collect::<HashMap<_, _>>();

    // Create the response
    let assignments_response = AssignmentsResponse {
        created_at: chrono::Utc::now().timestamp(),
        format: AssignmentFormat::Precomputed,
        // TODO: Need to figure out how to access the environment name.
        // from the UFC configuration but it's not public in the compiled config.
        environment: Environment {
            name: "UNKNOWN".to_string(),
        },
        flags: subject_assignments,
    };

    // Create an HTTP OK response with the assignments
    let response = match Response::from_status(StatusCode::OK).with_body_json(&assignments_response)
    {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Failed to serialize response: {:?}", e);
            return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                .with_body_text_plain("Failed to serialize response"));
        }
    };
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_hash() {
        // Test case with a known SDK key and its expected hash
        let sdk_key = "5qCSVzH1lCI11.ZWg9ZDhlYnhsLmV2ZW50cy5lcHBvLmxvY2FsaG9zdA".to_string();
        let expected_hash = "V--77TScV5Etm78nIMTSOdiroOh1__NsupwUwsetEVM";

        let result = token_hash(sdk_key);

        assert_eq!(result, expected_hash);
    }
}