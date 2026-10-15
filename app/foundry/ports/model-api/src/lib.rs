//! Owned model drafts and metadata admission; no object values or action runtime.
#![forbid(unsafe_code)]

#[macro_use]
mod wire;
mod admission;
mod bounds;
pub mod control;
mod discovery;
mod document;
mod integrity;
mod ledger;
mod named_strings;
mod native;
mod reference;
mod schema;
mod store;
mod transitions;
mod value_native;
mod values;

pub mod conformance;

pub use admission::PreparedPublication;
pub use discovery::{DraftPage, DraftPageRequest, DraftSummary};
pub use document::*;
pub use ledger::ModelLedger;
pub use reference::ReferenceModelStore;
pub use store::*;
pub use values::*;

pub use schema::{MAX_ONTOLOGY_SCHEMA_BYTES, schema_bytes};

#[cfg(test)]
mod named_strings_tests;
