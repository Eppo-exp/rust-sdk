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

/// A configuration poller thread.
///
/// The poller thread polls the server periodically to fetch the latest configuration using
/// [`ConfigurationFetcher`] and stores it in [`ConfigurationStore`].
pub struct PollerThread {
    join_handle: std::thread::JoinHandle<()>,

    /// Used to send a stop command to the poller thread.
    stop_sender: std::sync::mpsc::Sender<()>,

    /// Holds `None` if configuration hasn't been fetched yet. Holds `Some(Ok(()))` if configuration
    /// has been fetches successfully. Holds `Some(Err(...))` if there was an error fetching the
    /// first configuration.
    result: Arc<(Mutex<Option<Result<()>>>, Condvar)>,
}

const POLL_INTERVAL: Duration = Duration::from_secs(5 * 60);
const POLL_JITTER: Duration = Duration::from_secs(30);

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
        mut fetcher: ConfigurationFetcher,
        store: Arc<ConfigurationStore>,
    ) -> std::io::Result<PollerThread> {
        let (stop_sender, stop_receiver) = std::sync::mpsc::channel::<()>();

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
                    loop {
                        log::debug!(target: "eppo", "fetching new configuration");
                        let result = fetcher.fetch_configuration();
                        match result {
                            Ok(configuration) => {
                                store.set_configuration(configuration);
                                update_result(Ok(()))
                            }
                            Err(err @ (Error::Unauthorized | Error::InvalidBaseUrl(_))) => {
                                // Unrecoverable errors
                                update_result(Err(err));
                                return;
                            }
                            _ => {
                                // Other errors are retriable.
                            }
                        };

                        let timeout = jitter(POLL_INTERVAL, POLL_JITTER);
                        match stop_receiver.recv_timeout(timeout) {
                            Err(RecvTimeoutError::Timeout) => {
                                // Timed out. Loop to fetch new configuration.
                            }
                            Ok(()) => {
                                log::debug!(target: "eppo", "poller thread received stop command");
                                // The other end asked us to stop the poller thread.
                                return;
                            }
                            Err(RecvTimeoutError::Disconnected) => {
                                // When the other end of channel disconnects, calls to
                                // .recv_timeout() return immediately. Use normal thread sleep in
                                // this case.
                                std::thread::sleep(timeout);
                            }
                        }
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
        // Error means that the receiver was dropped (thread exited). Ignoring it as there's nothing
        // useful we can do—thread is already stopped.
        let _ = self.stop_sender.send(());
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

/// Apply a random jitter to `interval`.
fn jitter(interval: Duration, jitter: Duration) -> Duration {
    interval + thread_rng().gen_range(Duration::ZERO..jitter)
}
