use serde::{Deserialize, Serialize};

use crate::ufc::VariationType;

/// Enum representing possible errors that can occur during flag evaluation.
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FlagEvaluationError {
    /// Configuration has not been fetched yet.
    #[error("configuration missing")]
    ConfigurationMissing,

    /// The requested flag configuration was not found. It either does not exist or is disabled.
    #[error("flag not found")]
    FlagNotFound,

    /// Flag is found in configuration but it is disabled.
    #[error("flag is disabled")]
    FlagDisabled,

    /// No allocation found. This causes the return of default allocation.
    #[error("no allocation assigned")]
    NoAllocation,

    /// Requested flag has invalid type.
    #[error("invalid flag type (expected: {expected:?}, found: {found:?})")]
    InvalidType {
        /// Expected type of the flag.
        expected: VariationType,
        /// Actual type of the flag.
        found: VariationType,
    },

    /// An error occurred while parsing the configuration (server sent unexpected response). It is
    /// recommended to upgrade the Eppo SDK.
    #[error("error parsing configuration, try upgrading Eppo SDK")]
    ConfigurationParseError,

    /// Configuration received from the server is invalid for the SDK. This should normally never
    /// happen and is likely a signal that you should update SDK.
    #[error("configuration error, try upgrading Eppo SDK")]
    ConfigurationError,
}

impl FlagEvaluationError {
    /// Return `true` if the error is a normal running condition and the default value should be
    /// returned silently.
    pub(super) fn is_normal(self) -> bool {
        match self {
            FlagEvaluationError::ConfigurationMissing
            | FlagEvaluationError::FlagNotFound
            | FlagEvaluationError::FlagDisabled
            | FlagEvaluationError::NoAllocation => true,

            FlagEvaluationError::InvalidType { .. }
            | FlagEvaluationError::ConfigurationParseError
            | FlagEvaluationError::ConfigurationError => false,
        }
    }
}
