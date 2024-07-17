use std::sync::Arc;

use crate::ufc::FlagEvaluationError;

/// Represents a result type for operations in the Eppo SDK.
///
/// This type alias is used throughout the SDK to indicate the result of operations that may return
/// errors specific to the Eppo SDK.
///
/// This `Result` type is a standard Rust `Result` type where the error variant is defined by the
/// eppo-specific [`Error`] enum.
pub type Result<T> = std::result::Result<T, Error>;

/// Enum representing possible errors that can occur in the Eppo SDK.
#[derive(thiserror::Error, Debug, Clone)]
#[non_exhaustive]
pub enum Error {
    /// Error evaluating a flag.
    #[error(transparent)]
    FlagEvaluationError(FlagEvaluationError),

    /// Invalid base URL configuration.
    #[error("invalid base_url configuration")]
    InvalidBaseUrl(#[source] url::ParseError),

    /// The request was unauthorized, possibly due to an invalid API key.
    #[error("unauthorized, api_key is likely invalid")]
    Unauthorized,

    /// Indicates that the poller thread panicked. This should normally never happen.
    #[error("poller thread panicked")]
    PollerThreadPanicked,

    /// An I/O error.
    #[error(transparent)]
    // std::io::Error is not clonable, so we're wrapping it in an Arc.
    Io(Arc<std::io::Error>),

    /// Network error.
    #[error(transparent)]
    Network(Arc<reqwest::Error>),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(Arc::new(value))
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Network(Arc::new(value.without_url()))
    }
}
