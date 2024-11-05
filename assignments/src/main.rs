use eppo::ClientConfig;
use eppo_core::ufc::UniversalFlagConfig;
use eppo_core::{Configuration, SdkMetadata};
use fastly::http::{Method, StatusCode};
use fastly::{Error, Request, Response};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

fn get_subject_attributes(req: &Request) -> Value {
    req.get_query_parameter("subjectAttributes")
        .and_then(|v| serde_json::from_str(v).ok())
        .unwrap_or_else(|| json!({}))
}

fn parse_ufc_configuration(ufc_config_json: Value) -> UniversalFlagConfig {
    let config_json_bytes: Vec<u8> = serde_json::to_vec(&ufc_config_json).unwrap();
    let ufc_config: UniversalFlagConfig = UniversalFlagConfig::from_json(
        SdkMetadata {
            name: "rust-sdk",
            version: "4.0.1",
        },
        config_json_bytes,
    )
    .unwrap();
    return ufc_config;
}

fn offline_init(api_key: &str, ufc_config: UniversalFlagConfig) -> eppo::Client {
    let config = Configuration::from_server_response(ufc_config, None);
    let config_store = eppo_core::configuration_store::ConfigurationStore::new();
    config_store.set_configuration(Arc::new(config));
    let client = eppo::Client::new_with_configuration_store(
        ClientConfig::from_api_key(api_key),
        config_store.into(),
    );
    return client;
}

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    let start_time: Instant = Instant::now();

    match (req.get_method(), req.get_path()) {
        (&Method::GET, "/api/flag-config/v1/assignments") => {
            let api_key = req.get_query_parameter("apiKey").unwrap_or_default();
            let subject_key = req.get_query_parameter("subjectKey").unwrap_or_default();
            let subject_attributes: Arc<HashMap<String, eppo::AttributeValue>> = Arc::new(
                serde_json::from_value(get_subject_attributes(&req))
                    .unwrap_or_else(|_| HashMap::new()),
            );

            let url = format!(
                "https://fscdn.eppo.cloud/api/flag-config/v1/config?apiKey={}",
                api_key
            );
            let mut response: Response = Request::get(url).send("eppo_cloud").unwrap();
            if response.get_status() != StatusCode::OK {
                let error_body: Value = response.take_body_json()?;
                return Ok(Response::from_status(response.get_status())
                    .with_body_json(&error_body)
                    .unwrap());
            }

            let ufc_config_json: Value = response.take_body_json()?;
            let ufc_config = parse_ufc_configuration(ufc_config_json.clone());
            let client = offline_init(api_key, ufc_config);

            let flag_keys: Vec<String> = ufc_config_json["flags"]
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect();

            let mut assignment_cache = HashMap::new();

            for flag_key in &flag_keys {
                let subject_key = eppo_core::Str::from(subject_key);
                let assignment = client.get_assignment(flag_key, &subject_key, &subject_attributes);
                let variation_value: eppo::AssignmentValue = match assignment {
                    Ok(Some(value)) => value.clone(),
                    Ok(None) => eppo::AssignmentValue::Json(Arc::new(json!(null))),
                    Err(_) => eppo::AssignmentValue::Json(Arc::new(json!(null))),
                };
                assignment_cache.insert(flag_key.to_string(), variation_value);
            }

            let cpu_time_used = start_time.elapsed().as_millis();
            println!("CPU time used: {} ms", cpu_time_used);

            Ok(Response::new()
                .with_status(200)
                .with_body_json(&assignment_cache)?)
        }
        _ => Ok(Response::from_status(StatusCode::NOT_FOUND)),
    }
}
