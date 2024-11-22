use std::{cell::RefCell, str::FromStr, sync::Arc, time::Duration};

use eppo_core::{
    configuration_fetcher::{ConfigurationFetcher, ConfigurationFetcherConfig},
    configuration_store::ConfigurationStore,
    eval::{Evaluator, EvaluatorConfig},
    poller_thread::{PollerThread, PollerThreadConfig},
    ufc::VariationType,
    Attributes, ContextAttributes,
};
use magnus::{error::Result, exception, prelude::*, Error, TryConvert, Value};

use crate::{configuration::Configuration, SDK_METADATA};

#[derive(Debug)]
#[magnus::wrap(class = "EppoClient::Core::Config", size, free_immediately)]
pub struct Config {
    api_key: String,
    base_url: String,
    poll_interval: Option<Duration>,
    poll_jitter: Duration,
    log_level: Option<log::LevelFilter>,
}

impl TryConvert for Config {
    // `val` is expected to be of type EppoClient::Config.
    fn try_convert(val: magnus::Value) -> Result<Self> {
        let api_key = String::try_convert(val.funcall("api_key", ())?)?;
        let base_url = String::try_convert(val.funcall("base_url", ())?)?;
        let poll_interval_seconds =
            Option::<u64>::try_convert(val.funcall("poll_interval_seconds", ())?)?;
        let poll_jitter_seconds = u64::try_convert(val.funcall("poll_jitter_seconds", ())?)?;

        let log_level = {
            let s = Option::<String>::try_convert(val.funcall("log_level", ())?)?;
            s.map(|s| {
                log::LevelFilter::from_str(&s)
                    .map_err(|err| Error::new(exception::runtime_error(), err.to_string()))
            })
            .transpose()?
        };

        Ok(Config {
            api_key,
            base_url,
            poll_interval: poll_interval_seconds.map(Duration::from_secs),
            poll_jitter: Duration::from_secs(poll_jitter_seconds),
            log_level,
        })
    }
}

#[magnus::wrap(class = "EppoClient::Core::Client")]
pub struct Client {
    configuration_store: Arc<ConfigurationStore>,
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
        // Initialize logger
        {
            let mut builder = env_logger::Builder::from_env(
                env_logger::Env::new()
                    .filter_or("EPPO_LOG", "eppo=info")
                    .write_style("EPPO_LOG_STYLE"),
            );

            if let Some(log_level) = config.log_level {
                builder.filter_module("eppo", log_level);
            }

            // Logger can only be set once, so we ignore the initialization error here if client is
            // re-initialized.
            let _ = builder.try_init();
        };

        let configuration_store = Arc::new(ConfigurationStore::new());

        let poller_thread = if let Some(poll_interval) = config.poll_interval {
            Some(
                PollerThread::start_with_config(
                    ConfigurationFetcher::new(ConfigurationFetcherConfig {
                        base_url: config.base_url,
                        api_key: config.api_key,
                        sdk_metadata: SDK_METADATA,
                    }),
                    configuration_store.clone(),
                    PollerThreadConfig {
                        interval: poll_interval,
                        jitter: config.poll_jitter,
                    },
                )
                .expect("should be able to start poller thread"),
            )
        } else {
            None
        };

        let evaluator = Evaluator::new(EvaluatorConfig {
            configuration_store: configuration_store.clone(),
            sdk_metadata: SDK_METADATA,
        });

        Client {
            configuration_store,
            evaluator,
            poller_thread: RefCell::new(poller_thread),
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
            &subject_key.into(),
            &subject_attributes,
            &actions,
            &default_variation.into(),
        );

        serde_magnus::serialize(&result)
    }

    pub fn get_configuration(&self) -> Option<Configuration> {
        self.configuration_store
            .get_configuration()
            .map(|it| it.into())
    }

    pub fn set_configuration(&self, configuration: &Configuration) {
        self.configuration_store
            .set_configuration(configuration.clone().into())
    }

    pub fn shutdown(&self) {
        if let Some(t) = self.poller_thread.take() {
            let _ = t.shutdown();
        }
    }
}
