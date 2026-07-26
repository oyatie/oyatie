//! `talos-cosi`: a port of the COSI (Common Operating System Interface)
//! resource model and controller-runtime primitives used by Talos, mirroring
//! `cosi-project/runtime`.
//!
//! The crate provides:
//!
//! - [`metadata`]: the `(namespace, kind, id)` identity triple plus version,
//!   lifecycle [`Phase`](metadata::Phase), finalizers and labels.
//! - [`resource`]: the [`Resource`](resource::Resource) trait and
//!   [`ResourceKind`](resource::ResourceKind) descriptor.
//! - [`reduced`]: the metadata-only [`ReducedResource`](reduced::ReducedResource)
//!   view delivered in watch events.
//! - [`watch`]: watch [`Event`](watch::Event)s and a bounded
//!   [`WatchChannel`](watch::WatchChannel).
//! - [`store`]: an in-memory [`State`](store::State) store with CRUD,
//!   optimistic concurrency, teardown/finalizer lifecycle and watch fan-out.
//! - [`inputs`]: controller [`Input`](inputs::Input)/[`Output`](inputs::Output)
//!   declarations and the [`Spec`](inputs::Spec) surface.
//! - [`controller`]: the [`Controller`](controller::Controller) trait and the
//!   access-controlled [`ReconcileContext`](controller::ReconcileContext).
//! - [`runtime`]: the [`Runtime`](runtime::Runtime) controller engine with a
//!   dependency graph and a deterministic reconcile-to-convergence loop.

pub mod controller;
pub mod inputs;
pub mod metadata;
pub mod reduced;
pub mod resource;
pub mod runtime;
pub mod store;
pub mod watch;

pub use controller::{Controller, ControllerError, ReconcileContext, ReconcileResult};
pub use inputs::{Input, InputKind, Output, Spec};
pub use metadata::{Finalizers, Labels, Metadata, Phase};
pub use reduced::ReducedResource;
pub use resource::{AnyResource, Resource, ResourceKind};
pub use runtime::{ReconcileRecord, Runtime, RuntimeError};
pub use store::{State, StoreError, StoreResult};
pub use watch::{Event, EventKind, WatchChannel};
