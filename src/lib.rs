//! Eppo Rust SDK.
//!
//! See [`EppoClient`].

#![warn(rustdoc::missing_crate_level_docs)]
#![warn(missing_docs)]

mod assignment_logger;
mod client;
mod config;
mod rules;
mod ufc;

pub use assignment_logger::{AssignmentEvent, AssignmentLogger};
pub use client::{EppoClient, SubjectAttributes};
pub use config::Config;
