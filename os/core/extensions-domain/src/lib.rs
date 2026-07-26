//! # talos-extensions
//!
//! Models Talos system extensions, porting `siderolabs/talos`
//! `pkg/machinery/extensions` and the machined extension controllers:
//!
//! - [`manifest`] — parsing the extension `manifest.yaml` (name, version,
//!   author, kind, Talos compatibility constraint) plus the
//!   [`manifest::ExtensionKind`] taxonomy and a dependency-free YAML-subset
//!   parser.
//! - [`layer`] — the per-extension filesystem [`layer::ExtensionLayer`] and the
//!   overlay [`layer::LayerSet`] that merges layers with reserved-path and
//!   conflict validation.
//! - [`config`] — the `ExtensionServiceConfig` machine-config document
//!   (environment + config files) and its validation.
//! - [`service`] — the service-extension lifecycle: [`service::ServiceSpec`],
//!   restart policy, a [`service::ServiceLauncher`] OS-boundary trait with an
//!   in-memory impl, and [`service::ExtensionService`] implementing
//!   [`os_kernel::traits::Runnable`].
//! - [`controller`] — the extensions controller that reconciles discovered
//!   layers into [`controller::ExtensionStatus`] resources and drives service
//!   startup.
//!
//! The crate uses only the standard library plus `talos-core`; it pulls in no
//! external crates so the workspace build stays fully offline.

pub mod config;
pub mod controller;
pub mod layer;
pub mod manifest;
pub mod service;

pub use config::{ConfigFile, EnvVar, ExtensionServiceConfig};
pub use controller::{ExtensionController, ExtensionPhase, ExtensionStatus};
pub use layer::{ExtensionLayer, LayerEntry, LayerSet};
pub use manifest::{
    Compatibility, ConstraintOp, ExtensionKind, ExtensionManifest, VersionConstraint,
};
pub use service::{
    ExtensionService, InMemoryLauncher, ProcessHandle, RestartPolicy, ServiceLauncher, ServiceSpec,
};

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::traits::{RunState, Runnable};
    use os_kernel::version::Version;

    /// End-to-end: parse a service manifest, reconcile it through the
    /// controller, and run the service.
    #[test]
    fn end_to_end_service_extension() {
        let yaml = "\
name: gvisor
version: v20231214.0.0
author: Sidero Labs
kind: service
compatibility:
  talos:
    version: \">= v1.6.0\"
";
        let manifest = ExtensionManifest::parse(yaml).unwrap();
        assert_eq!(manifest.kind, ExtensionKind::Service);

        let mut controller = ExtensionController::new(Version::new(1, 7, 0));
        controller.add_layer(ExtensionLayer::new(manifest.clone()));

        let statuses = controller.reconcile();
        assert_eq!(statuses["gvisor"].phase, ExtensionPhase::Ready);

        let services = controller.start_services().unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].state(), RunState::Running);
    }

    #[test]
    fn incompatible_extension_does_not_run() {
        let yaml = "\
name: futuristic
version: v1.0.0
kind: service
compatibility:
  talos:
    version: \">= v2.0.0\"
";
        let manifest = ExtensionManifest::parse(yaml).unwrap();
        let mut controller = ExtensionController::new(Version::new(1, 7, 0));
        controller.add_layer(ExtensionLayer::new(manifest));
        assert!(controller.start_services().unwrap().is_empty());
    }

    #[test]
    fn config_drives_service_environment() {
        let spec = ServiceSpec::new("gvisor", "/usr/local/bin/runsc");
        let mut svc = ExtensionService::new(spec, InMemoryLauncher::new());
        let cfg = ExtensionServiceConfig::new("gvisor")
            .with_env("RUNSC_DEBUG", "true")
            .with_file(ConfigFile::new("/etc/runsc.conf", "platform=ptrace"));
        cfg.validate().unwrap();
        svc.configure(cfg).unwrap();
        svc.start().unwrap();
        assert!(
            svc.effective_env()
                .contains(&"RUNSC_DEBUG=true".to_string())
        );
    }

    #[test]
    fn overlay_merges_compatible_layers() {
        let mut controller = ExtensionController::new(Version::new(1, 7, 0));
        controller.add_layer(
            ExtensionLayer::new(ExtensionManifest::new(
                "tool-a",
                Version::new(1, 0, 0),
                ExtensionKind::Rootfs,
            ))
            .with_entry(LayerEntry::file("/usr/local/bin/a")),
        );
        controller.add_layer(
            ExtensionLayer::new(ExtensionManifest::new(
                "tool-b",
                Version::new(1, 0, 0),
                ExtensionKind::Rootfs,
            ))
            .with_entry(LayerEntry::file("/usr/local/bin/b")),
        );
        let overlay = controller.build_overlay().unwrap();
        assert_eq!(
            overlay.merged_paths(),
            vec!["/usr/local/bin/a", "/usr/local/bin/b"]
        );
    }
}
