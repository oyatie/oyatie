//! Foundry edits: the wire plane of the M3 write spine.
//!
//! This crate owns the byte-law vocabulary that Action payloads are built
//! from: the edit vocabulary ([`OntologyEdit`] / [`EditSet`]), the payload
//! roots ([`ActionRecord`] / [`DenialRecord`]), and the spine-owned typed
//! value mirror ([`WireValue`] / [`WireProperty`]). Log bytes must never
//! track another crate's in-memory shape, so this crate depends on
//! NOTHING — the missing edge is the compiler-enforced guarantee.
//!
//! Tags are wire-frozen from birth: [`EditTag`] fixes the u8 of every edit
//! kind, including the reserved kinds ([`EditTag::UnsetProperties`],
//! [`EditTag::DeleteObject`], [`EditTag::DeleteLink`]) that have no
//! [`OntologyEdit`] variant yet — nothing may enter the log that the fold
//! cannot apply, so a reserved tag is representable only as a tag, never
//! as an edit. The canonical encoding of these shapes is the next lane;
//! golden byte vectors freeze each `wire_format_version` there forever.
#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod edit;
mod property;
mod record;
mod value;

pub use edit::{EditError, EditSet, EditTag, OntologyEdit};
pub use property::{WireDataClass, WireProperty, WirePropertyError, WireTier};
pub use record::{ActionRecord, DenialRecord, RecordError, WIRE_FORMAT_VERSION};
pub use value::{WireDate, WireDouble, WireValue, WireValueError};
