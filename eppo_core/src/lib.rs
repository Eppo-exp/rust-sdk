//! `eppo_core` is a common library to build Eppo SDKs for different languages. If you're an Eppo
//! user, you probably want to take a look at one of existing SDKs.
//!
//! # Overview
//!
//! `eppo_core` is organized as a set of building blocks that help to build Eppo SDKs. Different
//! languages have different constraints. Some languages might use all building blocks and others
//! might reimplement some pieces in the host language.
//!
//! [`Configuration`] is the heart of an SDK. It is an immutable structure that encapsulates all
//! server-provided configuration ([flag configurations](ufc::UniversalFlagConfig) and [bandit
//! models](bandits::BanditResponse)) that describes how SDK should evaluate user requests.
//!
//! [`ConfigurationStore`](configuration_store::ConfigurationStore) is a thread-safe multi-reader
//! multi-writer in-memory manager for [`Configuration`]. The job of configuration store is to be a
//! central authority on what configuration is currently active. Whenever configuration changes, it
//! is replaced completely. When a reader gets a configuration, it receives a *snapshot* that is not
//! affected by further writes—to provide a consistent response to user, it is important that
//! reader uses the same `Configuration` snapshot throughout the operation.
//!
//! [`ConfigurationFetcher`](configuration_fetcher::ConfigurationFetcher) is an HTTP client that
//! knows how to fetch [`Configuration`] from the server. It's best to save and reuse the same
//! instance, so it can reuse the connection.
//!
//! [`PollerThread`](poller_thread::PollerThread) launches a background thread that periodically
//! fetches a new `Configuration` (using `ConfigurationFetcher`) and updates
//! `ConfigurationStore`. This is the simplest way to keep SDK configuration up-to-date.
//!
//! [`eval`] module contains functions for flag and bandit evaluation. It also supports evaluation
//! with [details](eval::eval_details::EvaluationDetails). These functions return evaluation results
//! along with [`events`]—they do not log events automatically.
//!
//! [`events`] module contains definitions of [`AssignmentEvent`](events::AssignmentEvent) and
//! [`BanditEvent`](events::BanditEvent) that need to be submitted to user's analytics storage for
//! further analysis. `eppo_core` does not provide an "assignment logger" abstraction yet as
//! callback handling is currently too different between languages (e.g., in Ruby, it's too tedious
//! to call from Rust into Ruby, so we return events into Ruby land where they get logged).
//!
//! Because evaluation functions are pure functions (they don't have side-effects and don't use any
//! global state), they are a bit tedious to call directly. [`Evaluator`](eval::Evaluator) is a
//! helper to simplify SDK code and pass repeated parameters automatically.
//!
//! Most SDKs are built from a `ConfigurationStore`, a `PollerThread`, and an `Evaluator`.
//!
//! # Versioning
//!
//! This library follows semver. However, it is considered an internal library, so expect frequent
//! breaking changes and major version bumps.

#![warn(rustdoc::missing_crate_level_docs)]

pub mod bandits;
pub mod configuration_fetcher;
pub mod configuration_store;
pub mod eval;
pub mod events;
pub mod poller_thread;
#[cfg(feature = "pyo3")]
pub mod pyo3;
pub mod sharder;
pub mod ufc;

mod attributes;
mod configuration;
mod context_attributes;
mod error;
mod sdk_metadata;
mod str;

pub use crate::str::Str;
pub use attributes::{AttributeValue, Attributes};
pub use configuration::Configuration;
pub use context_attributes::ContextAttributes;
pub use error::{Error, EvaluationError, Result};
pub use sdk_metadata::SdkMetadata;
