use std::collections::HashMap;

pub fn main() -> eppo::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("eppo")).init();

    let api_key = std::env::var("EPPO_API_KEY").unwrap();
    let mut client = eppo::ClientConfig::from_api_key(api_key).to_client();

    // Start a poller thread to fetch configuration from the server.
    let poller = client.start_poller_thread()?;

    // Block waiting for configuration. Until this call returns, the client will return None for all
    // assignments.
    if let Err(err) = poller.wait_for_configuration() {
        println!("error requesting configuration: {:?}", err);
    }

    // Get assignment for test-subject.
    let assignment = client
        .get_assignment("a-boolean-flag", "test-subject", &HashMap::new())
        .unwrap_or_default()
        .and_then(|x| x.as_boolean())
        // default assignment
        .unwrap_or(false);

    println!("Assignment: {:?}", assignment);

    Ok(())
}
