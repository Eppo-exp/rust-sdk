//! An HTTP client that fetches configuration from the server.
use std::sync::Arc;

use reqwest::{StatusCode, Url};

use crate::{ufc::UniversalFlagConfig, Configuration, Error, Result};

pub struct ConfigurationFetcherConfig {
    pub base_url: String,
    pub api_key: String,
    /// SDK name. Usually, language name.
    pub sdk_name: String,
    /// Version of SDK.
    pub sdk_version: String,
}

pub const DEFAULT_BASE_URL: &'static str = "https://fscdn.eppo.cloud/api";

const UFC_ENDPOINT: &'static str = "/flag-config/v1/config";

/// A client that fetches Eppo configuration from the server.
pub struct ConfigurationFetcher {
    // Client holds a connection pool internally, so we're reusing the client between requests.
    client: reqwest::blocking::Client,
    config: ConfigurationFetcherConfig,
    /// If we receive a 401 Unauthorized error during a request, it means the API key is not
    /// valid. We cache this error so we don't issue additional requests to the server.
    unauthorized: bool,
}

impl ConfigurationFetcher {
    pub fn new(config: ConfigurationFetcherConfig) -> ConfigurationFetcher {
        let client = reqwest::blocking::Client::new();

        ConfigurationFetcher {
            client,
            config,
            unauthorized: false,
        }
    }

    pub fn fetch_configuration(&mut self) -> Result<Configuration> {
        if self.unauthorized {
            return Err(Error::Unauthorized);
        }

        let ufc = self.fetch_ufc_configuration()?;

        Ok(Configuration {
            ufc: Some(Arc::new(ufc)),
        })
    }

    fn fetch_ufc_configuration(&mut self) -> Result<UniversalFlagConfig> {
        let url = Url::parse_with_params(
            &format!("{}{}", self.config.base_url, UFC_ENDPOINT),
            &[
                ("apiKey", &*self.config.api_key),
                ("sdkName", &*self.config.sdk_name),
                ("sdkVersion", &*self.config.sdk_version),
                ("coreVersion", env!("CARGO_PKG_VERSION")),
            ],
        )
        .map_err(|err| Error::InvalidBaseUrl(err))?;

        log::debug!(target: "eppo", "fetching UFC configuration");
        let response = self.client.get(url).send()?;

        let response = response.error_for_status().map_err(|err| {
            if err.status() == Some(StatusCode::UNAUTHORIZED) {
                    log::warn!(target: "eppo", "client is not authorized. Check your API key");
                    self.unauthorized = true;
                    return Error::Unauthorized;
                } else {
                    log::warn!(target: "eppo", "received non-200 response while fetching new configuration: {:?}", err);
                    return Error::from(err);

            }
        })?;

        let configuration = response.json()?;

        log::debug!(target: "eppo", "successfully fetched UFC configuration");

        Ok(configuration)
    }
}
