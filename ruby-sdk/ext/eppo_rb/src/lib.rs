mod client;

use magnus::{function, method, prelude::*, Error, Object, Ruby};

use crate::client::Client;

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("eppo")).init();

    let eppo_client = ruby.define_module("EppoClient")?;
    let core = eppo_client.define_module("Core")?;

    let core_client = core.define_class("Client", magnus::class::object())?;
    core_client.define_singleton_method("new", function!(Client::new, 1))?;
    core_client.define_method("get_assignment", method!(Client::get_assignment, 4))?;
    core_client.define_method("get_bandit_action", method!(Client::get_bandit_action, 5))?;
    core_client.define_method("shutdown", method!(Client::shutdown, 0))?;

    core.const_set(
        "DEFAULT_BASE_URL",
        eppo_core::configuration_fetcher::DEFAULT_BASE_URL,
    )?;

    Ok(())
}
