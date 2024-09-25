use std::sync::Arc;

use crate::{
    poller::{PollerThread, PollerThreadConfig},
    AssignmentValue, Attributes, ClientConfig, Error, EvaluationError, EvaluationResultWithDetails,
    SDK_METADATA,
};

use eppo_core::{
    configuration_store::ConfigurationStore,
    eval::{Evaluator, EvaluatorConfig},
    ufc::{Assignment, VariationType},
};

/// A client for Eppo API.
///
/// In order to create a client instance, first create [`ClientConfig`].
///
/// # Poller Thread
///
/// Before calling `Client::get_assignment()`, you should start the poller thread by calling
/// [`Client::start_poller_thread()`], ensuring that the configuration is fetched. It's also
/// recommended to call [`PollerThread::wait_for_configuration`] before calling `get_assignment()`.
///
/// The reason the poller thread is not started automatically is to allow SDK extension to support
/// `async` configuration fetching in the future (using async Rust runtimes).
///
/// # Examples
/// ```no_run
/// # use eppo::{Client, ClientConfig};
/// let mut client = Client::new(ClientConfig::from_api_key("api-key"));
/// client.start_poller_thread();
/// ```
pub struct Client<'a> {
    config: ClientConfig<'a>,
    configuration_store: Arc<ConfigurationStore>,
    evaluator: Evaluator,
}

impl<'a> Client<'a> {
    /// Create a new `Client` using the specified configuration.
    ///
    /// ```
    /// # use eppo::{ClientConfig, Client};
    /// let client = Client::new(ClientConfig::from_api_key("api-key"));
    /// ```
    pub fn new(config: ClientConfig<'a>) -> Self {
        Client::new_with_configuration_store(config, Arc::new(ConfigurationStore::new()))
    }

    fn new_with_configuration_store(
        config: ClientConfig<'a>,
        configuration_store: Arc<ConfigurationStore>,
    ) -> Self {
        let evaluator = Evaluator::new(EvaluatorConfig {
            configuration_store: configuration_store.clone(),
            sdk_metadata: SDK_METADATA.clone(),
        });
        Self {
            configuration_store,
            config,
            evaluator,
        }
    }

    /// Get the assignment value for a given feature flag and subject.
    ///
    /// If the subject is not eligible for any allocation, returns `Ok(None)`.
    ///
    /// If the configuration has not been fetched yet, returns `Ok(None)`.
    /// You should call [`Client::start_poller_thread`] before any call to
    /// `get_assignment()`. Otherwise, the client will always return `None`.
    ///
    /// It is recommended to wait for the Eppo configuration to get fetched with
    /// [`PollerThread::wait_for_configuration()`].
    ///
    /// # Typed versions
    ///
    /// There are typed versions of this function:
    /// - [`Client::get_string_assignment()`]
    /// - [`Client::get_integer_assignment()`]
    /// - [`Client::get_numeric_assignment()`]
    /// - [`Client::get_boolean_assignment()`]
    /// - [`Client::get_json_assignment()`]
    ///
    /// It is recommended to use typed versions of this function as they provide additional type
    /// safety. They can catch type errors even _before_ evaluating the assignment, which helps to
    /// detect errors if subject is not eligible for the flag allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn test(client: &eppo::Client) {
    /// let assignment = client
    ///     .get_assignment(
    ///         "a-boolean-flag",
    ///         "user-id",
    ///         &[("age".to_owned(), 42.0.into())]
    ///             .into_iter()
    ///             .collect(),
    ///     )
    ///     .unwrap_or_default()
    ///     .and_then(|x| x.as_boolean())
    ///     // default assignment
    ///     .unwrap_or(false);
    /// # }
    /// ```
    pub fn get_assignment(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
    ) -> Result<Option<AssignmentValue>, EvaluationError> {
        self.get_assignment_inner(flag_key, subject_key, subject_attributes, None, |x| x)
    }

    /// Retrieves the assignment value for a given feature flag and subject.
    ///
    /// If the subject is not eligible for any allocation, returns `Ok(None)`.
    ///
    /// If the configuration has not been fetched yet, returns `Ok(None)`.
    /// You should call [`Client::start_poller_thread`] before any call to
    /// `get_string_assignment()`. Otherwise, the client will always return `None`.
    ///
    /// It is recommended to wait for the Eppo configuration to get fetched with
    /// [`PollerThread::wait_for_configuration()`].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn test(client: &eppo::Client) {
    /// let assignment = client
    ///     .get_string_assignment("a-string-flag", "user-id", &[
    ///         ("language".into(), "en".into())
    ///     ].into_iter().collect())
    ///     .unwrap_or_default()
    ///     .unwrap_or("default_value".to_owned());
    /// # }
    /// ```
    pub fn get_string_assignment(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
    ) -> Result<Option<String>, EvaluationError> {
        self.get_assignment_inner(
            flag_key,
            subject_key,
            subject_attributes,
            Some(VariationType::String),
            |x| {
                x.to_string()
                    // The unwrap cannot fail because the type is checked during evaluation.
                    .unwrap()
            },
        )
    }

    /// Retrieves the assignment value for a given feature flag and subject as an integer value.
    ///
    /// If the subject is not eligible for any allocation, returns `Ok(None)`.
    ///
    /// If the configuration has not been fetched yet, returns `Ok(None)`.
    /// You should call [`Client::start_poller_thread`] before any call to
    /// `get_integer_assignment()`. Otherwise, the client will always return `None`.
    ///
    /// It is recommended to wait for the Eppo configuration to get fetched with
    /// [`PollerThread::wait_for_configuration()`].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn test(client: &eppo::Client) {
    /// let assignment = client
    ///     .get_integer_assignment("an-int-flag", "user-id", &[
    ///         ("age".to_owned(), 42.0.into())
    ///     ].into_iter().collect())
    ///     .unwrap_or_default()
    ///     .unwrap_or(0);
    /// # }
    /// ```
    pub fn get_integer_assignment(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
    ) -> Result<Option<i64>, EvaluationError> {
        self.get_assignment_inner(
            flag_key,
            subject_key,
            subject_attributes,
            Some(VariationType::Integer),
            |x| {
                x.as_integer()
                    // The unwrap cannot fail because the type is checked during evaluation.
                    .unwrap()
            },
        )
    }

    /// Retrieves the assignment value for a given feature flag and subject as a numeric value.
    ///
    /// If the subject is not eligible for any allocation, returns `Ok(None)`.
    ///
    /// If the configuration has not been fetched yet, returns `Ok(None)`.
    /// You should call [`Client::start_poller_thread`] before any call to
    /// `get_numeric_assignment()`. Otherwise, the client will always return `None`.
    ///
    /// It is recommended to wait for the Eppo configuration to get fetched with
    /// [`PollerThread::wait_for_configuration()`].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn test(client: &eppo::Client) {
    /// let assignment = client
    ///     .get_numeric_assignment("a-num-flag", "user-id", &[
    ///         ("age".to_owned(), 42.0.into())
    ///     ].iter().cloned().collect())
    ///     .unwrap_or_default()
    ///     .unwrap_or(0.0);
    /// # }
    /// ```
    pub fn get_numeric_assignment(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
    ) -> Result<Option<f64>, EvaluationError> {
        self.get_assignment_inner(
            flag_key,
            subject_key,
            subject_attributes,
            Some(VariationType::Numeric),
            |x| {
                x.as_numeric()
                    // The unwrap cannot fail because the type is checked during evaluation.
                    .unwrap()
            },
        )
    }

    /// Retrieves the assignment value for a given feature flag and subject as a boolean value.
    ///
    /// If the subject is not eligible for any allocation, returns `Ok(None)`.
    ///
    /// If the configuration has not been fetched yet, returns `Ok(None)`.
    /// You should call [`Client::start_poller_thread`] before any call to
    /// `get_boolean_assignment()`. Otherwise, the client will always return `None`.
    ///
    /// It is recommended to wait for the Eppo configuration to get fetched with
    /// [`PollerThread::wait_for_configuration()`].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn test(client: &eppo::Client) {
    /// let assignment = client
    ///     .get_boolean_assignment("a-bool-flag", "user-id", &[
    ///         ("age".to_owned(), 42.0.into())
    ///     ].into_iter().collect())
    ///     .unwrap_or_default()
    ///     .unwrap_or(false);
    /// # }
    /// ```
    pub fn get_boolean_assignment(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
    ) -> Result<Option<bool>, EvaluationError> {
        self.get_assignment_inner(
            flag_key,
            subject_key,
            subject_attributes,
            Some(VariationType::Boolean),
            |x| {
                x.as_boolean()
                    // The unwrap cannot fail because the type is checked during evaluation.
                    .unwrap()
            },
        )
    }

    /// Retrieves the assignment value for a given feature flag and subject as a JSON value.
    ///
    /// If the subject is not eligible for any allocation, returns `Ok(None)`.
    ///
    /// If the configuration has not been fetched yet, returns `Ok(None)`.
    /// You should call [`Client::start_poller_thread`] before any call to
    /// `get_json_assignment()`. Otherwise, the client will always return `None`.
    ///
    /// It is recommended to wait for the Eppo configuration to get fetched with
    /// [`PollerThread::wait_for_configuration()`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// # fn test(client: &eppo::Client) {
    /// let assignment = client
    ///     .get_json_assignment("a-json-flag", "user-id", &[
    ///         ("language".into(), "en".into())
    ///     ].into_iter().collect())
    ///     .unwrap_or_default()
    ///     .unwrap_or(json!({}));
    /// # }
    /// ```
    pub fn get_json_assignment(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
    ) -> Result<Option<serde_json::Value>, EvaluationError> {
        self.get_assignment_inner(
            flag_key,
            subject_key,
            subject_attributes,
            Some(VariationType::Json),
            |x| {
                x.to_json()
                    // The unwrap cannot fail because the type is checked during evaluation.
                    .unwrap()
            },
        )
    }

    fn get_assignment_inner<T>(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
        expected_type: Option<VariationType>,
        convert: impl FnOnce(AssignmentValue) -> T,
    ) -> Result<Option<T>, EvaluationError> {
        let assignment = self.evaluator.get_assignment(
            flag_key,
            subject_key,
            subject_attributes,
            expected_type,
        )?;

        let Some(Assignment { value, event }) = assignment else {
            return Ok(None);
        };

        if let Some(event) = event {
            log::trace!(target: "eppo",
                        event:serde;
                        "logging assignment");
            self.config.assignment_logger.log_assignment(event);
        }

        Ok(Some(convert(value)))
    }

    /// Get the assignment value for a given feature flag and subject, along with details of why
    /// this value was selected.
    ///
    /// *NOTE:* It is a debug function and is slower due to the need to collect all the
    /// details. Prefer using [`Client::get_assignment()`] or another detail-less function in
    /// production.
    ///
    /// # Typed versions
    ///
    /// There are typed versions of this function:
    /// - [`Client::get_string_assignment_details()`]
    /// - [`Client::get_integer_assignment_details()`]
    /// - [`Client::get_numeric_assignment_details()`]
    /// - [`Client::get_boolean_assignment_details()`]
    /// - [`Client::get_json_assignment_details()`]
    ///
    /// It is recommended to use typed versions of this function as they provide additional type
    /// safety. They can catch type errors even _before_ evaluating the assignment, which helps to
    /// detect errors if subject is not eligible for the flag allocation.
    pub fn get_assignment_details(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
    ) -> EvaluationResultWithDetails<AssignmentValue> {
        self.get_assignment_details_inner(flag_key, subject_key, subject_attributes, None)
    }

    /// Get the assignment value for a given feature flag and subject, along with details of why
    /// this value was selected.
    ///
    /// *NOTE:* It is a debug function and is slower due to the need to collect all the
    /// details. Prefer using [`Client::get_string_assignment()`] in production.
    pub fn get_string_assignment_details(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
    ) -> EvaluationResultWithDetails<String> {
        self.get_assignment_details_inner(
            flag_key,
            subject_key,
            subject_attributes,
            Some(VariationType::String),
        )
        .map(|it| {
            it.to_string()
                .expect("the type should have been checked during evaluation")
        })
    }

    /// Get the assignment value for a given feature flag and subject, along with details of why
    /// this value was selected.
    ///
    /// *NOTE:* It is a debug function and is slower due to the need to collect all the
    /// details. Prefer using [`Client::get_integer_assignment()`] in production.
    pub fn get_integer_assignment_details(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
    ) -> EvaluationResultWithDetails<i64> {
        self.get_assignment_details_inner(
            flag_key,
            subject_key,
            subject_attributes,
            Some(VariationType::Integer),
        )
        .map(|it| {
            it.as_integer()
                .expect("the type should have been checked during evaluation")
        })
    }

    /// Get the assignment value for a given feature flag and subject, along with details of why
    /// this value was selected.
    ///
    /// *NOTE:* It is a debug function and is slower due to the need to collect all the
    /// details. Prefer using [`Client::get_numeric_assignment()`] in production.
    pub fn get_numeric_assignment_details(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
    ) -> EvaluationResultWithDetails<f64> {
        self.get_assignment_details_inner(
            flag_key,
            subject_key,
            subject_attributes,
            Some(VariationType::Numeric),
        )
        .map(|it| {
            it.as_numeric()
                .expect("the type should have been checked during evaluation")
        })
    }

    /// Get the assignment value for a given feature flag and subject, along with details of why
    /// this value was selected.
    ///
    /// *NOTE:* It is a debug function and is slower due to the need to collect all the
    /// details. Prefer using [`Client::get_boolean_assignment()`] in production.
    pub fn get_boolean_assignment_details(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
    ) -> EvaluationResultWithDetails<bool> {
        self.get_assignment_details_inner(
            flag_key,
            subject_key,
            subject_attributes,
            Some(VariationType::Boolean),
        )
        .map(|it| {
            it.as_boolean()
                .expect("the type should have been checked during evaluation")
        })
    }

    /// Get the assignment value for a given feature flag and subject, along with details of why
    /// this value was selected.
    ///
    /// *NOTE:* It is a debug function and is slower due to the need to collect all the
    /// details. Prefer using [`Client::get_json_assignment()`] in production.
    pub fn get_json_assignment_details(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
    ) -> EvaluationResultWithDetails<serde_json::Value> {
        self.get_assignment_details_inner(
            flag_key,
            subject_key,
            subject_attributes,
            Some(VariationType::Json),
        )
        .map(|it| {
            it.to_json()
                .expect("the type should have been checked during evaluation")
        })
    }

    fn get_assignment_details_inner(
        &self,
        flag_key: &str,
        subject_key: &str,
        subject_attributes: &Attributes,
        expected_type: Option<VariationType>,
    ) -> EvaluationResultWithDetails<AssignmentValue> {
        let (result, event) = self.evaluator.get_assignment_details(
            flag_key,
            subject_key,
            subject_attributes,
            expected_type,
        );

        if let Some(event) = event {
            log::trace!(target: "eppo",
                        event:serde;
                        "logging assignment");
            self.config.assignment_logger.log_assignment(event);
        }

        result
    }

    /// Start a poller thread to fetch configuration from the server.
    pub fn start_poller_thread(&mut self) -> Result<PollerThread, Error> {
        PollerThread::start(PollerThreadConfig {
            store: self.configuration_store.clone(),
            base_url: self.config.base_url.clone(),
            api_key: self.config.api_key.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use crate::{client::AssignmentValue, Client, ClientConfig};
    use eppo_core::{
        configuration_store::ConfigurationStore,
        ufc::{
            Allocation, Environment, Flag, Split, TryParse, UniversalFlagConfig, Variation,
            VariationType,
        },
        Configuration,
    };

    #[test]
    fn returns_none_while_no_configuration() {
        let configuration_store = Arc::new(ConfigurationStore::new());
        let client = Client::new_with_configuration_store(
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
        let client = Client::new_with_configuration_store(
            ClientConfig::from_api_key("api-key"),
            configuration_store.clone(),
        );

        // updating configuration after client is created
        configuration_store.set_configuration(Arc::new(Configuration::from_server_response(
            UniversalFlagConfig {
                created_at: chrono::Utc::now(),
                environment: Environment {
                    name: "test".into(),
                },
                flags: [(
                    "flag".to_owned(),
                    TryParse::Parsed(Flag {
                        key: "flag".into(),
                        enabled: true,
                        variation_type: VariationType::Boolean,
                        variations: [(
                            "variation".to_owned(),
                            Variation {
                                key: "variation".into(),
                                value: true.into(),
                            },
                        )]
                        .into(),
                        allocations: vec![Allocation {
                            key: "allocation".into(),
                            rules: vec![],
                            start_at: None,
                            end_at: None,
                            splits: vec![Split {
                                shards: vec![],
                                variation_key: "variation".to_owned(),
                                extra_logging: Arc::new(HashMap::new()),
                            }],
                            do_log: false,
                        }],
                        total_shards: 10_000,
                    }),
                )]
                .into(),
                bandits: HashMap::new(),
            },
            None,
        )));

        assert_eq!(
            client
                .get_assignment("flag", "subject", &HashMap::new())
                .unwrap(),
            Some(AssignmentValue::Boolean(true))
        );
    }
}
