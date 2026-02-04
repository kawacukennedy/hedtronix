//! HEDTRONIX Core Library
//! 
//! Core domain models and business logic for the HEDTRONIX healthcare operating system.
//! This crate provides CRDT-enabled data structures for offline-first operation.

pub mod models;
pub mod error;
pub mod types;
pub mod crdt;

pub use error::{Error, Result};
pub use models::*;
pub use types::*;
