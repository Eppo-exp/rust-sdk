use std::{collections::HashMap, sync::Arc};

use derive_more::From;
use serde::{Deserialize, Serialize};

#[cfg(doc)]
use crate::Error;
use crate::{
    configuration_store::ConfigurationStore,
    poller::{PollerThread, PollerThreadConfig},
    sharder::Md5Sharder,
    ufc::VariationType,
    ClientConfig, Result,
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
    configuration_store: Arc<ConfigurationStore>,
    config: ClientConfig<'a>,
}

impl<'a> Client<'a> {
    /// Create a new `Client` using the specified configuration.
    ///
    /// ```
    /// # use eppo::{ClientConfig, Client};
    /// let client = Client::new(ClientConfig::from_api_key("api-key"));
    /// ```
    pub fn new(config: ClientConfig<'a>) -> Self {
        Client {
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
    /// # Errors
    ///
    /// Returns an error in the following cases:
    /// - [`Error::FlagNotFound`] if the requested flag configuration was not found.
    /// - [`Error::ConfigurationParseError`] or [`Error::ConfigurationError`] if the configuration
    /// received from the server is invalid.
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
        subject_attributes: &SubjectAttributes,
    ) -> Result<Option<AssignmentValue>> {
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
    /// # Errors
    ///
    /// Returns an error in the following cases:
    /// - [`Error::FlagNotFound`] if the requested flag configuration was not found.
    /// - [`Error::InvalidType`] if the requested flag has an invalid type or type conversion fails.
    /// - [`Error::ConfigurationParseError`] or [`Error::ConfigurationError`] if the configuration
    /// received from the server is invalid.
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
        subject_attributes: &SubjectAttributes,
    ) -> Result<Option<String>> {
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
    /// # Errors
    ///
    /// Returns an error in the following cases:
    /// - [`Error::FlagNotFound`] if the requested flag configuration was not found.
    /// - [`Error::InvalidType`] if the requested flag has an invalid type or type conversion fails.
    /// - [`Error::ConfigurationParseError`] or [`Error::ConfigurationError`] if the configuration
    /// received from the server is invalid.
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
        subject_attributes: &SubjectAttributes,
    ) -> Result<Option<i64>> {
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
    /// # Errors
    ///
    /// Returns an error in the following cases:
    /// - [`Error::FlagNotFound`] if the requested flag configuration was not found.
    /// - [`Error::InvalidType`] if the requested flag has an invalid type or type conversion fails.
    /// - [`Error::ConfigurationParseError`] or [`Error::ConfigurationError`] if the configuration
    /// received from the server is invalid.
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
        subject_attributes: &SubjectAttributes,
    ) -> Result<Option<f64>> {
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
    /// # Errors
    ///
    /// Returns an error in the following cases:
    /// - [`Error::FlagNotFound`] if the requested flag configuration was not found.
    /// - [`Error::InvalidType`] if the requested flag has an invalid type or type conversion fails.
    /// - [`Error::ConfigurationParseError`] or [`Error::ConfigurationError`] if the configuration
    /// received from the server is invalid.
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
        subject_attributes: &SubjectAttributes,
    ) -> Result<Option<bool>> {
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
    /// # Errors
    ///
    /// Returns an error in the following cases:
    /// - [`Error::FlagNotFound`] if the requested flag configuration was not found.
    /// - [`Error::InvalidType`] if the requested flag has an invalid type or type conversion fails.
    /// - [`Error::ConfigurationParseError`] or [`Error::ConfigurationError`] if the configuration
    /// received from the server is invalid.
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
        subject_attributes: &SubjectAttributes,
    ) -> Result<Option<serde_json::Value>> {
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
        subject_attributes: &SubjectAttributes,
        expected_type: Option<VariationType>,
        convert: impl FnOnce(AssignmentValue) -> T,
    ) -> Result<Option<T>> {
        let Some(configuration) = self.configuration_store.get_configuration() else {
            log::warn!(target: "eppo", flag_key, subject_key; "evaluating a flag before Eppo configuration has been fetched");
            // We treat missing configuration (the poller has not fetched config) as a normal
            // scenario (at least for now).
            return Ok(None);
        };

        let evaluation = match configuration.eval_flag(
            flag_key,
            subject_key,
            subject_attributes,
            &Md5Sharder,
            expected_type,
        ) {
            Ok(result) => result,
            Err(err) => {
                log::warn!(target: "eppo",
                           flag_key,
                           subject_key,
                           subject_attributes:serde;
                           "error occurred while evaluating a flag: {:?}", err,
                );
                return Err(err);
            }
        };

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

        Ok(Some(convert(value)))
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

/// Type alias for a HashMap representing key-value pairs of attributes describing a subject.
///
/// Keys are strings representing attribute names.
///
/// # Examples
/// ```
/// # use eppo::{SubjectAttributes, AttributeValue};
/// let attributes = [
///     ("age".to_owned(), 30.0.into()),
///     ("is_premium_member".to_owned(), true.into()),
///     ("username".to_owned(), "john_doe".into()),
/// ].into_iter().collect::<SubjectAttributes>();
/// ```
pub type SubjectAttributes = HashMap<String, AttributeValue>;

/// Enum representing possible values of an attribute for a subject.
///
/// Conveniently implements `From` conversions for `String`, `&str`, `f64`, and `bool` types.
///
/// Examples:
/// ```
/// # use eppo::AttributeValue;
/// let string_attr: AttributeValue = "example".into();
/// let number_attr: AttributeValue = 42.0.into();
/// let bool_attr: AttributeValue = true.into();
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, From, Clone)]
#[serde(untagged)]
pub enum AttributeValue {
    /// A string value.
    String(String),
    /// A numerical value.
    Number(f64),
    /// A boolean value.
    Boolean(bool),
    /// A null value or absence of value.
    Null,
}
impl From<&str> for AttributeValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

/// Enum representing values assigned to a subject as a result of feature flag evaluation.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum AssignmentValue {
    /// A string value.
    String(String),
    /// An integer value.
    Integer(i64),
    /// A numeric value (floating-point).
    Numeric(f64),
    /// A boolean value.
    Boolean(bool),
    /// Arbitrary JSON value.
    Json(serde_json::Value),
}

impl AssignmentValue {
    /// Checks if the assignment value is of type String.
    ///
    /// # Returns
    /// - `true` if the value is of type String, otherwise `false`.
    ///
    /// # Examples
    /// ```
    /// # use eppo::AssignmentValue;
    /// let value = AssignmentValue::String("example".to_owned());
    /// assert_eq!(value.is_string(), true);
    /// ```
    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }
    /// Returns the assignment value as a string if it is of type String.
    ///
    /// # Returns
    /// - The string value if the assignment value is of type String, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// # use eppo::AssignmentValue;
    /// let value = AssignmentValue::String("example".to_owned());
    /// assert_eq!(value.as_str(), Some("example"));
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        match self {
            AssignmentValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Extracts the assignment value as a string if it is of type String.
    ///
    /// # Returns
    /// - The string value if the assignment value is of type String, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// # use eppo::AssignmentValue;
    /// let value = AssignmentValue::String("example".to_owned());
    /// assert_eq!(value.to_string(), Some("example".to_owned()));
    /// ```
    pub fn to_string(self) -> Option<String> {
        match self {
            AssignmentValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Checks if the assignment value is of type Integer.
    ///
    /// # Returns
    /// - `true` if the value is of type Integer, otherwise `false`.
    ///
    /// # Examples
    /// ```
    /// use eppo::AssignmentValue;
    ///
    /// let value = AssignmentValue::Integer(42);
    /// assert_eq!(value.is_integer(), true);
    /// ```
    pub fn is_integer(&self) -> bool {
        self.as_integer().is_some()
    }
    /// Returns the assignment value as an integer if it is of type Integer.
    ///
    /// # Returns
    /// - The integer value if the assignment value is of type Integer, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// use eppo::AssignmentValue;
    ///
    /// let value = AssignmentValue::Integer(42);
    /// assert_eq!(value.as_integer(), Some(42));
    /// ```
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            AssignmentValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Checks if the assignment value is of type Numeric.
    ///
    /// # Returns
    /// - `true` if the value is of type Numeric, otherwise `false`.
    ///
    /// # Examples
    /// ```
    /// use eppo::AssignmentValue;
    ///
    /// let value = AssignmentValue::Numeric(3.14);
    /// assert_eq!(value.is_numeric(), true);
    /// ```
    pub fn is_numeric(&self) -> bool {
        self.as_numeric().is_some()
    }
    /// Returns the assignment value as a numeric value if it is of type Numeric.
    ///
    /// # Returns
    /// - The numeric value if the assignment value is of type Numeric, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// use eppo::AssignmentValue;
    ///
    /// let value = AssignmentValue::Numeric(3.14);
    /// assert_eq!(value.as_numeric(), Some(3.14));
    /// ```
    pub fn as_numeric(&self) -> Option<f64> {
        match self {
            Self::Numeric(n) => Some(*n),
            _ => None,
        }
    }

    /// Checks if the assignment value is of type Boolean.
    ///
    /// # Returns
    /// - `true` if the value is of type Boolean, otherwise `false`.
    ///
    /// # Examples
    /// ```
    /// use eppo::AssignmentValue;
    ///
    /// let value = AssignmentValue::Boolean(true);
    /// assert_eq!(value.is_boolean(), true);
    /// ```
    pub fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }
    /// Returns the assignment value as a boolean if it is of type Boolean.
    ///
    /// # Returns
    /// - The boolean value if the assignment value is of type Boolean, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// use eppo::AssignmentValue;
    ///
    /// let value = AssignmentValue::Boolean(true);
    /// assert_eq!(value.as_boolean(), Some(true));
    /// ```
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            AssignmentValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Checks if the assignment value is of type Json.
    ///
    /// # Returns
    /// - `true` if the value is of type Json, otherwise `false`.
    ///
    /// # Examples
    /// ```
    /// use eppo::AssignmentValue;
    /// use serde_json::json;
    ///
    /// let value = AssignmentValue::Json(json!({ "key": "value" }));
    /// assert_eq!(value.is_json(), true);
    /// ```
    pub fn is_json(&self) -> bool {
        self.as_json().is_some()
    }
    /// Returns the assignment value as a JSON value if it is of type Json.
    ///
    /// # Returns
    /// - The JSON value if the assignment value is of type Json, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// use eppo::AssignmentValue;
    /// use serde_json::json;
    ///
    /// let value = AssignmentValue::Json(json!({ "key": "value" }));
    /// assert_eq!(value.as_json(), Some(&json!({ "key": "value" })));
    /// ```
    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Json(v) => Some(v),
            _ => None,
        }
    }
    /// Extracts the assignment value as a JSON value if it is of type Json.
    ///
    /// # Returns
    /// - The JSON value if the assignment value is of type Json, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// use eppo::AssignmentValue;
    /// use serde_json::json;
    ///
    /// let value = AssignmentValue::Json(json!({ "key": "value" }));
    /// assert_eq!(value.to_json(), Some(json!({ "key": "value" })));
    /// ```
    pub fn to_json(self) -> Option<serde_json::Value> {
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
        Client, ClientConfig,
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
