//! The Rust SDK for Eppo, a next-generation feature flagging and experimentation platform.
//!
//! # Overview
//!
//! The SDK revolves around a [`Client`] that evaluates feature flag values for "subjects", where each
//! subject has a unique key and key-value attributes associated with it. Feature flag evaluation
//! results in an [`AssignmentValue`] being returned, representing a specific feature flag value assigned
//! to the subject.
//!
//! # Typed assignments
//!
//! Every Eppo flag has a return type that is set once on creation in the dashboard. Once a flag is
//! created, assignments in code should be made using the corresponding typed function:
//! - [`Client::get_string_assignment()`]
//! - [`Client::get_integer_assignment()`]
//! - [`Client::get_numeric_assignment()`]
//! - [`Client::get_boolean_assignment()`]
//! - [`Client::get_json_assignment()`]
//!
//! These functions provide additional type safety over [`Client::get_assignment()`] as they can
//! detect type mismatch even before evaluating the feature, so the error is returned even if
//! subject is otherwise uneligible (`get_assignment()` return `Ok(None)` in that case).
//!
//! # Assignment logger
//!
//! An [`AssignmentLogger`] should be provided to save assignment events to your storage,
//! facilitating tracking of which user received which feature flag values.
//!
//! ```
//! # use eppo::ClientConfig;
//! let config = ClientConfig::from_api_key("api-key").assignment_logger(|assignment| {
//!   println!("{:?}", assignment);
//! });
//! ```
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
mod poller;

#[doc(inline)]
pub use eppo_core::{
    ufc::{eval_details::*, AssignmentEvent, AssignmentValue},
    AttributeValue, Attributes, Error, EvaluationError, Result,
};

pub use assignment_logger::AssignmentLogger;
pub use client::Client;
pub use config::ClientConfig;
pub use poller::PollerThread;
