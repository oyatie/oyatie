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

/// This crate's own sources, for the engine-identity axis assembled by the facade.
mod sources;
pub use sources::CRATE_SOURCES;

mod apply;
mod body;
mod body_call;
mod body_parts;
mod body_cond;
mod body_expr;
mod body_idiom;
mod body_literal;
mod body_failure;
mod body_index;
mod body_loops;
mod body_ops;
mod docs;
mod error;
mod failure;
mod impls;
mod items;
mod items_self;
mod items_static;
mod naming;
mod ownership;
mod params;
mod promote;
mod resolve;
mod returns;
mod signature_table;
mod resolve_policy;
mod resolve_tables;
mod resolve_types;
mod sentinel;
mod signature;
mod survey;
mod vocabulary;

pub use apply::{TransformOutput, apply, apply_with_provenance};
pub use error::TransformError;
pub use naming::{
    escape_keyword, module_name, module_path, region_id_for, region_id_for_declaration,
    sanitize_ident, to_pascal_case, to_screaming_snake, to_snake_case,
};
pub use ownership::{DispositionLog, DispositionRecord, OwnershipContext};
pub use survey::{SurveyEntry, SurveyReport, survey};
pub use vocabulary::{
    ATTR_DOC, ATTR_OP, ATTR_REF, ATTR_SOURCE_NODE, ATTR_VALUE, CONSTRUCTION_EMPTY_CANARY,
    CONSTRUCTION_PASS_THROUGH, CONSTRUCTION_RUST_CONST, CONSTRUCTION_RUST_FN,
    CONSTRUCTION_RUST_FN_BODY, CONSTRUCTION_RUST_NEWTYPE, CONSTRUCTION_RUST_STRUCT,
    CONSTRUCTION_RUST_STRUCT_BODY, CONSTRUCTION_RUST_TRAIT, CONSTRUCTION_RUST_TYPE_ALIAS,
    FLAG_EXPORTED, FLAG_POINTER_RECEIVER, FLAG_VARIADIC, PRECONDITION_UNIT_PRESENT,
};

/// Fail-closed readiness gate. `true` once transform apply is present.
#[must_use]
pub const fn w0_ready() -> bool {
    true
}
