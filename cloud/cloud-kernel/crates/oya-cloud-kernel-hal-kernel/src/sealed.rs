//! The sealed-trait pattern.
//!
//! Several of the capability traits in this crate are meant to be implemented
//! **only** by the per-arch Frame backends shipped inside this workspace — never
//! by downstream code. We enforce that with the classic *sealed supertrait*
//! idiom: a public trait gets a private supertrait ([`Sealed`]) that only this
//! crate can name, so an external crate physically cannot satisfy the bound and
//! therefore cannot add a rogue implementation.
//!
//! Marker state-types used in type-state APIs (e.g. the W^X [`crate::mm`] flags
//! or the fallibility markers) are sealed the same way: the variant set is
//! closed, so the compiler can reason exhaustively about it.

/// Private marker every sealed trait inherits from. External crates cannot name
/// this trait, so they cannot implement anything sealed behind it.
pub trait Sealed {}
