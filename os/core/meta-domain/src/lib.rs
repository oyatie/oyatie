//! # talos-meta
//!
//! A `no_std` port of the Talos META partition subsystem, mirroring
//! `internal/pkg/meta` and `pkg/machinery/meta` from `siderolabs/talos`.
//!
//! The META partition is a small, dedicated disk partition that holds a
//! key/value store of *machine state*: information that must survive a wipe of
//! the system partition but is not part of the (immutable) machine
//! configuration. Examples include the staged upgrade image reference, the
//! unique machine token, and the state-partition encryption config.
//!
//! On disk the store is serialized in the **ADV** ("advertised") format, a
//! tag-length-value container with a fixed magic header and a CRC32 checksum.
//! Two on-disk layouts exist in real Talos: the legacy `ADV` format and the
//! newer `ADV1` format which supports tags larger than one byte and values
//! larger than 255 bytes. This crate models `ADV1`.
//!
//! ## Modules
//! - [`key`]    — the [`MetaKey`] enum of well-known tag identifiers.
//! - [`value`]  — the [`MetaValue`] wrapper around a stored byte string.
//! - [`adv`]    — the ADV binary container ([`AdvHeader`], [`Adv`]) in both the
//!   legacy and ADV1 layouts.
//! - [`codec`]  — the JSON transport codec for META records.
//! - [`meta`]   — [`Meta`], the high-level typed key/value facade.
//! - [`store`]  — the [`MetaStore`] trait + an in-memory implementation.
//! - [`partition`] — [`MetaPartition`], modelling the on-disk partition layout.
//!
//! The crate uses the Rust standard library.

pub mod adv;
pub mod codec;
pub mod key;
pub mod meta;
pub mod partition;
pub mod store;
pub mod value;

pub use adv::{ADV_HEADER_LEN, ADV_LEGACY_MAGIC, ADV1_MAGIC, Adv, AdvFormat, AdvHeader};
pub use codec::{JsonRecord, adv_from_json, adv_to_json, decode_records, encode_records};
pub use key::MetaKey;
pub use meta::Meta;
pub use partition::{META_PARTITION_LABEL, META_PARTITION_SIZE, MetaPartition, Slot};
pub use store::{InMemoryMetaStore, MetaStore};
pub use value::MetaValue;

/// Re-export of the shared error type so downstream crates do not need to
/// depend on `talos-core` directly just to match errors.
pub use os_kernel::{Error, Result};
