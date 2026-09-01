//! Zanzibar relationship expansion.
//!
//! `policy-cedar-domain` owns the vocabulary — tuples, usersets, rewrites,
//! consistency tokens and the tuple-store port. It has no evaluator, so a
//! `UsersetRewrite` there is a shape that means nothing on its own.
//!
//! This crate supplies the two missing halves: [`NamespaceConfig`], which
//! binds an object type and relation to a rewrite, and [`Expander`], which
//! walks that rewrite against a tuple store at one pinned snapshot.
//!
//! Every refusal denies. `Ok(false)` means the graph was walked and no grant
//! exists; an [`ExpansionError`] means it was not fully walked, and the two
//! must never be collapsed by a caller.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod bounds;
mod error;
mod expander;
mod namespace;
mod request;
mod session;
mod stratify;
mod walk;

pub use bounds::ExpansionBounds;
pub use error::ExpansionError;
pub use namespace::{NamespaceConfig, ValidatedNamespace};
pub use request::Expander;
pub use session::ExpansionSession;
