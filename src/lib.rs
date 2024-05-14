//! The Rust SDK for Eppo, a next-generation feature flagging and experimentation platform.
//!
//! # Overview
//!
//! The SDK revolves around a [`Client`] that evaluates feature flag values for `subjects`, where each
//! subject has a unique key and key-value attributes associated with it. Feature flag evaluation
//! results in an [`AssignmentValue`] being returned, representing a specific feature flag value assigned
//! to the subject.
//!
//! An [`AssignmentLogger`] should be provided to save assignment events to your storage,
//! facilitating tracking of which user received which feature flag values.
//!
//! # Error Handling
//!
//! Errors are represented by the [`Error`] enum.
//!
//! In production, it is recommended to ignore all errors, as feature flag evaluation should not be
//! critical enough to cause system crashes. However, the returned errors are valuable for debugging
//! and usually indicate that developer's attention is needed.
//!
//! # Logging
//!
//! The package uses the [`log`](https://docs.rs/log/latest/log/) crate for logging
//! messages. Consider integrating a `log`-compatible logger implementation for better visibility
//! into SDK operations.
//!
//! # Examples
//!
//! Examples can be found in the [examples directory](https://github.com/eppo-exp/rust-sdk/examples)
//! of the `eppo` crate repository.

#![warn(rustdoc::missing_crate_level_docs)]
#![warn(missing_docs)]

mod assignment_logger;
mod client;
mod config;
mod configuration_store;
mod error;
mod eval;
mod poller;
mod rules;
mod sharder;
mod ufc;

pub use assignment_logger::{AssignmentEvent, AssignmentLogger};
pub use client::{AssignmentValue, AttributeValue, Client, SubjectAttributes};
pub use config::ClientConfig;
pub use error::{Error, Result};
pub use poller::PollerThread;
