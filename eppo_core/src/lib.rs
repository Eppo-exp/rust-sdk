//! `eppo_core` is a common library to build Eppo SDKs for different languages. If you're an Eppo
//! user, you probably want to take a look at one of existing SDKs.
//!
//! # Overview
//!
//! `eppo_core` is organized as a set of building blocks that help to build Eppo SDKs. Different
//! languages have different constraints. Some languages might use all building blocks and others
//! might reimplement some pieces in the host language.
//!
//! # Versioning
//!
//! This library follows semver. However, it is considered an internal library, so expect frequent
//! breaking changes and major version bumps.

#![warn(rustdoc::missing_crate_level_docs)]

pub mod bandits;
pub mod configuration_fetcher;
pub mod configuration_store;
pub mod poller_thread;
pub mod sharder;
pub mod ufc;

mod attributes;
mod configuration;
mod context_attributes;
mod error;

pub use attributes::{AttributeValue, Attributes};
pub use configuration::Configuration;
pub use context_attributes::ContextAttributes;
pub use error::{Error, EvaluationError, Result};
