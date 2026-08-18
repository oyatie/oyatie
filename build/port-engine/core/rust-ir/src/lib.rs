//! # port-engine-rust-ir — the typed target IR and its renderer.
//!
//! ADR-0637 D1 core face: holds `TargetIr` rendering with stable ordering and normalized
//! formatting.
//!
//! The IR is a TREE. Items, statements and expressions are data; `quote!` lowers them to tokens,
//! `syn` parses those tokens, and `prettyplease` formats the result. Nothing in the emit path
//! builds Rust by string formatting, which is what lets three things be decided structurally
//! instead of textually:
//!
//! - **Precedence.** A text builder cannot see its own nesting, so the previous IR parenthesised
//!   every binary expression unconditionally. Here an operand is bracketed exactly when its own
//!   precedence binds looser than the position it sits in.
//! - **Visibility.** A `"pub "` string prefix concatenated into a trait body produced `pub fn` on
//!   a trait method — which `syn` parses and `rustc` rejects. Visibility is now a value, and a
//!   trait item simply has none to give.
//! - **Documentation.** Doc comments are carried as data and rendered as `///`, rather than
//!   dropped because a `format!` had nowhere to put them.
//!
//! ## Neutrality
//!
//! A needle scan over EMITTED BYTES used to live here, refusing rendered output containing corpus
//! identifiers or the source language's keywords. It is gone, and `tests/fences.rs` records why:
//! the engine must not KNOW about the corpus, which the production-source scan enforces, but the
//! engine's OUTPUT must mention it, because emitting a translation of that corpus is the point.
#![forbid(unsafe_code)]

mod expr;
mod item;
mod lower;
mod lower_body;
mod lower_parts;
mod ops;
mod render;
mod ty;

pub use expr::{MatchArm, RustExpr, RustStmt};
pub use item::{Receiver, RustField, RustFn, RustItem, RustParam, StructShape, Visibility};
pub use lower::lower_file;
pub use ops::{BinaryOp, Precedence, UnaryOp};
pub use render::{EmptyRenderer, FORMATTER_ID, RustIr, RustRenderer};
pub use ty::RustType;

/// Fail-closed readiness gate. `true` once the typed IR and its renderer are present.
#[must_use]
pub const fn w0_ready() -> bool {
    true
}
