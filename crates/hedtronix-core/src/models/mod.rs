//! Domain models for HEDTRONIX
//!
//! These models represent the core entities in the healthcare system
//! with CRDT support for offline-first operation.

mod user;
mod device;
mod patient;
mod appointment;
mod clinical_note;
mod billing;
mod audit_log;
mod department;
mod room;
mod encounter;

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
