//! Extensions controller.
//!
//! Mirrors `internal/app/machined/pkg/controllers/runtime` extension handling:
//! it scans the installed extension manifests/layers, validates them against the
//! running Talos version (compatibility + layer conflicts), and produces an
//! `ExtensionStatus` resource per extension. For service extensions it also
//! drives the [`ExtensionService`] lifecycle. The controller is modeled as a
//! pure reconcile function over in-memory inputs so it is fully testable.

use std::collections::BTreeMap;

use os_kernel::error::{Error, Result};
use os_kernel::traits::Runnable;
use os_kernel::version::Version;

use crate::layer::{ExtensionLayer, LayerSet};
use crate::manifest::{ExtensionKind, ExtensionManifest};
use crate::service::{ExtensionService, InMemoryLauncher, ServiceSpec};

/// The health/compatibility verdict for one extension after reconciliation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionPhase {
    /// Validated and compatible; will be applied.
    Ready,
    /// Rejected because it is not compatible with the running Talos version.
    Incompatible,
    /// Rejected because its layer is structurally invalid or conflicts.
    Invalid,
}

/// The controller-produced status resource for one extension, analogous to
/// Talos's `runtime.ExtensionStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionStatus {
    /// Extension name (the resource id).
    pub name: String,
    /// Extension's own version.
    pub version: Version,
    /// Extension kind.
    pub kind: ExtensionKind,
    /// Reconciliation verdict.
    pub phase: ExtensionPhase,
    /// Human-readable detail (empty when Ready).
    pub message: String,
}

impl ExtensionStatus {
    /// Whether the extension passed reconciliation.
    pub fn is_ready(&self) -> bool {
        self.phase == ExtensionPhase::Ready
    }
}

/// Inputs to the controller: the discovered extension layers plus the running
/// Talos version.
pub struct ExtensionController {
    talos_version: Version,
    layers: Vec<ExtensionLayer>,
}

impl ExtensionController {
    /// Construct a controller for `talos_version`.
    pub fn new(talos_version: Version) -> Self {
        ExtensionController {
            talos_version,
            layers: Vec::new(),
        }
    }

    /// Register a discovered extension layer.
    pub fn add_layer(&mut self, layer: ExtensionLayer) {
        self.layers.push(layer);
    }

    /// Reconcile: validate every layer, detect cross-layer path conflicts, and
    /// emit one [`ExtensionStatus`] per extension. The returned map is keyed by
    /// extension name and sorted.
    ///
    /// A layer that is individually valid but conflicts with another is marked
    /// [`ExtensionPhase::Invalid`]; an incompatible one is
    /// [`ExtensionPhase::Incompatible`]. Only `Ready` extensions are merged into
    /// the overlay.
    pub fn reconcile(&self) -> BTreeMap<String, ExtensionStatus> {
        let mut statuses: BTreeMap<String, ExtensionStatus> = BTreeMap::new();
        // First pass: per-layer validity & compatibility.
        let mut ready_layers: Vec<&ExtensionLayer> = Vec::new();
        for layer in &self.layers {
            let m = &layer.manifest;
            let (phase, message) = if let Err(e) = layer.validate() {
                (ExtensionPhase::Invalid, e.to_string())
            } else if !m.is_compatible_with(&self.talos_version) {
                (
                    ExtensionPhase::Incompatible,
                    format!("requires Talos other than {}", self.talos_version),
                )
            } else {
                ready_layers.push(layer);
                (ExtensionPhase::Ready, String::new())
            };
            statuses.insert(
                m.name.clone(),
                ExtensionStatus {
                    name: m.name.clone(),
                    version: m.version.clone(),
                    kind: m.kind,
                    phase,
                    message,
                },
            );
        }

        // Second pass: cross-layer conflict detection over the ready layers.
        let mut owner: BTreeMap<&str, &str> = BTreeMap::new();
        for layer in &ready_layers {
            for path in layer.file_paths() {
                if let Some(prev) = owner.insert(path, layer.manifest.name.as_str()) {
                    // Demote both conflicting extensions to Invalid.
                    let msg = format!("path conflict on '{path}'");
                    if let Some(s) = statuses.get_mut(layer.manifest.name.as_str()) {
                        s.phase = ExtensionPhase::Invalid;
                        s.message = msg.clone();
                    }
                    if let Some(s) = statuses.get_mut(prev) {
                        s.phase = ExtensionPhase::Invalid;
                        s.message = msg.clone();
                    }
                }
            }
        }

        statuses
    }

    /// Build a validated [`LayerSet`] from only the extensions that reconcile to
    /// `Ready`. Returns an error if even the ready subset fails final
    /// validation (which would indicate a controller bug).
    pub fn build_overlay(&self) -> Result<LayerSet> {
        let statuses = self.reconcile();
        let mut set = LayerSet::new();
        for layer in &self.layers {
            if statuses
                .get(&layer.manifest.name)
                .map(|s| s.is_ready())
                .unwrap_or(false)
            {
                set.push(layer.clone())?;
            }
        }
        Ok(set)
    }

    /// For every `Ready` service-kind extension, build and start an
    /// [`ExtensionService`] on the supplied in-memory launcher factory. Returns
    /// the started services keyed by name. Used to model the machined service
    /// startup pass.
    pub fn start_services(&self) -> Result<Vec<ExtensionService<InMemoryLauncher>>> {
        let statuses = self.reconcile();
        let mut started = Vec::new();
        for layer in &self.layers {
            if layer.manifest.kind != ExtensionKind::Service {
                continue;
            }
            if !statuses
                .get(&layer.manifest.name)
                .map(|s| s.is_ready())
                .unwrap_or(false)
            {
                continue;
            }
            let spec = derive_service_spec(&layer.manifest)?;
            let mut svc = ExtensionService::new(spec, InMemoryLauncher::new());
            svc.start()?;
            started.push(svc);
        }
        Ok(started)
    }
}

/// Derive a default [`ServiceSpec`] for a service extension whose spec is not
/// otherwise provided (entrypoint defaults to `/usr/local/bin/<name>`).
fn derive_service_spec(manifest: &ExtensionManifest) -> Result<ServiceSpec> {
    if manifest.kind != ExtensionKind::Service {
        return Err(Error::invalid(format!(
            "extension '{}' is not a service",
            manifest.name
        )));
    }
    Ok(ServiceSpec::new(
        manifest.name.clone(),
        format!("/usr/local/bin/{}", manifest.name),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::LayerEntry;
    use crate::manifest::{Compatibility, VersionConstraint};

    fn manifest(name: &str, kind: ExtensionKind) -> ExtensionManifest {
        ExtensionManifest::new(name, Version::new(1, 0, 0), kind)
    }

    #[test]
    fn reconcile_marks_ready() {
        let mut c = ExtensionController::new(Version::new(1, 7, 0));
        c.add_layer(
            ExtensionLayer::new(manifest("a", ExtensionKind::Rootfs))
                .with_entry(LayerEntry::file("/usr/bin/a")),
        );
        let statuses = c.reconcile();
        assert_eq!(statuses["a"].phase, ExtensionPhase::Ready);
        assert!(statuses["a"].is_ready());
    }

    #[test]
    fn reconcile_marks_incompatible() {
        let mut m = manifest("needsnew", ExtensionKind::Rootfs);
        m.compatibility = Compatibility {
            talos: Some(VersionConstraint::parse(">= v1.9.0").unwrap()),
        };
        let mut c = ExtensionController::new(Version::new(1, 7, 0));
        c.add_layer(ExtensionLayer::new(m).with_entry(LayerEntry::file("/usr/bin/x")));
        let statuses = c.reconcile();
        assert_eq!(statuses["needsnew"].phase, ExtensionPhase::Incompatible);
    }

    #[test]
    fn reconcile_demotes_conflicting_pair() {
        let mut c = ExtensionController::new(Version::new(1, 7, 0));
        c.add_layer(
            ExtensionLayer::new(manifest("a", ExtensionKind::Rootfs))
                .with_entry(LayerEntry::file("/usr/bin/shared")),
        );
        c.add_layer(
            ExtensionLayer::new(manifest("b", ExtensionKind::Rootfs))
                .with_entry(LayerEntry::file("/usr/bin/shared")),
        );
        let statuses = c.reconcile();
        assert_eq!(statuses["a"].phase, ExtensionPhase::Invalid);
        assert_eq!(statuses["b"].phase, ExtensionPhase::Invalid);
    }

    #[test]
    fn build_overlay_excludes_bad_extensions() {
        let mut c = ExtensionController::new(Version::new(1, 7, 0));
        c.add_layer(
            ExtensionLayer::new(manifest("good", ExtensionKind::Rootfs))
                .with_entry(LayerEntry::file("/usr/bin/good")),
        );
        // Invalid: firmware outside /lib/firmware.
        c.add_layer(
            ExtensionLayer::new(manifest("badfw", ExtensionKind::Firmware))
                .with_entry(LayerEntry::file("/usr/bin/bad")),
        );
        let overlay = c.build_overlay().unwrap();
        assert_eq!(overlay.len(), 1);
        assert_eq!(overlay.merged_paths(), vec!["/usr/bin/good"]);
    }

    #[test]
    fn start_services_only_starts_ready_service_kind() {
        let mut c = ExtensionController::new(Version::new(1, 7, 0));
        c.add_layer(ExtensionLayer::new(manifest(
            "svc1",
            ExtensionKind::Service,
        )));
        c.add_layer(
            ExtensionLayer::new(manifest("rootfs1", ExtensionKind::Rootfs))
                .with_entry(LayerEntry::file("/usr/bin/r")),
        );
        let started = c.start_services().unwrap();
        assert_eq!(started.len(), 1);
        assert_eq!(started[0].spec().name, "svc1");
        assert_eq!(started[0].state(), os_kernel::traits::RunState::Running);
    }
}
