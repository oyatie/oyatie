//! Transitional re-export of the Cedar-shaped policy kernel.
//!
//! The crate moved to `policy/core/cedar-domain` (package `policy-cedar-domain`)
//! when `policy/` was established as a capability: authorization evaluation is
//! owned by Policy, not by Identity.
//!
//! This shim exists so consumers migrate on their own schedule instead of
//! inside the move. It is deliberately behavior-free — every item below is the
//! same type the policy crate defines, not a copy. Depend on
//! `policy-cedar-domain` directly the next time you open a consumer for its own
//! reasons, and delete this crate once the last consumer has moved.

pub use policy_cedar_domain::*;
