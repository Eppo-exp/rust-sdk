//! Eppo Rust SDK.
//!
//! See [`EppoClient`].

#![warn(rustdoc::missing_crate_level_docs)]
#![warn(missing_docs)]

mod assignment_logger;
mod client;
mod config;
mod configuration_store;
mod eval;
mod rules;
mod sharder;
mod ufc;

pub use assignment_logger::{AssignmentEvent, AssignmentLogger};
pub use client::{AttributeValue, EppoClient, SubjectAttributes};
pub use config::ClientConfig;
