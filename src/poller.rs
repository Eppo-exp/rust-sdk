use std::{
    sync::{mpsc::RecvTimeoutError, Arc, Condvar, Mutex},
    time::Duration,
};

use rand::{thread_rng, Rng};
use reqwest::{StatusCode, Url};

use crate::{configuration_store::ConfigurationStore, Error, Result};

pub(crate) struct PollerThreadConfig {
    pub store: Arc<ConfigurationStore>,
    pub base_url: String,
    pub api_key: String,
}

/// A configuration poller thread.
///
/// Use [`Client.start_poller_thread`] to get an instance of it.
pub struct PollerThread {
    join_handle: std::thread::JoinHandle<()>,

    /// Used to send a stop command to the poller thread.
    stop_sender: std::sync::mpsc::Sender<()>,

    /// Holds `None` if configuration hasn't been fetched yet. Holds `Some(Ok(()))` if configuration
    /// has been fetches successfully. Holds `Some(Err(...))` if there was an error fetching the
    /// first configuration.
    result: Arc<(Mutex<Option<Result<()>>>, Condvar)>,
}

const UFC_ENDPOINT: &'static str = "/flag-config/v1/config";

const POLL_INTERVAL: Duration = Duration::from_secs(5 * 60);
const POLL_JITTER: Duration = Duration::from_secs(30);

impl PollerThread {
    pub(crate) fn start(config: PollerThreadConfig) -> Result<PollerThread> {
        let (stop_sender, stop_receiver) = std::sync::mpsc::channel::<()>();

        let result = Arc::new((Mutex::new(None), Condvar::new()));

        let join_handle = {
            // Cloning Arc for move into thread
            let result = Arc::clone(&result);
            let update_result = move |value| {
                *result.0.lock().unwrap() = Some(value);
                result.1.notify_all();
            };

            let client = reqwest::blocking::Client::new();
            let url = Url::parse_with_params(
                &format!("{}{}", config.base_url, UFC_ENDPOINT),
                &[
                    ("apiKey", &*config.api_key),
                    ("sdkName", "rust"),
                    ("sdkVersion", env!("CARGO_PKG_VERSION")),
                ],
            )
            .map_err(|err| Error::InvalidBaseUrl(err))?;

            std::thread::Builder::new()
                .name("eppo-poller".to_owned())
                .spawn(move || {
                    loop {
                        log::debug!(target: "eppo", "fetching new configuration");
                        match client.get(url.clone()).send() {
                            Ok(response) => {
                                match response.status() {
                                    StatusCode::OK => {
                                        match response.json() {
                                            Ok(ufc) => {
                                                log::debug!(target: "eppo", "sucessfully fetched configuration");
                                                config.store.set_configuration(ufc);
                                                update_result(Ok(()));
                                            }
                                            Err(err) => {
                                                log::warn!(target: "eppo", "failed to parse configuration response body: {:?}", err);
                                            }
                                        }
                                    }
                                    StatusCode::UNAUTHORIZED => {
                                        log::warn!(target: "eppo", "client is not authorized. Check your API key");
                                        update_result(Err(Error::Unauthorized));
                                        // Anauthorized means that API key is not valid and thus is not
                                        // recoverable. Stop the poller thread.
                                        return;
                                    }
                                    code => {
                                        // Ignore other errors, we'll try another request later.
                                        log::warn!(target: "eppo", "received non-200 response while fetching new configuration: {:?}", code);
                                    }
                                }

                            },
                            Err(err) => {
                                log::warn!(target: "eppo", "error while fetching new configuration: {:?}", err)
                            },
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

    /// Block waiting for the first configuration to get fetched.
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
        // useful we can do.
        let _ = self.stop_sender.send(());
    }

    /// Stop the poller thread and block waiting for it to exit.
    ///
    /// If you don't need to wait for the thread to exit, use [`PollerThread.stop`] instead.
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
