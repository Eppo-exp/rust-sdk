use fastly::http::StatusCode;
use fastly::{Error, KVStore, Request, Response};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct RequestBody {
    subject_key: String,
    subject_attributes: HashMap<String, serde_json::Value>,
    #[serde(rename = "banditActions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    bandit_actions: Option<HashMap<String, serde_json::Value>>,
}

const KV_STORE_NAME: &str = "edge-assignment-kv-store";

pub fn handle_assignments(mut req: Request) -> Result<Response, Error> {
    // Parse the apiKey from the request
    let api_key = req.get_query_parameter("sdk_key").unwrap_or_default();

    // Parse the request body
    let body: RequestBody = serde_json::from_slice(&req.take_body_bytes())?;
    let subject_key = body.subject_key;
    let subject_attributes = body.subject_attributes;
    let bandit_actions = body.bandit_actions;

    // Construct an KVStore instance which is connected to the KV Store named `my-store`
    //[Documentation for the KVStore open method can be found here](https://docs.rs/fastly/latest/fastly/struct.KVStore.html#method.open)
    let mut kv_store = KVStore::open(KV_STORE_NAME).map(|store| store.expect("KVStore exists"))?;

    let mut kv_store_item = kv_store.lookup("my_key")?;
    let kv_store_item_body = kv_store_item.take_body();

    // Parse the response from the KV store

    //let ufc_config_json: Value = kv_store_item_body.take_body_json()?;
    //let ufc_config = parse_ufc_configuration(kv_store_item_body.into_bytes());
    //let client = offline_init(api_key, ufc_config);

    // let flag_keys: Vec<String> = ufc_config_json["flags"]
    //     .as_object()
    //     .unwrap()
    //     .keys()
    //     .cloned()
    //     .collect();

    let flag_keys: Vec<u8> = Vec::from("flag1");

    // let mut assignment_cache: HashMap<String, AssignmentValue> = HashMap::new();

    //for flag_key in &flag_keys {
    // let subject_key = eppo_core::Str::from(subject_key);
    // let assignment = client.get_assignment(flag_key, &subject_key, &subject_attributes);
    // let variation_value: eppo::AssignmentValue = match assignment {
    //     Ok(Some(value)) => value.clone(),
    //     Ok(None) => eppo::AssignmentValue::Json(Arc::new(json!(null))),
    //     Err(_) => eppo::AssignmentValue::Json(Arc::new(json!(null))),
    // };
    // assignment_cache.insert(flag_key.to_string(), variation_value);
    //}

    // Create an HTTP OK response
    let response = Response::from_status(StatusCode::OK)
        .with_body_text_plain("Request processed successfully");

    Ok(response)
}
