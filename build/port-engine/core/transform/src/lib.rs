//! # port-engine-transform — plan → `RustIr` construction apply.
//!
//! ADR-0637 D1: the kernel plans; this core face applies rule **construction** / **precondition**
//! data (strings from the pack) into a deterministic [`RustIr`]. Unknown constructions refuse.
//!
//! Two rule shapes, told apart by DATA rather than by a flag:
//!
//! - A rule that captures nothing is **unit-level**: one region per unit. This is the shape the
//!   canary path uses, and it is unchanged.
//! - A rule that captures one or more declaration kinds is **declaration-level**: one region per
//!   captured declaration. This is the shape that actually ports Go.
//!
//! Neutrality is unchanged and load-bearing. No Go type, kind, or keyword is named in this crate:
//! `int` arrives as a key to look up in the pack's type map, and `struct` as a string the pack
//! chose to capture. What this crate DOES own is Rust's side of the translation — identifier
//! casing and the shape of an emitted item — because that is the target language it renders, not
//! the source language it must stay ignorant of.
#![forbid(unsafe_code)]

mod apply;
mod body;
mod error;
mod items;
mod naming;
mod resolve;
mod signature;
mod vocabulary;

pub use apply::{apply, apply_with_provenance};
pub use error::TransformError;
pub use naming::{
    region_id_for, region_id_for_declaration, sanitize_ident, to_pascal_case, to_screaming_snake,
    to_snake_case,
};
pub use vocabulary::{
    ATTR_OP, ATTR_REF, ATTR_SOURCE_NODE, ATTR_VALUE, CONSTRUCTION_EMPTY_CANARY,
    CONSTRUCTION_PASS_THROUGH, CONSTRUCTION_RUST_CONST, CONSTRUCTION_RUST_FN,
    CONSTRUCTION_RUST_FN_BODY, CONSTRUCTION_RUST_NEWTYPE, CONSTRUCTION_RUST_STRUCT,
    CONSTRUCTION_RUST_TRAIT, CONSTRUCTION_RUST_TYPE_ALIAS, FLAG_EXPORTED, FLAG_POINTER_RECEIVER,
    FLAG_VARIADIC, PRECONDITION_UNIT_PRESENT,
};

/// Fail-closed readiness gate. `true` once transform apply is present.
#[must_use]
pub const fn w0_ready() -> bool {
    true
}
