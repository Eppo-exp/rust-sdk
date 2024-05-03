use std::collections::HashMap;

pub fn main() {
    let api_key = std::env::var("EPPO_API_KEY").unwrap();
    let mut client = eppo::ClientConfig::from_api_key(api_key).to_client();

    // Start a poller thread to fetch configuration from the server.
    let poller = client.start_poller_thread();

    // Block waiting for configuration. Until this call returns, the client will return None for all
    // assignments.
    poller.wait_for_configuration();

    // Get assignment for test-subject.
    let assignment = client
        .get_assignment("a-boolean-flag", "test-subject", &HashMap::new())
        .and_then(|x| x.as_boolean())
        // default assignment
        .unwrap_or(false);

    println!("Assignment: {:?}", assignment);
}
