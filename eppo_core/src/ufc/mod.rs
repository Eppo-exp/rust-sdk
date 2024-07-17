//! Universal Flag Configuration.
mod assignment;
mod error;
mod eval;
mod models;
mod rules;

pub use assignment::{Assignment, AssignmentEvent, AssignmentValue};
pub use error::FlagEvaluationError;
pub use models::*;
