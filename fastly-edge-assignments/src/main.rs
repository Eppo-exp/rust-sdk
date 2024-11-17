mod handlers;

use fastly::http::{Method, StatusCode};
use fastly::{Error, Request, Response};

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
