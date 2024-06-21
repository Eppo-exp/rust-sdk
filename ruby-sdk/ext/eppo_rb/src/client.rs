use std::{cell::RefCell, sync::Arc};

use eppo_core::{
    configuration_fetcher::ConfigurationFetcher,
    configuration_store::ConfigurationStore,
    poller_thread::PollerThread,
    ufc::{AssignmentEvent, AssignmentValue, VariationType},
    Attributes, Configuration,
};
use magnus::{
    error::Result,
    exception::{self, exception},
    prelude::*,
    Error, IntoValue, RHash, RString, Symbol, TryConvert, Value,
};

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

#[derive(Debug, serde::Serialize)]
pub struct Assignment {
    value: Option<AssignmentValue>,
    event: Option<AssignmentEvent>,
}
impl Assignment {
    const fn empty() -> Assignment {
        Assignment {
            value: None,
            event: None,
        }
    }
}
impl IntoValue for Assignment {
    fn into_value_with(self, handle: &magnus::Ruby) -> Value {
        serde_magnus::serialize(&self).expect("Assignment value should be serializable")
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
    ) -> Result<Assignment> {
        let expected_type: VariationType = serde_magnus::deserialize(expected_type)?;
        let subject_attributes: Attributes = serde_magnus::deserialize(subject_attributes)?;

        let Configuration { ufc: Some(ufc) } = self.configuration_store.get_configuration() else {
            log::warn!(target: "eppo", flag_key, subject_key; "evaluating a flag before Eppo configuration has been fetched");
            // We treat missing configuration (the poller has not fetched config) as a normal
            // scenario (at least for now).
            return Ok(Assignment::empty());
        };

        let evaluation = match ufc.eval_flag(
            &flag_key,
            &subject_key,
            &subject_attributes,
            Some(expected_type),
        ) {
            Ok(result) => result,
            Err(err) => {
                log::warn!(target: "eppo",
                           flag_key,
                           subject_key,
                           subject_attributes:serde;
                           "error occurred while evaluating a flag: {:?}", err,
                );
                return Err(Error::new(exception::runtime_error(), "blah"));
                // return Err(err);
            }
        };

        log::trace!(target: "eppo",
                    flag_key,
                    subject_key,
                    subject_attributes:serde,
                    assignment:serde = evaluation.as_ref().map(|(value, _event)| value);
                    "evaluated a flag");

        let Some((value, event)) = evaluation else {
            return Ok(Assignment::empty());
        };

        Ok(Assignment {
            value: Some(value),
            event,
        })
    }

    pub fn shutdown(&self) {
        if let Some(t) = self.poller_thread.take() {
            let _ = t.shutdown();
        }
    }
}
