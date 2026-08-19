//! Shared service-root discovery for every gate that scans the repository tree.
//!
//! # Why this module exists
//!
//! Seven gate modules each carried their own private copy of
//!
//! ```ignore
//! const DEFAULT_SERVICE_ROOTS: &[&str] = &["cloud", "oya", "microservices"];
//! ```
//!
//! across 21 call sites. Two of those three roots stopped existing —
//! `cloud/` was deleted, `microservices/` was renamed away — and the
//! discovery helper swallowed the absence with `let Ok(entries) =
//! fs::read_dir(root) else { return out; }`. A missing root yielded an
//! empty scan and a green gate, so blocker gates reported success over a
//! fraction of the tree they believed they covered.
//!
//! Two rules follow from that failure, and this module exists to enforce
//! both in one place:
//!
//! 1. **The root set is derived, never hand-listed.** It comes from the
//!    closed capability registry (`governance/capability-registry.json`,
//!    the ADR-0562 placement authority), so registering a capability
//!    extends gate coverage automatically instead of silently leaving the
//!    new root unscanned.
//! 2. **An expected root that is absent is an ERROR that names the path.**
//!    Absence is the failure mode that produced this bug; it must never
//!    again read as an empty success.
//!
//! # What counts as a service root
//!
//! * Every registry `capabilities[]` entry whose `absorbs_current_dirs` is
//!   non-empty. The registry uses that list to record which directories a
//!   capability actually owns on disk, so a non-empty list is the
//!   registry's own statement that the root is materialized and must be
//!   present. On the tree this module was written against that predicate
//!   agrees with the filesystem for all 24 registered capabilities, and
//!   `expected_capability_roots_match_the_tree` keeps it honest.
//! * A registered capability with an EMPTY `absorbs_current_dirs` is
//!   registered but not yet materialized (today: `policy`, extracted from
//!   `iam` by ADR-0615 with its directories still to move). It is reported
//!   by [`ServiceRootScan::unmaterialized`] so it stays visible, but it is
//!   not scanned and not an error — the registry itself says nothing lives
//!   there yet. When its move lands, `absorbs_current_dirs` fills in and
//!   the root becomes expected without anyone editing this file.
//! * `app/`, the ADR-0562 composition ring. It is a `meta_directories`
//!   entry rather than a capability, and it is the only meta directory
//!   that hosts per-product capability document trees
//!   (`app/<product>/capabilities/…`). The other meta directories
//!   (`kernel/`, `os/`, `base/`, `governance/`, `build/`, `third-party/`)
//!   host no microservice document trees and are deliberately excluded —
//!   `governance/policy/*.cedar` in particular is the shared canonical
//!   policy envelope, not a microservice's policy fragment, and scanning
//!   it would pair unrelated documents.
//! * `oya/`, named explicitly below as a LEGACY root.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// Pre-ADR-0562 root that the capability-first strangler migration has not
/// finished moving.
///
/// `oya/` is in NEITHER the registry's `capabilities` nor its
/// `meta_directories`, so no registry-driven derivation can produce it —
/// and it holds the clear majority of the repository's capability
/// documents. Deriving roots from the registry alone would therefore drop
/// them from every gate: exactly the silent-shrink bug this module exists
/// to prevent, wearing a more principled hat.
///
/// It is listed here, visibly and on purpose, so that it is impossible to
/// lose by accident. **Delete this entry when the strangler migration
/// lands and `oya/` no longer exists on disk** — until then its absence is
/// an error like any other expected root's, which is what will tell you
/// the migration finished.
pub(crate) const LEGACY_SERVICE_ROOTS: &[&str] = &["oya"];

/// The one `meta_directories` entry that hosts capability document trees.
const COMPOSITION_RING_ROOT: &str = "app";

/// Path of the closed capability registry, relative to the repository root.
///
/// Gates run with the repository root as their working directory and use
/// relative service roots (`oya/…`), so this is resolved the same way.
const CAPABILITY_REGISTRY: &str = "governance/capability-registry.json";

/// The outcome of resolving the default service roots against the tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ServiceRootScan {
    /// Expected roots that exist and can be scanned.
    pub present: Vec<PathBuf>,
    /// Expected roots that are ABSENT. Non-empty means the caller must
    /// fail: this is the condition that used to pass as an empty scan.
    pub missing: Vec<PathBuf>,
    /// Registered capabilities the registry says are not yet materialized.
    /// Reported for visibility; neither scanned nor an error.
    pub unmaterialized: Vec<String>,
}

impl ServiceRootScan {
    /// Render the missing roots as one operator-facing error naming every
    /// absent path, or `None` when nothing is missing.
    pub(crate) fn missing_roots_error(&self) -> Option<String> {
        if self.missing.is_empty() {
            return None;
        }
        let names = self
            .missing
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        Some(format!(
            "service-root discovery failed: {} expected service root(s) absent from the tree: {names}. \
             An expected root that does not exist is a gate-coverage hole, not an empty scan. \
             Either the directory was deleted or moved without updating \
             {CAPABILITY_REGISTRY}, or a legacy root in \
             marketplace/facade/dev-cli/src/service_roots.rs::LEGACY_SERVICE_ROOTS has completed \
             its migration and must be removed from that list.",
            self.missing.len(),
        ))
    }
}

/// Resolve the default service roots for a gate, relative to `base`.
///
/// Returns an error when the registry cannot be read or parsed, or when
/// any expected root is absent from the tree.
pub(crate) fn default_service_roots_in(base: &Path) -> Result<Vec<PathBuf>, String> {
    let scan = scan_service_roots_in(base)?;
    if let Some(error) = scan.missing_roots_error() {
        return Err(error);
    }
    Ok(scan.present)
}

/// Resolve the default service roots for the repository the process is
/// running inside — the form every gate uses.
pub(crate) fn default_service_roots() -> Result<Vec<PathBuf>, String> {
    default_service_roots_in(&find_repo_root()?)
}

/// Locate the repository root: the nearest ancestor holding the capability
/// registry.
///
/// When the working directory already IS the repository root — how the
/// gates are invoked — this returns `.` so that discovered paths keep
/// their repository-relative shape (`oya/tasks/manifest.json`) in gate
/// output. Otherwise it ascends, which is what lets discovery work from a
/// crate subdirectory (notably under `cargo test`, whose working directory
/// is the package root) instead of silently finding nothing there.
fn find_repo_root() -> Result<PathBuf, String> {
    if Path::new(CAPABILITY_REGISTRY).is_file() {
        return Ok(PathBuf::from("."));
    }
    let mut dir = std::env::current_dir()
        .map_err(|error| format!("current directory unreadable: {error}"))?;
    loop {
        if dir.join(CAPABILITY_REGISTRY).is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err(format!(
                "capability registry unreadable at {CAPABILITY_REGISTRY}: not found in any \
                 ancestor of the working directory. Service-root discovery has no authority to \
                 derive from; gates cannot report coverage without it."
            ));
        }
    }
}

/// Full scan result, including absent and not-yet-materialized roots.
pub(crate) fn scan_service_roots_in(base: &Path) -> Result<ServiceRootScan, String> {
    let registry_path = base.join(CAPABILITY_REGISTRY);
    let registry = fs::read_to_string(&registry_path).map_err(|error| {
        format!(
            "capability registry unreadable at {}: {error}. Service-root discovery has no \
             authority to derive from; gates cannot report coverage without it.",
            registry_path.display()
        )
    })?;
    let derived = derive_expected_roots(&registry)?;

    let mut present = Vec::new();
    let mut missing = Vec::new();
    for name in &derived.expected {
        // Roots stay relative to `base` so gate output keeps its existing
        // repository-relative shape (`oya/tasks/...`).
        let candidate = if base == Path::new(".") {
            PathBuf::from(name)
        } else {
            base.join(name)
        };
        if candidate.is_dir() {
            present.push(candidate);
        } else {
            missing.push(candidate);
        }
    }

    Ok(ServiceRootScan {
        present,
        missing,
        unmaterialized: derived.unmaterialized,
    })
}

struct DerivedRoots {
    expected: Vec<String>,
    unmaterialized: Vec<String>,
}

/// Derive the expected + not-yet-materialized root names from the registry
/// document. Pure: no filesystem access, so it is unit-testable against a
/// fixture as well as against the real registry.
fn derive_expected_roots(registry_json: &str) -> Result<DerivedRoots, String> {
    let registry: Value = serde_json::from_str(registry_json)
        .map_err(|error| format!("capability registry parse failed: {error}"))?;

    let capabilities = registry
        .get("capabilities")
        .and_then(Value::as_array)
        .ok_or_else(|| "capability registry has no `capabilities` array".to_string())?;

    let mut expected: Vec<String> = Vec::new();
    let mut unmaterialized: Vec<String> = Vec::new();
    for capability in capabilities {
        let Some(name) = capability.get("name").and_then(Value::as_str) else {
            return Err("capability registry entry has no `name`".to_string());
        };
        let owns_directories = capability
            .get("absorbs_current_dirs")
            .and_then(Value::as_array)
            .is_some_and(|dirs| !dirs.is_empty());
        if owns_directories {
            expected.push(name.to_string());
        } else {
            unmaterialized.push(name.to_string());
        }
    }

    // The composition ring, and then the legacy root, appended explicitly.
    expected.push(COMPOSITION_RING_ROOT.to_string());
    for legacy in LEGACY_SERVICE_ROOTS {
        expected.push((*legacy).to_string());
    }

    expected.sort();
    expected.dedup();
    unmaterialized.sort();
    Ok(DerivedRoots {
        expected,
        unmaterialized,
    })
}

/// One directory holding a service's documents, plus the microservice name
/// the gates key their pairing on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ServiceSubpath {
    pub path: PathBuf,
    pub microservice: String,
}

/// List every `<relative>` directory under `root`, across BOTH layout
/// shapes the tree actually uses.
///
/// * Depth-2 — `<root>/<service>/<relative>/`. The common shape; the
///   microservice name is `<service>`.
/// * Depth-1 — `<root>/<relative>/`, with no service segment. Capabilities
///   that were not decomposed into per-service directories keep their
///   documents directly under the capability root. The predecessor helper
///   walked only the depth-2 shape, so every depth-1 document was
///   invisible to every gate even where the root itself was scanned.
///
/// The depth-1 microservice name is the ROOT ITSELF. That is the
/// defensible answer: under ADR-0562 the capability root with no service
/// segment *is* the unit that owns those documents, so keying on the root
/// pairs `<root>/capabilities/` against `<root>/policy/` — the same
/// "does THIS unit's claim have THIS unit's forbid rule?" tie the depth-2
/// shape gets — rather than collapsing them into a shared bucket.
///
/// Names are deliberately BARE (`tasks`, not `workflow/tasks`): downstream
/// kernels match them against declared name lists, notably
/// `check-ontology-projection-coverage`'s `CANONICAL_ENTITY_OWNERS`.
/// Qualifying them with the root would silently stop every one of those
/// comparisons from matching — the same class of quiet coverage loss this
/// module exists to prevent.
pub(crate) fn list_service_subpaths(root: &Path, relative: &str) -> Vec<ServiceSubpath> {
    let mut out = Vec::new();

    // Depth-1: <root>/<relative>/
    let direct = root.join(relative);
    if direct.is_dir() {
        out.push(ServiceSubpath {
            path: direct,
            microservice: leaf_name(root),
        });
    }

    // Depth-2: <root>/<service>/<relative>/
    if let Ok(entries) = fs::read_dir(root) {
        let mut nested: Vec<ServiceSubpath> = Vec::new();
        for entry in entries.flatten() {
            let service_dir = entry.path();
            if !service_dir.is_dir() {
                continue;
            }
            let candidate = service_dir.join(relative);
            if candidate.is_dir() {
                nested.push(ServiceSubpath {
                    microservice: leaf_name(&service_dir),
                    path: candidate,
                });
            }
        }
        // Deterministic gate output regardless of readdir order.
        nested.sort_by(|a, b| a.path.cmp(&b.path));
        out.extend(nested);
    }

    out
}

/// List every `<file_name>` document under `root`, across BOTH layout
/// shapes — the file counterpart of [`list_service_subpaths`].
///
/// * Depth-2 — `<root>/<service>/<file_name>`; name is `<service>`.
/// * Depth-1 — `<root>/<file_name>`; name is the root itself.
///
/// `manifest.json` has the same split as the document directories: most
/// services carry one at depth 2, while capability roots that were never
/// decomposed into per-service directories carry theirs at depth 1. The
/// predecessor walkers only ever looked at depth 2, so the depth-1
/// manifests were invisible to the manifest-driven gates.
pub(crate) fn list_service_files(root: &Path, file_name: &str) -> Vec<ServiceSubpath> {
    let mut out = Vec::new();

    let direct = root.join(file_name);
    if direct.is_file() {
        out.push(ServiceSubpath {
            path: direct,
            microservice: leaf_name(root),
        });
    }

    if let Ok(entries) = fs::read_dir(root) {
        let mut nested: Vec<ServiceSubpath> = Vec::new();
        for entry in entries.flatten() {
            let service_dir = entry.path();
            if !service_dir.is_dir() {
                continue;
            }
            let candidate = service_dir.join(file_name);
            if candidate.is_file() {
                nested.push(ServiceSubpath {
                    microservice: leaf_name(&service_dir),
                    path: candidate,
                });
            }
        }
        nested.sort_by(|a, b| a.path.cmp(&b.path));
        out.extend(nested);
    }

    out
}

/// Every service "unit" under the given roots: the ROOT ITSELF, plus each
/// `<root>/<service>` directory.
///
/// Use this for walkers that join a FIXED subpath onto each unit
/// (`unit/contracts/openapi`, `unit/backfill-replay.md`). Including the
/// root is what makes the depth-1 layout shape visible: 17 capability
/// roots carry `<root>/contracts/` with no service segment, and a
/// subdirectories-only enumeration never sees them. Because each unit
/// joins a distinct fixed subpath, the root unit and the service units
/// cannot collide.
///
/// Do NOT use this for walkers that recurse from the unit downwards — the
/// root unit would re-walk every service beneath it and double-count under
/// the wrong name. Those want [`nested_service_dirs_from_roots`].
pub(crate) fn service_units_from_roots(roots: &[PathBuf]) -> Vec<ServiceSubpath> {
    let mut out = Vec::new();
    for root in roots {
        if !root.is_dir() {
            continue;
        }
        out.push(ServiceSubpath {
            path: root.clone(),
            microservice: leaf_name(root),
        });
        out.extend(nested_service_dirs(root));
    }
    out
}

/// Only the `<root>/<service>` directories, never the root itself.
///
/// For walkers that recurse downwards from each unit, where including the
/// root would re-walk everything beneath it.
pub(crate) fn nested_service_dirs_from_roots(roots: &[PathBuf]) -> Vec<ServiceSubpath> {
    let mut out = Vec::new();
    for root in roots {
        out.extend(nested_service_dirs(root));
    }
    out
}

fn nested_service_dirs(root: &Path) -> Vec<ServiceSubpath> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let dir = entry.path();
            if dir.is_dir() {
                out.push(ServiceSubpath {
                    microservice: leaf_name(&dir),
                    path: dir,
                });
            }
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// The microservice name for a document directory: the path component
/// immediately before the `capabilities` / `policy` / `scorecards` segment.
///
/// The predecessor keyed on a literal `microservices` path component and
/// returned `None` for everything else. That component was renamed out of
/// the tree, so every document fell back to `unwrap_or_default()` — the
/// EMPTY string — and the whole repository audited as a single unnamed
/// microservice. The gate's core pairing degraded from "does this
/// microservice's claim have a matching forbid rule in this
/// microservice's policy?" to "does any forbid rule exist anywhere?".
pub(crate) fn microservice_name_for(path: &Path) -> Option<String> {
    let parent = path.parent()?;
    let name = parent.file_name()?.to_string_lossy().to_string();
    if name.is_empty() {
        return None;
    }
    Some(name)
}

fn leaf_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    const FIXTURE: &str = r#"{
      "capabilities": [
        { "name": "iam", "absorbs_current_dirs": ["iam", "iam/identity"] },
        { "name": "workflow", "absorbs_current_dirs": ["workflow"] },
        { "name": "policy", "absorbs_current_dirs": [] }
      ],
      "meta_directories": [ { "dir": "app/" } ]
    }"#;

    fn repo_root() -> PathBuf {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        while !root.join(CAPABILITY_REGISTRY).is_file() {
            assert!(
                root.pop(),
                "{CAPABILITY_REGISTRY} not found above {}",
                env!("CARGO_MANIFEST_DIR")
            );
        }
        root
    }

    #[test]
    fn derives_expected_roots_and_keeps_the_legacy_root() {
        let derived = derive_expected_roots(FIXTURE).expect("fixture derives");
        assert_eq!(
            derived.expected,
            vec![
                "app".to_string(),
                "iam".to_string(),
                "oya".to_string(),
                "workflow".to_string()
            ]
        );
    }

    #[test]
    fn capability_without_directories_is_unmaterialized_not_expected() {
        let derived = derive_expected_roots(FIXTURE).expect("fixture derives");
        assert_eq!(derived.unmaterialized, vec!["policy".to_string()]);
        assert!(!derived.expected.contains(&"policy".to_string()));
    }

    #[test]
    fn absent_expected_root_is_an_error_that_names_the_path() {
        let scan = ServiceRootScan {
            present: vec![PathBuf::from("oya")],
            missing: vec![PathBuf::from("cloud"), PathBuf::from("microservices")],
            unmaterialized: Vec::new(),
        };
        let error = scan.missing_roots_error().expect("missing roots error");
        assert!(error.contains("cloud"), "error names cloud: {error}");
        assert!(
            error.contains("microservices"),
            "error names microservices: {error}"
        );
        assert!(error.contains('2'), "error counts the roots: {error}");
    }

    #[test]
    fn no_missing_roots_yields_no_error() {
        let scan = ServiceRootScan {
            present: vec![PathBuf::from("oya")],
            missing: Vec::new(),
            unmaterialized: Vec::new(),
        };
        assert_eq!(scan.missing_roots_error(), None);
    }

    /// The registry-derived expectation must agree with the tree it is
    /// used to scan. If a capability's directories are deleted without the
    /// registry being updated (or vice versa) this fails here rather than
    /// shrinking every gate in silence.
    #[test]
    fn expected_capability_roots_match_the_tree() {
        let root = repo_root();
        let scan = scan_service_roots_in(&root).expect("scan the real tree");
        assert!(
            scan.missing.is_empty(),
            "expected service roots absent from the tree: {:?}",
            scan.missing
        );
        assert!(
            !scan.present.is_empty(),
            "service-root discovery produced no roots"
        );
    }

    /// Every root that actually holds capability documents must be
    /// discovered. This is the regression bar for the original defect: the
    /// hardcoded `["cloud", "oya", "microservices"]` list saw exactly one
    /// of these.
    #[test]
    fn discovery_covers_every_root_holding_capability_documents() {
        let root = repo_root();
        let scan = scan_service_roots_in(&root).expect("scan the real tree");
        let discovered: BTreeSet<String> = scan
            .present
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect();
        for holder in [
            "app",
            "audit",
            "billing",
            "comms",
            "console",
            "data",
            "flags",
            "gateway",
            "iac",
            "iam",
            "intelligence",
            "k8s",
            "marketplace",
            "observability",
            "oya",
            "secrets",
            "storage",
            "tenancy",
            "workflow",
        ] {
            assert!(
                discovered.contains(holder),
                "root {holder} holds capability documents but was not discovered; \
                 discovered={discovered:?}"
            );
        }
    }

    /// Both on-disk layout shapes must be enumerated. `workflow/` has the
    /// depth-2 shape (`workflow/tasks/capabilities/`) and `marketplace/`
    /// has the depth-1 shape (`marketplace/capabilities/`).
    #[test]
    fn lists_both_depth_one_and_depth_two_capability_shapes() {
        let root = repo_root();

        let nested = list_service_subpaths(&root.join("workflow"), "capabilities");
        assert!(
            nested
                .iter()
                .any(|s| s.microservice == "tasks" && s.path.ends_with("tasks/capabilities")),
            "depth-2 shape workflow/tasks/capabilities not found: {nested:?}"
        );

        let direct = list_service_subpaths(&root.join("marketplace"), "capabilities");
        assert!(
            direct
                .iter()
                .any(|s| s.microservice == "marketplace"
                    && s.path.ends_with("marketplace/capabilities")),
            "depth-1 shape marketplace/capabilities not found under its own name: {direct:?}"
        );
    }

    #[test]
    fn microservice_name_for_extracts_both_live_shapes() {
        // Depth-2: the component before `capabilities`.
        assert_eq!(
            microservice_name_for(Path::new("workflow/tasks/capabilities")),
            Some("tasks".into())
        );
        assert_eq!(
            microservice_name_for(Path::new("oya/payments/scorecards")),
            Some("payments".into())
        );
        // Depth-1: the capability root itself owns the documents.
        assert_eq!(
            microservice_name_for(Path::new("marketplace/capabilities")),
            Some("marketplace".into())
        );
        assert_eq!(
            microservice_name_for(Path::new("gateway/policy")),
            Some("gateway".into())
        );
    }

    #[test]
    fn microservice_name_for_returns_none_without_a_parent_segment() {
        assert_eq!(microservice_name_for(Path::new("capabilities")), None);
    }

    #[test]
    fn missing_registry_is_an_error_not_an_empty_scan() {
        let error = scan_service_roots_in(Path::new("/nonexistent/repo/root"))
            .expect_err("absent registry must error");
        assert!(
            error.contains("capability registry unreadable"),
            "error explains itself: {error}"
        );
    }
}
