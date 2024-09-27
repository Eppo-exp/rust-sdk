use std::{cell::RefCell, sync::Arc};

use eppo_core::{
    configuration_fetcher::{ConfigurationFetcher, ConfigurationFetcherConfig},
    configuration_store::ConfigurationStore,
    eval::{Evaluator, EvaluatorConfig},
    poller_thread::PollerThread,
    ufc::VariationType,
    Attributes, ContextAttributes, SdkMetadata,
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
    evaluator: Evaluator,
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

        let sdk_metadata = SdkMetadata {
            name: "ruby",
            version: env!("CARGO_PKG_VERSION"),
        };

        let poller_thread = PollerThread::start(
            ConfigurationFetcher::new(ConfigurationFetcherConfig {
                base_url: config.base_url,
                api_key: config.api_key,
                sdk_metadata: sdk_metadata.clone(),
            }),
            configuration_store.clone(),
        )
        .expect("should be able to start poller thread");

        let evaluator = Evaluator::new(EvaluatorConfig {
            configuration_store,
            sdk_metadata,
        });

        Client {
            evaluator,
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

        let result = self
            .evaluator
            .get_assignment(
                &flag_key,
                &subject_key.into(),
                &Arc::new(subject_attributes),
                Some(expected_type),
            )
            // TODO: maybe expose possible errors individually.
            .map_err(|err| Error::new(exception::runtime_error(), err.to_string()))?;

        Ok(serde_magnus::serialize(&result).expect("assignment value should be serializable"))
    }

    pub fn get_assignment_details(
        &self,
        flag_key: String,
        subject_key: String,
        subject_attributes: Value,
        expected_type: Value,
    ) -> Result<Value> {
        let expected_type: VariationType = serde_magnus::deserialize(expected_type)?;
        let subject_attributes: Attributes = serde_magnus::deserialize(subject_attributes)?;

        let result = self.evaluator.get_assignment_details(
            &flag_key,
            &subject_key.into(),
            &Arc::new(subject_attributes),
            Some(expected_type),
        );

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

        let result = self.evaluator.get_bandit_action(
            &flag_key,
            &subject_key.into(),
            &subject_attributes,
            &actions,
            &default_variation.into(),
        );

        serde_magnus::serialize(&result)
    }

    pub fn get_bandit_action_details(
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

        let result = self.evaluator.get_bandit_action_details(
            &flag_key,
            &subject_key.into_boxed_str().into(),
            &subject_attributes,
            &actions,
            &default_variation.into(),
        );

        serde_magnus::serialize(&result)
    }

    pub fn shutdown(&self) {
        if let Some(t) = self.poller_thread.take() {
            let _ = t.shutdown();
        }
    }
}
