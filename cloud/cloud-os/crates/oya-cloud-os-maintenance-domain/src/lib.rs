//! # talos-maintenance
//!
//! Models Talos *maintenance mode*: the limited API surface a node serves
//! before it has applied a machine configuration. This mirrors
//! `siderolabs/talos` `internal/app/maintenance`.
//!
//! When a Talos node boots without a machine config (no config on disk, none
//! delivered via platform metadata), it enters maintenance mode. In that state
//! `machined` brings up a stripped-down gRPC service that exposes only the
//! handful of methods needed to get the node configured:
//!
//! - `ApplyConfiguration` — accept a machine config, validate it, persist it,
//!   and trigger the transition out of maintenance (and optionally an install).
//! - `GenerateConfiguration` — generate a config + talosconfig for the node.
//! - `Upgrade` — upgrade the running maintenance image.
//! - `Reset` — wipe the machine and reboot.
//! - read-only introspection (`Version`, `Hostname`, ...).
//!
//! Because no PKI exists yet, the maintenance service bootstraps a *self-signed*
//! server certificate and serves TLS without client-cert verification; the
//! dashboard/HTTP config endpoint is also served so an operator can push config
//! over HTTP. Once a valid config is applied the node leaves maintenance mode.
//!
//! All OS boundaries (cert generation, config persistence, install trigger,
//! reboot) are modeled as traits with in-memory implementations so the state
//! machine and validation logic are exercised in pure, offline Rust.
//!
//! ## Module map
//!
//! - [`service`] — the maintenance gRPC method enum, the [`MaintenanceService`]
//!   trait, the [`MaintenanceState`] machine, and the request/response types.
//! - [`config_apply`] — the `ApplyConfiguration` flow: validation, persistence,
//!   apply-mode semantics, and the maintenance -> configured transition.
//! - [`server`] — the [`MaintenanceServer`] that wires state + boundaries
//!   together and dispatches RPCs, plus the HTTP config endpoint.
//! - [`tls`] — self-signed certificate bootstrap modeled as a trait.

pub mod config_apply;
pub mod server;
pub mod service;
pub mod tls;

pub use config_apply::{
    ApplyConfigInput, ApplyConfigOutcome, ApplyError, ApplyMode, ConfigSink, ConfigValidator,
    DefaultConfigValidator, InMemoryConfigSink, StoredConfig,
};
pub use server::{
    ConfigSource, DefaultMaintenanceServer, HttpConfigEndpoint, HttpMethod, HttpRequest,
    HttpResponse, InstallRequest, Installer, MaintenanceServer, NoopInstaller, Rebooter,
    RecordingInstaller, RecordingRebooter,
};
pub use service::{
    GenerateRequest, GenerateResponse, MaintenanceError, MaintenanceMethod, MaintenancePhase,
    MaintenanceService, MaintenanceState, ResetRequest, UpgradeRequest, UpgradeResponse,
};
pub use tls::{CertBootstrap, Certificate, InMemoryCertBootstrap, SubjectAltNames, TlsConfig};
