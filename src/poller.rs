use std::{
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

use rand::{thread_rng, Rng};
use reqwest::{StatusCode, Url};

use crate::configuration_store::ConfigurationStore;

pub(crate) struct PollerThreadConfig {
    pub store: Arc<ConfigurationStore>,
    pub base_url: String,
    pub api_key: String,
}

pub struct PollerThread {
    join_handle: std::thread::JoinHandle<()>,
    stop_sender: std::sync::mpsc::Sender<()>,

    first_configuration_received: Arc<(Mutex<Option<()>>, Condvar)>,
}

const UFC_ENDPOINT: &'static str = "/flag-config/v1/config";

const POLL_INTERVAL: Duration = Duration::from_secs(5 * 60);
const POLL_JITTER: Duration = Duration::from_secs(30);

impl PollerThread {
    pub(crate) fn start(config: PollerThreadConfig) -> PollerThread {
        let client = reqwest::blocking::Client::new();
        let url = Url::parse_with_params(
            &format!("{}{}", config.base_url, UFC_ENDPOINT),
            &[
                ("apiKey", &*config.api_key),
                ("sdkName", "rust"),
                ("sdkVersion", env!("CARGO_PKG_VERSION")),
            ],
        )
        .unwrap();

        let (stop_sender, stop_receiver) = std::sync::mpsc::channel::<()>();

        let first_configuration_received = Arc::new((Mutex::new(None), Condvar::new()));

        let first_configuration = first_configuration_received.clone();
        let join_handle = std::thread::Builder::new()
            .name("eppo-poller".to_owned())
            .spawn(move || {
                let mut has_first_configuration = false;

                loop {
                    if let Ok(response) = client.get(url.clone()).send() {
                        match response.status() {
                            StatusCode::OK => {
                                if let Ok(ufc) = response.json() {
                                    config.store.set_configuration(ufc);

                                    if !has_first_configuration {
                                        has_first_configuration = true;
                                        let mut first_configuration_received =
                                            first_configuration.0.lock().unwrap();
                                        *first_configuration_received = Some(());
                                        first_configuration.1.notify_all();
                                    }
                                }
                            }
                            StatusCode::UNAUTHORIZED => {
                                // Anauthorized means that API key is not valid and thus is not
                                // recoverable. Break and stop the poller thread.
                                break;
                            }
                            _ => {
                                // Ignore other errors, we'll try another request later.
                            }
                        }
                    }

                    if let Ok(()) = stop_receiver.recv_timeout(jitter(POLL_INTERVAL, POLL_JITTER)) {
                        break;
                    }
                }
            })
            .unwrap();

        PollerThread {
            join_handle,
            stop_sender,
            first_configuration_received,
        }
    }

    pub fn wait_for_configuration(&self) {
        let first_configuration_received = self.first_configuration_received.0.lock().unwrap();
        if first_configuration_received.is_none() {
            let _unused = self
                .first_configuration_received
                .1
                .wait_while(first_configuration_received, |x| x.is_none())
                .unwrap();
        }
    }

    /// Stop the poller thread.
    pub fn stop(&self) {
        // Ignoring error as there's nothing useful we can do.
        let _ = self.stop_sender.send(());
    }

    /// Stop the poller thread and wait for it to exit.
    pub fn join(self) {
        // Send stop signal in case it wasn't sent before.
        self.stop();

        self.join_handle.join();
    }
}

/// Apply a random jitter to `interval`.
fn jitter(interval: Duration, jitter: Duration) -> Duration {
    interval + thread_rng().gen_range(Duration::ZERO..jitter)
}
