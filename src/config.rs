use crate::{assignment_logger::NoopAssignmentLogger, AssignmentLogger, EppoClient};

/// Configuration for [`EppoClient`].
pub struct Config<'a> {
    pub(crate) api_key: String,
    pub(crate) base_url: String,
    pub(crate) assignment_logger: Box<dyn AssignmentLogger + Send + Sync + 'a>,
}

impl<'a> Config<'a> {
    /// Create a default Eppo configuration using the specified API key.
    ///
    /// ```
    /// # use eppo::Config;
    /// Config::from_api_key("api-key");
    /// ```
    pub fn from_api_key(api_key: impl Into<String>) -> Self {
        Config {
            api_key: api_key.into(),
            base_url: Config::DEFAULT_BASE_URL.to_owned(),
            assignment_logger: Box::new(NoopAssignmentLogger),
        }
    }

    /// Set assignment logger to pass variation assignments to your data warehouse.
    ///
    /// ```
    /// # use eppo::Config;
    /// let config = Config::from_api_key("api-key").assignment_logger(|event| {
    ///   println!("{:?}", event);
    /// });
    /// ```
    pub fn assignment_logger(
        &mut self,
        assignment_logger: impl AssignmentLogger + Send + Sync + 'a,
    ) -> &mut Self {
        self.assignment_logger = Box::new(assignment_logger);
        self
    }

    /// Default base URL for API calls.
    pub const DEFAULT_BASE_URL: &'static str = "https://fscdn.eppo.cloud/api";

    /// Override base URL for API calls. Clients should use the default setting in most cases.
    pub fn base_url(&mut self, base_url: impl Into<String>) -> &mut Self {
        self.base_url = base_url.into();
        self
    }

    /// Create a new [`EppoClient`] using the specified configuration.
    ///
    /// ```
    /// # use eppo::{Config, EppoClient};
    /// let client: EppoClient = Config::from_api_key("api-key").to_client();
    /// ```
    pub fn to_client(self) -> EppoClient<'a> {
        EppoClient::new(self)
    }
}
