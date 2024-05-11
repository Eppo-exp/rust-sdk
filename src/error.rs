use std::sync::Arc;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("flag not found")]
    FlagNotFound,
    #[error("error parsing configuration, try upgrading Eppo SDK")]
    ConfigurationParseError,
    #[error("configuration error")]
    ConfigurationError,
    #[error("invalid base_url configuration")]
    InvalidBaseUrl(#[source] url::ParseError),
    #[error("unauthorized, api_key is likely invalid")]
    Unauthorized,
    // std::io::Error is not clonable, so we're wrapping it in an Arc.
    #[error(transparent)]
    Io(Arc<std::io::Error>),
    #[error("poller thread panicked")]
    PollerThreadPanicked,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(Arc::new(value))
    }
}
