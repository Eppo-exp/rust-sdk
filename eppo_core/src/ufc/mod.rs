//! Universal Flag Configuration.
mod assignment;
mod error;
mod eval;
mod eval_details;
mod eval_visitor;
mod models;
mod rules;

pub use assignment::{Assignment, AssignmentEvent, AssignmentValue};
pub use error::FlagEvaluationError;
pub use models::*;
