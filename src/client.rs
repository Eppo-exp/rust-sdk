use std::{collections::HashMap, sync::Arc};

use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::{
    configuration_store::ConfigurationStore,
    poller::{PollerThread, PollerThreadConfig},
    sharder::Md5Sharder,
    ClientConfig, Result,
};

/// A client for Eppo API.
///
/// In order to create a client instance, first create [`ClientConfig`].
///
/// # Examples
/// ```
/// # use eppo::{EppoClient, ClientConfig};
/// EppoClient::new(ClientConfig::from_api_key("api-key"));
/// ```
pub struct EppoClient<'a> {
    configuration_store: Arc<ConfigurationStore>,
    config: ClientConfig<'a>,
}

impl<'a> EppoClient<'a> {
    /// Create a new `EppoClient` using the specified configuration.
    ///
    /// ```
    /// # use eppo::{ClientConfig, EppoClient};
    /// let client = EppoClient::new(ClientConfig::from_api_key("api-key"));
    /// ```
    pub fn new(config: ClientConfig<'a>) -> Self {
        EppoClient {
            configuration_store: Arc::new(ConfigurationStore::new()),
            config,
        }
    }

    #[cfg(test)]
    fn new_with_configuration_store(
        config: ClientConfig<'a>,
        configuration_store: Arc<ConfigurationStore>,
    ) -> Self {
        Self {
            configuration_store,
            config,
        }
    }

    /// Get variation assignment for the given subject.
    pub fn get_assignment(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &SubjectAttributes,
    ) -> Result<Option<AssignmentValue>> {
        let Some(configuration) = self.configuration_store.get_configuration() else {
            log::warn!(target: "eppo", flag_key, subject_key; "evaluating a flag before Eppo configuration has been fetched");
            // We treat missing configuration (the poller has not fetched config) as a normal
            // scenario (at least for now).
            return Ok(None);
        };

        let evaluation = configuration
            .eval_flag(flag_key, subject_key, subject_attributes, &Md5Sharder)
            .inspect_err(|err| {
                log::warn!(target: "eppo",
                    flag_key,
                    subject_key,
                    subject_attributes:serde;
                    "error occurred while evaluating a flag: {:?}", err,
                );
            })?;

        log::trace!(target: "eppo",
                    flag_key,
                    subject_key,
                    subject_attributes:serde,
                    assignment:serde = evaluation.as_ref().map(|(value, _event)| value);
                    "evaluated a flag");

        let Some((value, event)) = evaluation else {
            return Ok(None);
        };

        if let Some(event) = event {
            log::trace!(target: "eppo",
                        event:serde;
                        "logging assignment");
            self.config.assignment_logger.log_assignment(event);
        }

        Ok(Some(value))
    }

    /// Start a poller thread to fetch configuration from the server.
    pub fn start_poller_thread(&mut self) -> Result<PollerThread> {
        PollerThread::start(PollerThreadConfig {
            store: self.configuration_store.clone(),
            base_url: self.config.base_url.clone(),
            api_key: self.config.api_key.clone(),
        })
    }
}

pub type SubjectAttributes = HashMap<String, AttributeValue>;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, From, Clone)]
#[serde(untagged)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}
impl From<&str> for AttributeValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum AssignmentValue {
    String(String),
    Integer(i64),
    Numeric(f64),
    Boolean(bool),
    Json(serde_json::Value),
}

impl AssignmentValue {
    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            AssignmentValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_integer(&self) -> bool {
        self.as_integer().is_some()
    }
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            AssignmentValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn is_numeric(&self) -> bool {
        self.as_numeric().is_some()
    }
    pub fn as_numeric(&self) -> Option<f64> {
        match self {
            Self::Numeric(n) => Some(*n),
            _ => None,
        }
    }

    pub fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            AssignmentValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn is_json(&self) -> bool {
        self.as_json().is_some()
    }
    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Json(v) => Some(v),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use crate::{
        client::AssignmentValue,
        configuration_store::ConfigurationStore,
        ufc::{Allocation, Flag, Split, TryParse, UniversalFlagConfig, Variation, VariationType},
        ClientConfig, EppoClient,
    };

    #[test]
    fn returns_none_while_no_configuration() {
        let configuration_store = Arc::new(ConfigurationStore::new());
        let client = EppoClient::new_with_configuration_store(
            ClientConfig::from_api_key("api-key"),
            configuration_store.clone(),
        );

        assert_eq!(
            client
                .get_assignment("flag", "subject", &HashMap::new())
                .unwrap(),
            None
        );
    }

    #[test]
    fn returns_proper_configuration_once_config_is_fetched() {
        let configuration_store = Arc::new(ConfigurationStore::new());
        let client = EppoClient::new_with_configuration_store(
            ClientConfig::from_api_key("api-key"),
            configuration_store.clone(),
        );

        // updating configuration after client is created
        configuration_store.set_configuration(UniversalFlagConfig {
            flags: [(
                "flag".to_owned(),
                TryParse::Parsed(Flag {
                    key: "flag".to_owned(),
                    enabled: true,
                    variation_type: VariationType::Boolean,
                    variations: [(
                        "variation".to_owned(),
                        Variation {
                            key: "variation".to_owned(),
                            value: true.into(),
                        },
                    )]
                    .into(),
                    allocations: vec![Allocation {
                        key: "allocation".to_owned(),
                        rules: vec![],
                        start_at: None,
                        end_at: None,
                        splits: vec![Split {
                            shards: vec![],
                            variation_key: "variation".to_owned(),
                            extra_logging: HashMap::new(),
                        }],
                        do_log: false,
                    }],
                    total_shards: 10_000,
                }),
            )]
            .into(),
        });

        assert_eq!(
            client
                .get_assignment("flag", "subject", &HashMap::new())
                .unwrap(),
            Some(AssignmentValue::Boolean(true))
        );
    }
}
