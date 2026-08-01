//! WASM runtime discipline gate — advisory CI lane per ADR-0200.
//!
//! # What this gate enforces
//!
//! ADR-0200 makes Wasmtime the canonical WASM runtime, accessed via
//! the single `oya-shared-wasm-runtime-kernel` substrate. No
//! µservice may import `wasmtime`, `wasmer`, or `wasmedge` directly
//! — those imports belong inside the kernel adapter only.
//!
//! Violations:
//!
//! 1. `DirectRuntimeImport` — a µservice Cargo.toml lists one of
//!    the forbidden crates as a direct dependency.
//! 2. `KernelMissing` — a Cargo.toml mentions WASM execution
//!    intent (substring `wasm-execute` or similar marker) but
//!    fails to depend on `oya-shared-wasm-runtime-kernel`.
//! 3. `KernelCrateImportsForbiddenAdapterOutsideAllowlist` —
//!    only the kernel adapter sub-crate (allowlisted by name) may
//!    pull in `wasmtime`. Other crates that try are flagged.
//!
//! # Layer
//!
//! `domain` (port-in-kernel per ADR-0056).
//!
//! # Naming justification
//!
//! `check-wasm-runtime-discipline` follows the ADR-0532/0533 de-branded grammar:
//! `<group:check>-<axis:wasm-runtime-discipline>`.
//!
//! # References
//!
//! - ADR-0200 — WASM runtime canonical (Wasmtime).

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]
#![allow(clippy::result_large_err)]

use std::fmt;

/// One Cargo.toml under audit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoManifestText {
    pub crate_name: String,
    pub path: String,
    pub contents: String,
}

/// Configuration: which crate names are allowed to depend on a
/// forbidden runtime crate directly. The canonical allowlist is
/// the kernel-adapter sub-crate; parent-wiring can extend.
#[derive(Clone, Debug, Default)]
pub struct DisciplineConfig {
    pub adapter_allowlist: Vec<String>,
}

impl DisciplineConfig {
    /// Default allowlist: only the canonical kernel adapter
    /// may import `wasmtime` directly. The kernel crate itself
    /// (port-in-kernel) is dep-free; the adapter sub-crate ships
    /// the integration.
    #[must_use]
    pub fn canonical() -> Self {
        Self {
            adapter_allowlist: vec!["oya-shared-wasm-runtime-kernel-adapter-wasmtime".to_string()],
        }
    }
}

/// Successful audit report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisciplineReport {
    pub manifests_checked: usize,
}

/// Violation record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisciplineViolation {
    pub crate_name: String,
    pub manifest_path: String,
    pub kind: ViolationKind,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ViolationKind {
    /// Direct dependency on `wasmtime` / `wasmer` / `wasmedge`
    /// outside the adapter allowlist.
    DirectRuntimeImport,
    /// Crate signals WASM execution but does not depend on the
    /// canonical kernel.
    KernelMissing,
    /// Crate not in the allowlist imports the canonical runtime
    /// directly.
    KernelCrateImportsForbiddenAdapterOutsideAllowlist,
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ViolationKind::DirectRuntimeImport => f.write_str("direct-runtime-import"),
            ViolationKind::KernelMissing => f.write_str("kernel-missing"),
            ViolationKind::KernelCrateImportsForbiddenAdapterOutsideAllowlist => {
                f.write_str("kernel-crate-imports-forbidden-adapter-outside-allowlist")
            }
        }
    }
}

const FORBIDDEN_RUNTIME_CRATES: &[&str] = &["wasmtime", "wasmer", "wasmedge"];
const KERNEL_CRATE: &str = "oya-shared-wasm-runtime-kernel";
const WASM_INTENT_MARKERS: &[&str] = &["wasm-execute", "wasm-sandbox", "envoy-wasm-filter"];

/// Audit a batch of manifests against the WASM runtime discipline
/// invariants. Order-independent; the same input always yields the
/// same violation set sorted by (crate_name, kind).
///
/// # Errors
/// None — returns `(report, violations)`.
#[must_use]
pub fn audit(
    config: &DisciplineConfig,
    manifests: &[CargoManifestText],
) -> (DisciplineReport, Vec<DisciplineViolation>) {
    let mut violations: Vec<DisciplineViolation> = Vec::new();
    for m in manifests {
        // Cheap substring detection. The advisory gate is meant to
        // catch direct `[dependencies]` table entries — a real
        // parser is a follow-up. Substring is the established
        // pattern across our existing check crates.
        let mentions_kernel = m.contents.contains(KERNEL_CRATE);
        for forbidden in FORBIDDEN_RUNTIME_CRATES {
            // Match a `[dependencies]` style entry — the crate name
            // followed by a space, `=`, or quoting character.
            let entry_marker = format!("{forbidden} =");
            let table_marker = format!("\"{forbidden}\"");
            if m.contents.contains(&entry_marker) || m.contents.contains(&table_marker) {
                let allowed = config.adapter_allowlist.iter().any(|n| n == &m.crate_name);
                if !allowed {
                    violations.push(DisciplineViolation {
                        crate_name: m.crate_name.clone(),
                        manifest_path: m.path.clone(),
                        kind: if m.crate_name == KERNEL_CRATE {
                            ViolationKind::KernelCrateImportsForbiddenAdapterOutsideAllowlist
                        } else {
                            ViolationKind::DirectRuntimeImport
                        },
                        summary: format!(
                            "{} imports forbidden runtime crate `{}`; route through `{}`",
                            m.crate_name, forbidden, KERNEL_CRATE
                        ),
                    });
                }
            }
        }
        // Intent markers — crate hints it does WASM but doesn't
        // depend on the kernel.
        if !mentions_kernel
            && m.crate_name != KERNEL_CRATE
            && WASM_INTENT_MARKERS
                .iter()
                .any(|marker| m.contents.contains(marker))
        {
            violations.push(DisciplineViolation {
                crate_name: m.crate_name.clone(),
                manifest_path: m.path.clone(),
                kind: ViolationKind::KernelMissing,
                summary: format!(
                    "{} signals WASM execution intent but does not depend on `{}`",
                    m.crate_name, KERNEL_CRATE
                ),
            });
        }
    }
    violations.sort_by(|a, b| {
        a.crate_name
            .cmp(&b.crate_name)
            .then(a.kind.cmp(&b.kind))
            .then(a.manifest_path.cmp(&b.manifest_path))
    });
    (
        DisciplineReport {
            manifests_checked: manifests.len(),
        },
        violations,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mf(name: &str, contents: &str) -> CargoManifestText {
        CargoManifestText {
            crate_name: name.into(),
            path: format!("crates/{name}/Cargo.toml"),
            contents: contents.into(),
        }
    }

    #[test]
    fn clean_workspace_has_no_violations() {
        let cfg = DisciplineConfig::canonical();
        let manifests = vec![
            mf(
                "oya-intelligence-tool-runner",
                "[dependencies]\noya-shared-wasm-runtime-kernel = { path = \"../oya-shared-wasm-runtime-kernel\" }",
            ),
            mf(
                "oya-shared-wasm-runtime-kernel",
                "[dependencies]\n# trait-only, no deps",
            ),
        ];
        let (rep, viols) = audit(&cfg, &manifests);
        assert_eq!(rep.manifests_checked, 2);
        assert!(viols.is_empty(), "expected no violations, got {viols:?}");
    }

    #[test]
    fn direct_wasmtime_import_outside_allowlist_is_flagged() {
        let cfg = DisciplineConfig::canonical();
        let manifests = vec![mf(
            "oya-rogue-microservice",
            "[dependencies]\nwasmtime = \"30\"\n",
        )];
        let (_, viols) = audit(&cfg, &manifests);
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].kind, ViolationKind::DirectRuntimeImport);
        assert_eq!(viols[0].crate_name, "oya-rogue-microservice");
    }

    #[test]
    fn direct_wasmer_import_is_flagged() {
        let cfg = DisciplineConfig::canonical();
        let manifests = vec![mf("oya-rogue-wasmer", "[dependencies]\nwasmer = \"4\"\n")];
        let (_, viols) = audit(&cfg, &manifests);
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].kind, ViolationKind::DirectRuntimeImport);
    }

    #[test]
    fn allowlisted_adapter_may_import_wasmtime() {
        let cfg = DisciplineConfig::canonical();
        let manifests = vec![mf(
            "oya-shared-wasm-runtime-kernel-adapter-wasmtime",
            "[dependencies]\nwasmtime = \"30\"\n",
        )];
        let (_, viols) = audit(&cfg, &manifests);
        assert!(
            viols.is_empty(),
            "allowlisted adapter must pass; got {viols:?}"
        );
    }

    #[test]
    fn intent_marker_without_kernel_dep_is_flagged() {
        let cfg = DisciplineConfig::canonical();
        let manifests = vec![mf(
            "oya-some-microservice",
            "# wasm-execute path here\n[dependencies]\nserde = \"1\"\n",
        )];
        let (_, viols) = audit(&cfg, &manifests);
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].kind, ViolationKind::KernelMissing);
    }

    #[test]
    fn kernel_crate_self_reference_is_not_flagged() {
        let cfg = DisciplineConfig::canonical();
        let manifests = vec![mf(
            "oya-shared-wasm-runtime-kernel",
            "[dependencies]\n# wasm-execute substrate\n",
        )];
        let (_, viols) = audit(&cfg, &manifests);
        assert!(
            viols.is_empty(),
            "kernel itself must not be flagged for KernelMissing"
        );
    }

    #[test]
    fn multiple_violations_are_sorted_by_crate_then_kind() {
        let cfg = DisciplineConfig::canonical();
        let manifests = vec![
            mf("zzz-late", "[dependencies]\nwasmtime = \"30\"\n"),
            mf("aaa-early", "[dependencies]\nwasmer = \"4\"\n"),
        ];
        let (_, viols) = audit(&cfg, &manifests);
        assert_eq!(viols.len(), 2);
        assert_eq!(viols[0].crate_name, "aaa-early");
        assert_eq!(viols[1].crate_name, "zzz-late");
    }
}
