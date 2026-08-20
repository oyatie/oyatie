//! Registry-derived scan-root resolution for the `ci/facade` gate fleet.
//!
//! # The class of defect this removes
//!
//! Most connected gates decide WHAT TO SCAN from a hand-enumerated list of directory names carried
//! in their policy JSON. Those lists drift from the tree, silently, in one direction: a capability
//! lands, nobody edits the gate, and the gate reports GREEN over a set that no longer contains the
//! new code. Measured on `embedded-asset-hermeticity` before this crate existed: 16 declared roots,
//! 18 of the 24 REGISTERED capabilities absent, 3 declared roots dead. A dangling embedded asset
//! planted in `comms/` left the gate green.
//!
//! The repository already solved this one level up. The root `Cargo.toml` used to enumerate
//! workspace members one glob per capability; its comment records the diagnosis verbatim — "the
//! array was a mutex again, one level up, which is why every reorg move serialized on this file
//! (68 of 68 movers)" — and the fix was to replace 24 per-capability globs with four SHAPE globs
//! (`*/core/*`, `*/ports/*`, `*/adapters/*`, `*/facade/*`) that describe ADR-0562's layout instead
//! of enumerating its instances. A conforming capability is a member BY CONSTRUCTION.
//!
//! This crate is that same move for gates. The authority for what exists is the closed capability
//! registry, `governance/capability-registry.json` (ADR-0562 as amended by ADR-0615). A gate routed
//! through [`derive`] scans every materialized capability and every meta directory the moment it
//! lands, with zero edit to the gate.
//!
//! # The three rules that make absence loud
//!
//! 1. **A root the registry claims is MATERIALIZED but that is absent from disk is an ERROR that
//!    names the path** ([`DeriveError::MaterializedRootAbsent`]). Absence reading as success is the
//!    entire defect class; a `continue` on a missing directory is not a skip, it is a silent
//!    coverage hole.
//! 2. **A root the registry declares but that has NOT materialized is PENDING, not scanned, and
//!    reported by name** ([`ScanRootSet::pending`]). A caller freezes the pending set two-sided, so
//!    a pending root that LANDS must be struck from the frozen set in the same change that makes it
//!    real — the property `scan-root-liveness`'s hand-maintained `forward_declarations` map was
//!    buying with per-gate bookkeeping.
//! 3. **A LEGACY root carries a written deletion condition and its disappearance is an ERROR**
//!    ([`DeriveError::LegacyRootAbsent`]). Legacy roots are not in the registry, so they cannot be
//!    derived; they are enumerated here, once, for the whole fleet, and they shrink to nothing as
//!    the reorg drains them. This mirrors the root `Cargo.toml`'s own legacy glob block.
//!
//! # What is deliberately NOT derived
//!
//! `third-party/` is a meta directory holding reindeer-vendored UPSTREAM sources. First-party
//! doctrine gates must not police code the repository did not write, so it is carved out here with
//! a written reason rather than left to each gate's exclude list to remember. See
//! [`VENDORED_META_DIRS`].

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

/// Path of the closed capability registry, relative to the repository root.
pub const CAPABILITY_REGISTRY_PATH: &str = "governance/capability-registry.json";

// ---------------------------------------------------------------------------
// Legacy roots — the reorg's remaining debt, enumerated ONCE for the whole fleet
// ---------------------------------------------------------------------------

/// A pre-ADR-0562 top-level root that is not in the closed capability registry and therefore cannot
/// be derived. Each carries the condition under which its entry is DELETED from this list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LegacyRoot {
    /// Repository-relative directory name.
    pub path: &'static str,
    /// The event that retires this entry. Enforced, not decorative: when the directory stops
    /// existing, [`derive`] raises [`DeriveError::LegacyRootAbsent`] and the entry must go.
    pub deletion_condition: &'static str,
}

/// The legacy roots, shared by every gate routed through this crate. Shrinks to nothing as the
/// ADR-0562 capability moves land; the root `Cargo.toml` carries the matching legacy member globs
/// and empties out on the same schedule.
pub const LEGACY_ROOTS: &[LegacyRoot] = &[
    LegacyRoot {
        path: "oya",
        deletion_condition: "Delete when oya/ no longer exists on disk — i.e. when every \
                             oya/<service>/crates/oya-* crate has been absorbed by a registered \
                             capability. The root Cargo.toml's `oya/*/crates/oya-*` and \
                             `oya/office/oya-*` member globs retire in the same change.",
    },
    LegacyRoot {
        path: "libs",
        deletion_condition: "Delete when libs/ no longer exists on disk — i.e. when the libs/ \
                             consolidation move has re-homed every shared kernel into base/ or an \
                             owning capability. The root Cargo.toml's `libs/oya-*` member glob \
                             retires in the same change.",
    },
    LegacyRoot {
        path: "tools",
        deletion_condition: "Delete when tools/ no longer exists on disk — i.e. when every \
                             tools/oya-* and tools/*-app crate has moved to its owning capability's \
                             facade. The root Cargo.toml's two `tools/` member globs retire in the \
                             same change.",
    },
    LegacyRoot {
        path: "infra",
        deletion_condition: "Delete when infra/ no longer exists on disk — i.e. when the running \
                             GitOps surface has finished draining into the iac and k8s \
                             capabilities. infra/ owns no workspace members, so no Cargo.toml glob \
                             retires with it.",
    },
];

/// Meta directories that hold code the repository did NOT write. Derived roots exclude these: a
/// first-party doctrine gate that walks vendored upstream sources reports findings nobody in this
/// repository can fix, which is how an exclude list grows until it is load-bearing and unreviewed.
///
/// Deletion condition for the single entry: delete when `third-party/` stops holding
/// reindeer-vendored upstream crate sources.
pub const VENDORED_META_DIRS: &[&str] = &["third-party"];

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// Where a resolved root came from. Callers that need to report coverage per provenance (a gate
/// reporting "12 capability roots, 3 meta roots, 4 legacy roots") read this rather than
/// re-deriving it from the path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    /// A registered capability (`governance/capability-registry.json` → `capabilities[]`).
    Capability,
    /// A meta directory (`governance/capability-registry.json` → `meta_directories[]`).
    Meta,
    /// A pre-ADR-0562 root carried in [`LEGACY_ROOTS`].
    Legacy,
}

impl Origin {
    /// Stable lowercase token for use in finding keys and diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Origin::Capability => "capability",
            Origin::Meta => "meta",
            Origin::Legacy => "legacy",
        }
    }
}

/// One resolved, materialized root a gate should scan.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScanRoot {
    /// Repository-relative directory name, no trailing slash.
    pub path: String,
    /// Where the root came from.
    pub origin: Origin,
}

/// One root that is REGISTERED but has not materialized on disk yet. Not scanned — and reported by
/// name so a caller can freeze the set two-sided instead of letting the gap sit unaudited.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PendingRoot {
    /// Repository-relative directory name, no trailing slash.
    pub path: String,
    /// Where the declaration came from.
    pub origin: Origin,
    /// Why it is legitimately absent today, taken from the registry.
    pub reason: String,
}

/// The derived scan-root set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanRootSet {
    /// Materialized roots to scan, sorted, deduplicated.
    pub roots: Vec<ScanRoot>,
    /// Registered-but-absent roots, sorted. Covered by construction the moment they land.
    pub pending: Vec<PendingRoot>,
    /// Capabilities whose directory EXISTS while the registry still records them as unmaterialized
    /// (`absorbs_current_dirs` empty). Included in [`ScanRootSet::roots`] — scanning wins over
    /// bookkeeping — and surfaced here so the registry drift is repaired rather than tolerated.
    pub materialized_but_unregistered: Vec<String>,
}

impl ScanRootSet {
    /// The scan-root paths, sorted. The form most gate collectors want.
    #[must_use]
    pub fn root_paths(&self) -> Vec<String> {
        self.roots.iter().map(|root| root.path.clone()).collect()
    }

    /// The pending-root paths as a set, for two-sided freezing against a committed baseline.
    #[must_use]
    pub fn pending_paths(&self) -> BTreeSet<String> {
        self.pending.iter().map(|p| p.path.clone()).collect()
    }
}

/// A derivation failure. Every variant NAMES THE PATH: a gate that cannot say which root it failed
/// to resolve has reproduced the silent-skip defect in a different colour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeriveError {
    /// The registry file could not be read.
    RegistryUnreadable {
        /// Path attempted.
        path: String,
        /// Underlying error text.
        detail: String,
    },
    /// The registry parsed but does not carry the shape this crate depends on.
    RegistryShape {
        /// What was expected.
        detail: String,
    },
    /// The registry records this capability as materialized (`absorbs_current_dirs` non-empty) but
    /// the directory is absent. Never a skip: the gate would scan less than the registry promises.
    MaterializedRootAbsent {
        /// Capability name.
        capability: String,
        /// The absent repository-relative path.
        path: String,
    },
    /// A legacy root's directory is gone, so its deletion condition is met and its entry in
    /// [`LEGACY_ROOTS`] must be deleted. Two-sided: the list cannot outlive the debt.
    LegacyRootAbsent {
        /// The absent repository-relative path.
        path: String,
        /// The written condition that is now satisfied.
        deletion_condition: &'static str,
    },
}

impl std::fmt::Display for DeriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeriveError::RegistryUnreadable { path, detail } => {
                write!(f, "capability registry unreadable at `{path}`: {detail}")
            }
            DeriveError::RegistryShape { detail } => {
                write!(f, "capability registry shape: {detail}")
            }
            DeriveError::MaterializedRootAbsent { capability, path } => write!(
                f,
                "capability `{capability}` is recorded as materialized (absorbs_current_dirs is \
                 non-empty) but its root `{path}/` is absent — the gate would scan less than the \
                 registry promises. Either the capability moved and the registry is stale, or the \
                 directory was deleted without retiring the registry row."
            ),
            DeriveError::LegacyRootAbsent {
                path,
                deletion_condition,
            } => write!(
                f,
                "legacy root `{path}/` no longer exists, so its deletion condition is met — delete \
                 its LEGACY_ROOTS entry in ci/adapters/scan-root-derivation. Condition as written: \
                 {deletion_condition}"
            ),
        }
    }
}

impl std::error::Error for DeriveError {}

// ---------------------------------------------------------------------------
// Derivation
// ---------------------------------------------------------------------------

/// Derive the scan-root set from an already-parsed capability registry.
///
/// `exists` answers "does this repository-relative directory exist?". Keeping it a parameter makes
/// the derivation itself pure and unit-testable against a synthetic tree — the whole rule set can be
/// exercised without a filesystem, which is how the RED cases below are proven to fail closed.
///
/// # Errors
///
/// Returns every [`DeriveError`] found, not just the first: a caller repairing registry drift wants
/// the full list, and reporting one path at a time turns a single repair into N red runs.
pub fn derive(
    registry: &Value,
    exists: &dyn Fn(&str) -> bool,
) -> Result<ScanRootSet, Vec<DeriveError>> {
    let mut errors: Vec<DeriveError> = Vec::new();
    let mut roots: BTreeMap<String, Origin> = BTreeMap::new();
    let mut pending: Vec<PendingRoot> = Vec::new();
    let mut materialized_but_unregistered: Vec<String> = Vec::new();

    let Some(capabilities) = registry.get("capabilities").and_then(Value::as_array) else {
        return Err(vec![DeriveError::RegistryShape {
            detail: "`capabilities` must be an array".to_owned(),
        }]);
    };
    let Some(meta_dirs) = registry.get("meta_directories").and_then(Value::as_array) else {
        return Err(vec![DeriveError::RegistryShape {
            detail: "`meta_directories` must be an array".to_owned(),
        }]);
    };

    // --- capabilities -------------------------------------------------------
    // The materialization predicate is `absorbs_current_dirs` being non-empty: the registry records
    // there which on-disk directories the capability has absorbed, so an empty list is the registry
    // stating, in its own vocabulary, that nothing has landed yet.
    for capability in capabilities {
        let Some(name) = capability.get("name").and_then(Value::as_str) else {
            errors.push(DeriveError::RegistryShape {
                detail: "a capability row has no string `name`".to_owned(),
            });
            continue;
        };
        let absorbed = capability
            .get("absorbs_current_dirs")
            .and_then(Value::as_array)
            .map(|dirs| !dirs.is_empty())
            .unwrap_or(false);
        let on_disk = exists(name);

        match (absorbed, on_disk) {
            (true, true) => {
                roots.insert(name.to_owned(), Origin::Capability);
            }
            (true, false) => errors.push(DeriveError::MaterializedRootAbsent {
                capability: name.to_owned(),
                path: name.to_owned(),
            }),
            (false, true) => {
                // Scanning wins over bookkeeping: the code is there, so it gets scanned. The drift
                // is reported rather than swallowed so the registry row gets repaired.
                roots.insert(name.to_owned(), Origin::Capability);
                materialized_but_unregistered.push(name.to_owned());
            }
            (false, false) => pending.push(PendingRoot {
                path: name.to_owned(),
                origin: Origin::Capability,
                reason: format!(
                    "registered capability with no absorbed directories yet; scanned by \
                     construction the moment `{name}/` lands"
                ),
            }),
        }
    }

    // --- meta directories ---------------------------------------------------
    // Every meta directory except the vendored carve-out. NOT filtered on the registry's
    // `owns_crates` flag: build/ and governance/ are both recorded as owning zero CAPABILITY
    // crates while in fact holding first-party Rust (build/port-engine/core/*, governance/corpus/*),
    // so keying coverage on that flag would narrow the scan on a technicality about ownership.
    for meta in meta_dirs {
        let Some(dir) = meta.get("dir").and_then(Value::as_str) else {
            errors.push(DeriveError::RegistryShape {
                detail: "a meta_directories row has no string `dir`".to_owned(),
            });
            continue;
        };
        let path = dir.trim_end_matches('/');
        if path.is_empty() {
            errors.push(DeriveError::RegistryShape {
                detail: "a meta_directories row has an empty `dir`".to_owned(),
            });
            continue;
        }
        if VENDORED_META_DIRS.contains(&path) {
            continue;
        }
        if exists(path) {
            roots.entry(path.to_owned()).or_insert(Origin::Meta);
        } else {
            // A meta directory has no materialization field, so absence is read as "declared
            // destination, not yet landed" — the same status a capability with no absorbed dirs
            // gets. It is reported, never silently dropped, and the caller freezes it two-sided.
            pending.push(PendingRoot {
                path: path.to_owned(),
                origin: Origin::Meta,
                reason: format!(
                    "declared meta directory that has not landed yet; scanned by construction the \
                     moment `{path}/` lands"
                ),
            });
        }
    }

    // --- legacy roots -------------------------------------------------------
    for legacy in LEGACY_ROOTS {
        if exists(legacy.path) {
            roots
                .entry(legacy.path.to_owned())
                .or_insert(Origin::Legacy);
        } else {
            errors.push(DeriveError::LegacyRootAbsent {
                path: legacy.path.to_owned(),
                deletion_condition: legacy.deletion_condition,
            });
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    pending.sort();
    materialized_but_unregistered.sort();
    Ok(ScanRootSet {
        roots: roots
            .into_iter()
            .map(|(path, origin)| ScanRoot { path, origin })
            .collect(),
        pending,
        materialized_but_unregistered,
    })
}

/// Read `governance/capability-registry.json` under `repo_root` and derive the scan-root set
/// against the real tree.
///
/// # Errors
///
/// See [`derive`]; additionally [`DeriveError::RegistryUnreadable`] when the registry cannot be read
/// or parsed.
pub fn derive_from_repo(repo_root: &Path) -> Result<ScanRootSet, Vec<DeriveError>> {
    let registry_path = repo_root.join(CAPABILITY_REGISTRY_PATH);
    let text = match std::fs::read_to_string(&registry_path) {
        Ok(text) => text,
        Err(e) => {
            return Err(vec![DeriveError::RegistryUnreadable {
                path: CAPABILITY_REGISTRY_PATH.to_owned(),
                detail: e.to_string(),
            }]);
        }
    };
    let registry: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(e) => {
            return Err(vec![DeriveError::RegistryUnreadable {
                path: CAPABILITY_REGISTRY_PATH.to_owned(),
                detail: e.to_string(),
            }]);
        }
    };
    derive(&registry, &|candidate: &str| {
        repo_root.join(candidate).is_dir()
    })
}

// ---------------------------------------------------------------------------
// Policy-driven resolution — the single entry point every routed gate calls
// ---------------------------------------------------------------------------

/// The resolved scan-root set for one gate run, in the flat string form collectors want.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    /// Roots to walk, sorted. Every one is proven to exist.
    pub roots: Vec<String>,
    /// Registered roots that have not materialized, sorted. Not walked; reported so the caller can
    /// freeze the set two-sided.
    pub pending: Vec<String>,
    /// `"capability-registry"` or `"policy-explicit"`.
    pub source: &'static str,
}

/// Why a gate policy's scan roots could not be resolved. Each variant is renderable by the gate into
/// its own error type without losing the path that failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// The policy declares neither `scan_root_source` nor a non-empty `scan_roots` array.
    NoScanRootsDeclared,
    /// `scan_root_source.kind` is not a form this resolver implements.
    UnknownSourceKind(String),
    /// An explicitly declared root does not exist. NEVER a skip.
    DeclaredRootAbsent(String),
    /// Registry-derived resolution failed; see [`DeriveError`].
    Derivation(Vec<DeriveError>),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::NoScanRootsDeclared => write!(
                f,
                "policy must carry either `scan_root_source` (registry-derived) or a non-empty \
                 `scan_roots` array of strings"
            ),
            ResolveError::UnknownSourceKind(kind) => write!(
                f,
                "unknown scan_root_source.kind `{kind}`; the only derived form is \
                 `capability_registry_derived`"
            ),
            ResolveError::DeclaredRootAbsent(path) => write!(
                f,
                "declared scan root `{path}` does not exist — the gate would report GREEN over a \
                 root it never walked. Remove the declaration or restore the path; absence is never \
                 a skip."
            ),
            ResolveError::Derivation(errors) => write!(f, "{}", render_errors(errors)),
        }
    }
}

impl std::error::Error for ResolveError {}

/// Resolve the roots a gate walks, from its policy document.
///
/// TWO MODES, and the DEFAULT is derivation. `scan_root_source.kind == "capability_registry_derived"`
/// resolves the roots from `governance/capability-registry.json`: every materialized capability,
/// every landed meta directory except the vendored carve-out, and the shared legacy list. Nothing is
/// enumerated per gate, so a capability that materializes is scanned BY CONSTRUCTION.
///
/// A literal `scan_roots` array remains supported so a gate stays pack-shaped for an adopting
/// repository that has no capability registry. In THAT mode a declared root that is absent is a hard
/// [`ResolveError::DeclaredRootAbsent`], not a skip — the drift has to be loud in both modes or the
/// fix only holds in one of them.
///
/// # Errors
///
/// See [`ResolveError`].
pub fn resolve_policy_scan_roots(
    repo_root: &Path,
    policy: &Value,
) -> Result<Resolution, ResolveError> {
    if let Some(source) = policy.get("scan_root_source") {
        let kind = source
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if kind != "capability_registry_derived" {
            return Err(ResolveError::UnknownSourceKind(kind.to_owned()));
        }
        let derived = derive_from_repo(repo_root).map_err(ResolveError::Derivation)?;
        return Ok(Resolution {
            roots: derived.root_paths(),
            pending: derived.pending_paths().into_iter().collect(),
            source: "capability-registry",
        });
    }

    let declared: Vec<String> = policy
        .get("scan_roots")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    if declared.is_empty() {
        return Err(ResolveError::NoScanRootsDeclared);
    }
    for candidate in &declared {
        if !repo_root.join(candidate).is_dir() {
            return Err(ResolveError::DeclaredRootAbsent(candidate.clone()));
        }
    }
    Ok(Resolution {
        roots: declared,
        pending: Vec::new(),
        source: "policy-explicit",
    })
}

/// Render a derivation failure as the multi-line message a gate should print before failing. Shared
/// so every routed gate reports the same thing the same way.
#[must_use]
pub fn render_errors(errors: &[DeriveError]) -> String {
    let mut out = String::from(
        "scan-root derivation FAILED — the gate refuses to scan a set it cannot fully resolve:\n",
    );
    for error in errors {
        out.push_str("  * ");
        out.push_str(&error.to_string());
        out.push('\n');
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn registry() -> Value {
        json!({
            "capabilities": [
                { "name": "comms", "absorbs_current_dirs": ["comms", "comms/mail"] },
                { "name": "ci", "absorbs_current_dirs": ["ci"] },
                { "name": "policy", "absorbs_current_dirs": [] }
            ],
            "meta_directories": [
                { "dir": "kernel/", "owns_crates": true },
                { "dir": "base/", "owns_crates": true },
                { "dir": "build/", "owns_crates": false },
                { "dir": "third-party/", "owns_crates": false }
            ]
        })
    }

    fn present(paths: &[&'static str]) -> impl Fn(&str) -> bool + use<> {
        let set: BTreeSet<String> = paths.iter().map(|p| (*p).to_owned()).collect();
        move |candidate: &str| set.contains(candidate)
    }

    /// Everything the registry says is materialized, plus every landed meta dir, plus legacy.
    #[test]
    fn derives_materialized_capabilities_meta_and_legacy_roots() {
        let exists = present(&[
            "comms", "ci", "kernel", "build", "oya", "libs", "tools", "infra",
        ]);
        let set = derive(&registry(), &exists).expect("derives");
        assert_eq!(
            set.root_paths(),
            vec![
                "build", "ci", "comms", "infra", "kernel", "libs", "oya", "tools"
            ]
        );
        assert_eq!(
            set.roots
                .iter()
                .find(|r| r.path == "comms")
                .map(|r| r.origin),
            Some(Origin::Capability)
        );
        assert_eq!(
            set.roots
                .iter()
                .find(|r| r.path == "kernel")
                .map(|r| r.origin),
            Some(Origin::Meta)
        );
        assert_eq!(
            set.roots.iter().find(|r| r.path == "oya").map(|r| r.origin),
            Some(Origin::Legacy)
        );
    }

    /// THE FINDING, INVERTED. A capability the registry calls materialized but that is absent is an
    /// ERROR naming the path — never the `continue` that made a dangling asset in comms/ read GREEN.
    #[test]
    fn red_materialized_capability_with_no_directory_is_an_error_naming_the_path() {
        let exists = present(&["ci", "kernel", "build", "oya", "libs", "tools", "infra"]);
        let errors = derive(&registry(), &exists).expect_err("absent materialized root must fail");
        assert_eq!(
            errors,
            vec![DeriveError::MaterializedRootAbsent {
                capability: "comms".to_owned(),
                path: "comms".to_owned(),
            }]
        );
        assert!(
            errors[0].to_string().contains("`comms`"),
            "the error must NAME the path: {}",
            errors[0]
        );
    }

    /// Registered-but-unmaterialized roots are PENDING, reported by name, and not scanned. A caller
    /// freezes this set two-sided so a landing root cannot stay an unaudited exemption.
    #[test]
    fn unmaterialized_roots_are_pending_not_silently_dropped() {
        let exists = present(&[
            "comms", "ci", "kernel", "build", "oya", "libs", "tools", "infra",
        ]);
        let set = derive(&registry(), &exists).expect("derives");
        assert_eq!(
            set.pending_paths(),
            ["base".to_owned(), "policy".to_owned()]
                .into_iter()
                .collect()
        );
        assert!(
            !set.root_paths().contains(&"policy".to_owned()),
            "a pending root is not scanned"
        );
    }

    /// The by-construction property, stated as a test: a pending capability that materializes moves
    /// into the scanned set with NO edit to this crate and no edit to any gate policy.
    #[test]
    fn a_pending_capability_that_materializes_is_scanned_with_zero_edit() {
        let mut reg = registry();
        reg["capabilities"][2]["absorbs_current_dirs"] = json!(["policy"]);
        let exists = present(&[
            "comms", "ci", "policy", "kernel", "build", "oya", "libs", "tools", "infra",
        ]);
        let set = derive(&reg, &exists).expect("derives");
        assert!(set.root_paths().contains(&"policy".to_owned()));
        assert!(!set.pending_paths().contains("policy"));
    }

    /// A legacy root whose directory is gone has met its written deletion condition, and that is an
    /// ERROR until the entry is deleted — the list cannot outlive the debt.
    #[test]
    fn red_legacy_root_that_has_drained_is_an_error_until_its_entry_is_deleted() {
        let exists = present(&["comms", "ci", "kernel", "build", "oya", "tools", "infra"]);
        let errors = derive(&registry(), &exists).expect_err("drained legacy root must fail");
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            DeriveError::LegacyRootAbsent { path, .. } if path == "libs"
        ));
        assert!(errors[0].to_string().contains("deletion condition is met"));
    }

    /// Vendored upstream sources are never derived, by construction rather than by every gate's
    /// exclude list remembering.
    #[test]
    fn vendored_meta_directories_are_never_scanned() {
        let exists = present(&[
            "comms",
            "ci",
            "kernel",
            "build",
            "third-party",
            "oya",
            "libs",
            "tools",
            "infra",
        ]);
        let set = derive(&registry(), &exists).expect("derives");
        assert!(!set.root_paths().contains(&"third-party".to_owned()));
        assert!(!set.pending_paths().contains("third-party"));
    }

    /// A capability directory that lands before its registry row is filled in is SCANNED (coverage
    /// wins) and reported as drift (bookkeeping is repaired, not tolerated).
    #[test]
    fn a_directory_ahead_of_its_registry_row_is_scanned_and_reported_as_drift() {
        let exists = present(&[
            "comms", "ci", "policy", "kernel", "build", "oya", "libs", "tools", "infra",
        ]);
        let set = derive(&registry(), &exists).expect("derives");
        assert!(set.root_paths().contains(&"policy".to_owned()));
        assert_eq!(set.materialized_but_unregistered, vec!["policy".to_owned()]);
    }

    /// All failures are reported at once; a repair pass should not need N red runs to find N paths.
    #[test]
    fn every_failure_is_reported_not_just_the_first() {
        let exists = present(&["ci", "kernel", "build", "oya", "infra"]);
        let errors = derive(&registry(), &exists).expect_err("multiple failures");
        assert_eq!(
            errors.len(),
            3,
            "comms absent + libs drained + tools drained"
        );
        let rendered = render_errors(&errors);
        for needle in ["comms", "libs", "tools"] {
            assert!(rendered.contains(needle), "{rendered} must name {needle}");
        }
    }

    #[test]
    fn registry_shape_failures_name_what_was_expected() {
        let errors = derive(&json!({ "capabilities": [] }), &present(&[]))
            .expect_err("missing meta_directories");
        assert_eq!(
            errors,
            vec![DeriveError::RegistryShape {
                detail: "`meta_directories` must be an array".to_owned(),
            }]
        );
    }

    /// Every legacy entry carries a real, non-placeholder deletion condition. A written condition
    /// nobody wrote is how a "temporary" list becomes permanent.
    #[test]
    fn every_legacy_root_carries_a_written_deletion_condition() {
        for legacy in LEGACY_ROOTS {
            assert!(
                legacy.deletion_condition.len() > 60,
                "{} has no substantive deletion condition",
                legacy.path
            );
            assert!(
                legacy.deletion_condition.contains("Delete when"),
                "{} must state the retirement event",
                legacy.path
            );
        }
    }
}
