//! Domain models for HEDTRONIX
//!
//! These models represent the core entities in the healthcare system
//! with CRDT support for offline-first operation.

pub mod user;
pub mod device;
pub mod patient;
pub mod appointment;
pub mod clinical_note;
pub mod billing;
pub mod audit_log;
pub mod department;
pub mod room;
pub mod encounter;

pub use user::*;
pub use device::*;
pub use patient::*;
pub use appointment::*;
pub use clinical_note::*;
pub use billing::*;
pub use audit_log::*;
pub use department::*;
pub use room::*;
pub use encounter::*;
