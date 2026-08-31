//! Foundry records port: the append-only Action log.
//!
//! Every business write in Foundry is an Action that appends an envelope to a
//! per-tenant log; ontology state is a projection derived by replay, never
//! written directly. This port is therefore event-shaped from the start: an
//! adapter stores and returns envelopes, and everything an adapter must honor
//! is expressed as the executable conformance suite in [`conformance`], which
//! every adapter runs unchanged.
//!
//! The port owns its envelope types and depends on nothing: a log is
//! content-agnostic, and binding it to ontology types would couple every
//! future adapter to one consumer's schema.
#![forbid(unsafe_code)]

mod envelope;
mod log;

pub mod conformance;

pub use envelope::{ActionEnvelope, EnvelopeError, Receipt, SealedEnvelope};
pub use log::{RecordsLog, RecordsLogError};
