mod handlers;

use fastly::http::{Method, StatusCode};
use fastly::{Error, Request, Response};

// fn parse_ufc_configuration(ufc_config_json: Vec<u8>) -> UniversalFlagConfig {
//     //let config_json_bytes: Vec<u8> = serde_json::to_vec(&ufc_config_json).unwrap();
//     UniversalFlagConfig::from_json(
//         SdkMetadata {
//             name: "rust-sdk",
//             version: "4.0.1",
//         },
//         ufc_config_json,
//     )
//     .unwrap()
// }

// fn offline_init(api_key: &str, ufc_config: UniversalFlagConfig) -> eppo::Client {
//     let config = Configuration::from_server_response(ufc_config, None);
//     let config_store = eppo_core::configuration_store::ConfigurationStore::new();
//     config_store.set_configuration(Arc::new(config));
//     let client = eppo::Client::new_with_configuration_store(
//         ClientConfig::from_api_key(api_key),
//         config_store.into(),
//     );
//     return client;
// }

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    match (req.get_method(), req.get_path()) {
        (&Method::POST, "/assignments") => handlers::handle_assignments(req),
        (&Method::GET, "/health") => handlers::handle_health(req),
        _ => Ok(Response::from_status(StatusCode::NOT_FOUND).with_body_text_plain("Not Found")),
    }
}
