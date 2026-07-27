//! Talos installer and bootloader subsystem.
//!
//! A faithful, self-contained model of the Talos (siderolabs/talos) install
//! path: A/B boot entries, a pluggable [`bootloader::Bootloader`] interface
//! with a concrete [`grub::Grub`] implementation, the SecureBoot key hierarchy
//! and signing boundary, and an upgrade [`install::Upgrade`] state machine that
//! extracts an image onto the inactive slot and either confirms or rolls back.
//!
//! All kernel/disk/network interactions are modeled as traits with in-memory
//! implementations so the logic is fully testable offline.

// These pedantic lints fire pervasively across this model crate and adding the
// suggested attributes/sections would be pure noise without changing behavior or
// improving the API: nearly every accessor is a `#[must_use]` candidate, the
// error/panic conditions are already described in prose, and `doc_markdown`
// flags proper nouns (SecureBoot, GRUB) in doc comments.
#![allow(
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::return_self_not_must_use
)]

pub mod boot_entry;
pub mod bootloader;
pub mod grub;
pub mod install;
pub mod sdboot;
pub mod secureboot;

pub use boot_entry::{BootEntry, BootSlot, BootState};
pub use bootloader::{BootResult, Bootloader, BootloaderError, BootloaderKind};
pub use grub::Grub;
pub use install::{
    DiskLayout, HashingExtractor, ImageExtractor, InstallImage, Partition, Upgrade, UpgradeError,
    UpgradeOptions, UpgradePhase, compare_versions, digest_bytes, upgrade_permitted,
};
pub use sdboot::{SdBoot, SdBootEntry};
pub use secureboot::{
    EnrollmentPhase, PcrPolicy, SecureBootKeys, SecureBootState, Signature, Signer, SigningKey,
    Uki, UkiSection, measure_pcr11,
};
