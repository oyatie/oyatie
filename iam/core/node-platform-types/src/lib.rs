#![cfg_attr(not(test), no_std)]
// This crate is a large surface of small primitive accessors and fallible
// constructors. The following pedantic lints would require annotating well over
// a hundred trivial methods without making the API meaningfully clearer, so we
// opt out crate-wide rather than littering per-item attributes:
//   - `must_use_candidate` / `return_self_not_must_use`: pure accessors and
//     builders where ignoring the result is already obviously a no-op.
//   - `missing_errors_doc`: the `Result`-returning functions document their
//     failure modes inline; a separate `# Errors` section adds noise here.
#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::missing_errors_doc
)]
//! # talos-core
//!
//! Foundational crate for the operating-system Talos migration. Every other crate in the
//! workspace depends on this one (and only this one).
//!
//! It mirrors the primitives found in the `siderolabs/talos` `machinery`
//! package: machine types, RBAC roles, semantic versions, node addressing,
//! resource identifiers, supported platforms, and the cross-cutting traits used
//! to run services and generate identifiers.
//!
//! The crate is `no_std` for real builds and only uses the `alloc` crate. Under
//! `cargo test` it links against `std` on the host so the test harness works.

extern crate alloc;

pub mod address;
pub mod cel;
pub mod error;
pub mod id;
pub mod machine_type;
pub mod os;
pub mod platform;
pub mod primitives;
pub mod resource;
pub mod role;
pub mod traits;
pub mod version;

pub use address::{Cidr, Hostname, NodeAddress, Port, ResourceId};
pub use cel::{
    DiskLocator, evaluate_disk_locator_bool_expression, validate_disk_locator_bool_expression,
    validate_volume_locator_bool_expression,
};
pub use error::{Error, Result};
pub use id::{Fingerprint, IdGenerator};
pub use machine_type::MachineType;
pub use os::{
    Clock, CommandExecutor, CommandOutput, FileSystem, InMemorySyscalls, ManualClock, MemoryFs,
    MockExecutor, MountEntry, PowerAction, SyscallProvider,
};
pub use platform::Platform;
pub use resource::{Metadata, Namespace, Phase, ResourceKind, ResourcePointer};
pub use role::{PREFIX as ROLE_PREFIX, Role, RoleSet};
pub use traits::Runnable;
pub use version::{Version, VersionRange};
