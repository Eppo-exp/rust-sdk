//! Eppo Rust SDK.
//!
//! See [`EppoClient`].

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
pub use client::{AssignmentValue, AttributeValue, EppoClient, SubjectAttributes};
pub use config::ClientConfig;
pub use error::{Error, Result};
pub use poller::PollerThread;
