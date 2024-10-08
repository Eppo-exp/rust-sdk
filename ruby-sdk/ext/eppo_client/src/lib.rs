mod client;
mod configuration;
mod gc_lock;

use eppo_core::SdkMetadata;
use magnus::{function, method, prelude::*, Error, Object, Ruby};

use crate::client::Client;

pub(crate) const SDK_METADATA: SdkMetadata = SdkMetadata {
    name: "ruby",
    version: env!("CARGO_PKG_VERSION"),
};

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("eppo=debug")).init();

    let eppo_client = ruby.define_module("EppoClient")?;
    let core = eppo_client.define_module("Core")?;

    let core_client = core.define_class("Client", magnus::class::object())?;
    core_client.define_singleton_method("new", function!(Client::new, 1))?;
    core_client.define_method("get_assignment", method!(Client::get_assignment, 4))?;
    core_client.define_method(
        "get_assignment_details",
        method!(Client::get_assignment_details, 4),
    )?;
    core_client.define_method("get_bandit_action", method!(Client::get_bandit_action, 5))?;
    core_client.define_method(
        "get_bandit_action_details",
        method!(Client::get_bandit_action_details, 5),
    )?;
    core_client.define_method("configuration", method!(Client::get_configuration, 0))?;
    core_client.define_method("configuration=", method!(Client::set_configuration, 1))?;
    core_client.define_method("shutdown", method!(Client::shutdown, 0))?;

    core.const_set(
        "DEFAULT_BASE_URL",
        eppo_core::configuration_fetcher::DEFAULT_BASE_URL,
    )?;
    core.const_set(
        "DEFAULT_POLL_INTERVAL_SECONDS",
        eppo_core::poller_thread::PollerThreadConfig::DEFAULT_POLL_INTERVAL.as_secs(),
    )?;
    core.const_set(
        "DEFAULT_POLL_JITTER_SECONDS",
        eppo_core::poller_thread::PollerThreadConfig::DEFAULT_POLL_JITTER.as_secs(),
    )?;

    configuration::init(ruby)?;

    Ok(())
}
