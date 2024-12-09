//! A background poller thread that periodically requests configuration from the server and stores
//! it in a configuration store.
use std::{
    sync::{mpsc::RecvTimeoutError, Arc, Condvar, Mutex},
    time::Duration,
};

use rand::{thread_rng, Rng};

use crate::configuration_fetcher::ConfigurationFetcher;
use crate::configuration_store::ConfigurationStore;
use crate::{Error, Result};

/// Configuration for [`PollerThread`].
// Not implementing `Copy` as we may add non-copyable fields in the future.
#[derive(Debug, Clone)]
pub struct PollerThreadConfig {
    /// Interval to wait between requests for configuration.
    ///
    /// Defaults to [`PollerThreadConfig::DEFAULT_POLL_INTERVAL`].
    pub interval: Duration,
    /// Jitter applies a randomized duration to wait between requests for configuration. This helps
    /// to avoid multiple server instances synchronizing and producing spiky network load.
    ///
    /// Defaults to [`PollerThreadConfig::DEFAULT_POLL_JITTER`].
    pub jitter: Duration,
}

impl PollerThreadConfig {
    /// Default value for [`PollerThreadConfig::interval`].
    pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(30);
    /// Default value for [`PollerThreadConfig::jitter`].
    pub const DEFAULT_POLL_JITTER: Duration = Duration::from_secs(3);

    /// Create a new `PollerThreadConfig` using default configuration.
    pub fn new() -> PollerThreadConfig {
        PollerThreadConfig::default()
    }

    /// Update poll interval with `interval`.
    pub fn with_interval(mut self, interval: Duration) -> PollerThreadConfig {
        self.interval = interval;
        self
    }

    /// Update poll interval jitter with `jitter`.
    pub fn with_jitter(mut self, jitter: Duration) -> PollerThreadConfig {
        self.jitter = jitter;
        self
    }
}

impl Default for PollerThreadConfig {
    fn default() -> PollerThreadConfig {
        PollerThreadConfig {
            interval: PollerThreadConfig::DEFAULT_POLL_INTERVAL,
            jitter: PollerThreadConfig::DEFAULT_POLL_JITTER,
        }
    }
}

/// A configuration poller thread.
///
/// The poller thread polls the server periodically to fetch the latest configuration using
/// [`ConfigurationFetcher`] and stores it in [`ConfigurationStore`].
pub struct PollerThread {
    join_handle: std::thread::JoinHandle<()>,

    /// Used to send a stop command to the poller thread.
    // TODO: take a look at `std::thread::park_timeout()`. It could be used to build simpler
    // synchronization.
    stop_sender: std::sync::mpsc::SyncSender<()>,

    /// Holds `None` if configuration hasn't been fetched yet. Holds `Some(Ok(()))` if configuration
    /// has been fetches successfully. Holds `Some(Err(...))` if there was an error fetching the
    /// first configuration.
    result: Arc<(Mutex<Option<Result<()>>>, Condvar)>,
}

impl PollerThread {
    /// Starts the configuration poller thread.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with the `PollerThread` instance if successful, or an `Error` if an issue
    /// occurs.
    ///
    /// # Errors
    ///
    /// This method can return the following errors:
    /// - IO Error if poller thread failed to start.
    pub fn start(
        fetcher: ConfigurationFetcher,
        store: Arc<ConfigurationStore>,
    ) -> std::io::Result<PollerThread> {
        PollerThread::start_with_config(fetcher, store, PollerThreadConfig::default())
    }

    /// Starts the configuration poller thread with the provided configuration.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with the `PollerThread` instance if successful, or an `Error` if an issue
    /// occurs.
    ///
    /// # Errors
    ///
    /// This method can return the following errors:
    /// - IO Error if poller thread failed to start.
    pub fn start_with_config(
        mut fetcher: ConfigurationFetcher,
        store: Arc<ConfigurationStore>,
        config: PollerThreadConfig,
    ) -> std::io::Result<PollerThread> {
        // Using `sync_channel` here as it makes `stop_sender` `Sync` (shareable between
        // threads). Buffer size of 1 should be enough for our use case as we're sending a stop
        // command, and we can simply `try_send()` and ignore if the buffer is full (another thread
        // has sent a stop command already).
        let (stop_sender, stop_receiver) = std::sync::mpsc::sync_channel::<()>(1);

        let result = Arc::new((Mutex::new(None), Condvar::new()));

        let join_handle = {
            // Cloning Arc for move into thread
            let result = Arc::clone(&result);
            let update_result = move |value| {
                *result.0.lock().unwrap() = Some(value);
                result.1.notify_all();
            };

            std::thread::Builder::new()
                .name("eppo-poller".to_owned())
                .spawn(move || {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let runtime = match tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            Ok(runtime) => runtime,
                            Err(err) => {
                                update_result(Err(Error::from(err)));
                                return;
                            }
                        };

                        loop {
                            log::debug!(target: "eppo", "fetching new configuration");
                            let result = runtime.block_on(fetcher.fetch_configuration());
                            match result {
                                Ok(configuration) => {
                                    store.set_configuration(Arc::new(configuration));
                                    update_result(Ok(()))
                                }
                                Err(err @ (Error::Unauthorized | Error::InvalidBaseUrl(_))) => {
                                    // Unrecoverable errors
                                    update_result(Err(err));
                                    return;
                                }
                                _ => {
                                    // Other errors are retrievable.
                                }
                            };

                            let timeout = jitter(config.interval, config.jitter);
                            match stop_receiver.recv_timeout(timeout) {
                                Err(RecvTimeoutError::Timeout) => {
                                    // Timed out. Loop back to fetch a new configuration.
                                }
                                Ok(()) => {
                                    log::debug!(target: "eppo", "poller thread received stop command");
                                    // Stop command received, break out of the loop to end the thread.
                                    return;
                                }
                                Err(RecvTimeoutError::Disconnected) => {
                                    // When the other end of channel disconnects, calls to
                                    // .recv_timeout() return immediately.
                                    // Stop the thread.
                                    log::debug!(target: "eppo", "poller thread received disconnected");
                                    return;
                                }
                            }
                        }
                    }));

                    // If catch_unwind returns Err, it means a panic occurred.
                    if let Err(_panic_info) = result {
                        // Handle the panic gracefully by updating the result with an error.
                        update_result(Err(Error::PollerThreadPanicked));
                    }
                })?
        };

        Ok(PollerThread {
            join_handle,
            stop_sender,
            result,
        })
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
    /// - [`Error::PollerThreadPanicked`]
    /// - [`Error::Unauthorized`]
    /// - [`Error::InvalidBaseUrl`]
    ///
    /// # Example
    ///
    /// ```
    /// # fn test(mut poller_thread: eppo_core::poller_thread::PollerThread) {
    /// match poller_thread.wait_for_configuration() {
    ///     Ok(()) => println!("Configuration fetched successfully."),
    ///     Err(err) => eprintln!("Cannot fetch configuration: {:?}", err),
    /// }
    /// # }
    /// ```
    pub fn wait_for_configuration(&self) -> Result<()> {
        let mut lock = self
            .result
            .0
            .lock()
            .map_err(|_| Error::PollerThreadPanicked)?;
        loop {
            match &*lock {
                Some(result) => {
                    // The poller has already fetched the configuration. Return Ok(()) or a possible
                    // error.
                    return result.clone();
                }
                None => {
                    // Block waiting for configuration to get fetched.
                    lock = self
                        .result
                        .1
                        .wait(lock)
                        .map_err(|_| Error::PollerThreadPanicked)?;
                }
            }
        }
    }

    /// Stop the poller thread.
    ///
    /// This function does not wait for the thread to actually stop.
    pub fn stop(&self) {
        // Error means that the receiver was dropped (thread exited) or the channel buffer is
        // full. First case can be ignored it as there's nothing useful we can do—thread is already
        // stopped. Second case can be ignored as it indicates that another thread already sent a
        // stop command and the thread will stop anyway.
        let _ = self.stop_sender.try_send(());
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
    /// # fn test(poller_thread: eppo_core::poller_thread::PollerThread) {
    /// poller_thread.shutdown().expect("Failed to shut down the poller thread");
    /// # }
    /// ```
    pub fn shutdown(self) -> Result<()> {
        // Send stop signal in case it wasn't sent before.
        self.stop();

        // Error means that the thread has panicked and there's nothing useful we can do in that
        // case.
        self.join_handle
            .join()
            .map_err(|_| Error::PollerThreadPanicked)?;

        Ok(())
    }
}

/// Apply randomized `jitter` to `interval`.
fn jitter(interval: Duration, jitter: Duration) -> Duration {
    Duration::saturating_sub(interval, thread_rng().gen_range(Duration::ZERO..=jitter))
}

#[cfg(test)]
mod jitter_tests {
    use std::time::Duration;

    #[test]
    fn jitter_is_subtractive() {
        let interval = Duration::from_secs(30);
        let jitter = Duration::from_secs(30);

        let result = super::jitter(interval, jitter);

        assert!(result <= interval, "{result:?} must be <= {interval:?}");
    }

    #[test]
    fn jitter_truncates_to_zero() {
        let interval = Duration::ZERO;
        let jitter = Duration::from_secs(30);

        let result = super::jitter(interval, jitter);

        assert_eq!(result, Duration::ZERO);
    }

    #[test]
    fn jitter_works_with_zero_jitter() {
        let interval = Duration::from_secs(30);
        let jitter = Duration::ZERO;

        let result = super::jitter(interval, jitter);

        assert_eq!(result, Duration::from_secs(30));
    }
}
