use std::{cell::RefCell, sync::Arc};

use eppo_core::{
    configuration_fetcher::ConfigurationFetcher,
    configuration_store::ConfigurationStore,
    eval::{get_assignment, get_bandit_action},
    poller_thread::PollerThread,
    ufc::VariationType,
    Attributes, ContextAttributes,
};
use magnus::{error::Result, exception, prelude::*, Error, TryConvert, Value};

#[derive(Debug)]
#[magnus::wrap(class = "EppoClient::Core::Config", size, free_immediately)]
pub struct Config {
    api_key: String,
    base_url: String,
}

impl TryConvert for Config {
    // `val` is expected to be of type EppoClient::Config.
    fn try_convert(val: magnus::Value) -> Result<Self> {
        let api_key = String::try_convert(val.funcall("api_key", ())?)?;
        let base_url = String::try_convert(val.funcall("base_url", ())?)?;
        Ok(Config { api_key, base_url })
    }
}

#[magnus::wrap(class = "EppoClient::Core::Client")]
pub struct Client {
    configuration_store: Arc<ConfigurationStore>,
    // Magnus only allows sharing aliased references (&T) through the API, so we need to use RefCell
    // to get interior mutability.
    //
    // This should be safe as Ruby only uses a single OS thread, and `Client` lives in the Ruby
    // world.
    poller_thread: RefCell<Option<PollerThread>>,
}

impl Client {
    pub fn new(config: Config) -> Client {
        let configuration_store = Arc::new(ConfigurationStore::new());

        let poller_thread = PollerThread::start(
            ConfigurationFetcher::new(
                eppo_core::configuration_fetcher::ConfigurationFetcherConfig {
                    base_url: config.base_url,
                    api_key: config.api_key,
                    sdk_name: "ruby".to_owned(),
                    sdk_version: env!("CARGO_PKG_VERSION").to_owned(),
                },
            ),
            configuration_store.clone(),
        )
        .expect("should be able to start poller thread");

        Client {
            configuration_store,
            poller_thread: RefCell::new(Some(poller_thread)),
        }
    }

    pub fn get_assignment(
        &self,
        flag_key: String,
        subject_key: String,
        subject_attributes: Value,
        expected_type: Value,
    ) -> Result<Value> {
        let expected_type: VariationType = serde_magnus::deserialize(expected_type)?;
        let subject_attributes: Attributes = serde_magnus::deserialize(subject_attributes)?;

        let config = self.configuration_store.get_configuration();
        let result = get_assignment(
            config.as_ref().map(AsRef::as_ref),
            &flag_key,
            &subject_key,
            &subject_attributes,
            Some(expected_type),
        )
        // TODO: maybe expose possible errors individually.
        .map_err(|err| Error::new(exception::runtime_error(), err.to_string()))?;

        Ok(serde_magnus::serialize(&result).expect("assignment value should be serializable"))
    }

    pub fn get_bandit_action(
        &self,
        flag_key: String,
        subject_key: String,
        subject_attributes: Value,
        actions: Value,
        default_variation: String,
    ) -> Result<Value> {
        let subject_attributes = serde_magnus::deserialize::<_, ContextAttributes>(
            subject_attributes,
        )
        .map_err(|err| {
            Error::new(
                exception::runtime_error(),
                format!("enexpected value for subject_attributes: {err}"),
            )
        })?;
        let actions = serde_magnus::deserialize(actions)?;

        let config = self.configuration_store.get_configuration();
        let result = get_bandit_action(
            config.as_ref().map(AsRef::as_ref),
            &flag_key,
            &subject_key,
            &subject_attributes,
            &actions,
            &default_variation,
        );

        serde_magnus::serialize(&result)
    }

    pub fn shutdown(&self) {
        if let Some(t) = self.poller_thread.take() {
            let _ = t.shutdown();
        }
    }
}
