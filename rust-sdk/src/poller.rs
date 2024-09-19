use std::sync::Arc;

use crate::{Result, SDK_METADATA};
use eppo_core::configuration_fetcher::{ConfigurationFetcher, ConfigurationFetcherConfig};
use eppo_core::configuration_store::ConfigurationStore;
use eppo_core::poller_thread::PollerThread as PollerThreadImpl;
#[cfg(doc)]
use eppo_core::Error;

pub(crate) struct PollerThreadConfig {
    pub(crate) store: Arc<ConfigurationStore>,
    pub(crate) base_url: String,
    pub(crate) api_key: String,
}

/// A configuration poller thread.
///
/// The poller thread polls the server periodically to fetch the latest configuration.
///
/// Use [`Client::start_poller_thread`][crate::Client::start_poller_thread] to get an instance.
///
/// The Client returns `None` for assignments before the first configuration is fetched. So it is
/// recommended to call [`PollerThread::wait_for_configuration`] before requesting assignments.
pub struct PollerThread(PollerThreadImpl);

impl PollerThread {
    /// Starts the configuration poller thread.
    ///
    /// # Arguments
    ///
    /// * `config` - A [`PollerThreadConfig`] containing configuration details.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with the `PollerThread` instance if successful, or an `Error` if an issue occurs.
    ///
    /// # Errors
    ///
    /// This method can return the following errors:
    ///
    /// - [`Error::InvalidBaseUrl`] if the base URL configuration is invalid.
    /// - [`Error::Unauthorized`] if the request is unauthorized, possibly due to an invalid API key.
    /// - [`Error::PollerThreadPanicked`] if an unexpected panic occurs in the poller thread.
    /// - [`Error::Io`] for any I/O related errors.
    pub(crate) fn start(config: PollerThreadConfig) -> Result<PollerThread> {
        let fetcher = ConfigurationFetcher::new(ConfigurationFetcherConfig {
            base_url: config.base_url,
            api_key: config.api_key,
            sdk_metadata: SDK_METADATA.clone(),
        });
        let inner = PollerThreadImpl::start(fetcher, config.store)?;
        Ok(PollerThread(inner))
    }

    /// Waits for the configuration to be fetched.
    ///
    /// This method blocks until the poller thread has fetched the configuration.
    ///
    /// # Returns
    ///
    /// Returns `Result<()>` where `Ok(())` indicates successful configuration fetch and any
    /// error that occurred during the process.
    ///
    /// # Errors
    ///
    /// This method can fail with the following errors:
    ///
    /// - [`Error::PollerThreadPanicked`]: If the poller thread panicked while waiting for
    /// configuration.
    ///
    /// # Example
    ///
    /// ```
    /// # fn test(mut client: eppo::Client) {
    /// let poller = client.start_poller_thread().unwrap();
    /// match poller.wait_for_configuration() {
    ///     Ok(()) => println!("Configuration fetched successfully."),
    ///     Err(err) => eprintln!("Error fetching configuration: {:?}", err),
    /// }
    /// # }
    /// ```
    pub fn wait_for_configuration(&self) -> Result<()> {
        self.0.wait_for_configuration()
    }

    /// Stop the poller thread.
    ///
    /// This function does not wait for the thread to actually stop.
    pub fn stop(&self) {
        self.0.stop()
    }

    /// Stop the poller thread and block waiting for it to exit.
    ///
    /// If you don't need to wait for the thread to exit, use [`PollerThread::stop`] instead.
    ///
    /// # Errors
    ///
    /// Returns an error of type [`Error`] in the following cases:
    ///
    /// - [`Error::PollerThreadPanicked`] if the thread has panicked.
    ///
    /// # Examples
    ///
    /// ```
    /// # use eppo::PollerThread;
    /// # fn test(poller_thread: eppo::PollerThread) {
    /// poller_thread.shutdown().expect("Failed to shut down the poller thread");
    /// # }
    /// ```
    pub fn shutdown(self) -> Result<()> {
        self.0.shutdown()
    }
}
