use std::collections::HashMap;

pub fn main() -> eppo::Result<()> {
    // Configure env_logger to see Eppo SDK logs.
    env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("eppo")).init();

    let api_key =
        std::env::var("EPPO_API_KEY").expect("EPPO_API_KEY env variable should contain API key");
    let mut client = eppo::ClientConfig::from_api_key(api_key)
        .assignment_logger(|event| {
            println!("Logging assignment event: {:?}", event);
        })
        .to_client();

    // Start a poller thread to fetch configuration from the server.
    let poller = client.start_poller_thread()?;

    // Block waiting for configuration. Until this call returns, the client will return None for all
    // assignments.
    if let Err(err) = poller.wait_for_configuration() {
        println!("error requesting configuration: {:?}", err);
    }

    // Get assignment for test-subject.
    let assignment = client
        .get_boolean_assignment("a-boolean-flag", "test-subject", &HashMap::new())
        .unwrap_or_default()
        // default assignment
        .unwrap_or(false);

    println!("Assignment: {:?}", assignment);

    Ok(())
}
