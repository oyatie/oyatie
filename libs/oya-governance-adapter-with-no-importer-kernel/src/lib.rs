//! Foundry adapter-with-no-importer fitness kernel.
//!
//! # Naming justification
//!
//! - Crate `oya-governance-adapter-with-no-importer-kernel` —
//!   v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:adapter-with-no-importer>-<layer:kernel>`;
//!   12-layer-enum suffix `kernel` (innermost ring: I/O-free port + pure check
//!   functions per ADR-0056 "port-in-kernel").
//! - Dev-CLI `oya-governance-adapter-with-no-importer-app` —
//!   v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:adapter-with-no-importer>-<layer:app>`;
//!   binary tool surface (canonical `app` suffix per ADR-0105 amendment 2026-05-15),
//!   wraps the kernel for `oya gate validate`.
//!
//! # Intent
//!
//! The check (ADR-0104 follow-up #4) detects the anti-pattern surfaced in
//! audit #7: an `*-adapter-*` crate that has no `*-importer-*` consumer.
//! Adapters exist to be imported. An adapter crate with no importer is a
//! premature-crate-shell (the ADR-0104 failure mode).
//!
//! # Algorithm (kernel — I/O-free)
//!
//! Runners enumerate workspace crate names and pass them as
//! [`WorkspaceCrate`] records into [`check`]. The kernel:
//!
//! 1. Collects all `*-adapter-*` and `*-adapter` crates (excluding kernels
//!    and packages whose names match the importer pattern themselves).
//! 2. For each adapter crate `<base>-adapter[-<variant>]`, derives the
//!    importer expectation: `<base>-importer*`.
//! 3. Emits a [`Violation`] for any adapter whose derived importer pattern
//!    has no match in the workspace.
//!
//! The check is intentionally pure: filesystem walking, `Cargo.toml`
//! parsing, and exit-code mapping live in the dev-CLI runner.

#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceCrate {
    pub name: String,          // data_class: INTERNAL_ONLY
    pub manifest_path: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    pub adapter_crate: String,             // data_class: INTERNAL_ONLY
    pub expected_importer_pattern: String, // data_class: INTERNAL_ONLY
    pub hint: String,                      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterImporterReport {
    pub adapters_checked: usize,    // data_class: INTERNAL_ONLY
    pub importers_observed: usize,  // data_class: INTERNAL_ONLY
    pub violations: Vec<Violation>, // data_class: INTERNAL_ONLY
}

impl AdapterImporterReport {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Layer suffixes from the 12-layer enum (ADR-0056). A crate name whose
/// final `-`-separated segment is one of these is a layer crate
/// (kernel/domain/api/...), not a concrete adapter variant. Used so the
/// check ignores port-declaring siblings (e.g. `oya-intelligence-adapter-anthropic-api-kernel`).
const LAYER_SUFFIXES: &[&str] = &[
    "kernel",
    "domain",
    "usecase",
    "api",
    "app",
    "infrastructure",
    "rest",
    "cli",
    "gate",
];

/// Returns true for concrete adapter crates. The rule: the crate's layer
/// segment (the final `-`-separated token) is exactly `adapter`, OR the
/// crate matches `<base>-adapter-<variant>` where `<base>-adapter` is the
/// layer-suffixed prefix and `<variant>` is a single token (no further
/// dashes) that is NOT itself a 12-layer suffix.
///
/// Excludes port-declaring siblings like `oya-intelligence-adapter-anthropic-api-kernel`
/// (last segment = `kernel`, not adapter) and the fitness kernel itself
/// (`oya-governance-adapter-with-no-importer-kernel` — last segment
/// `kernel`).
fn is_adapter_crate(name: &str) -> bool {
    let Some(last_dash) = name.rfind('-') else {
        return false;
    };
    let last_segment = &name[last_dash + 1..];
    if last_segment == "adapter" {
        return true;
    }
    // Possible `<base>-adapter-<variant>` form: peel off the last segment
    // and check the resulting prefix ends with `-adapter`.
    if LAYER_SUFFIXES.contains(&last_segment) {
        return false;
    }
    let prefix = &name[..last_dash];
    prefix.ends_with("-adapter")
}

fn is_importer_crate(name: &str) -> bool {
    name.contains("-importer-") || name.ends_with("-importer")
}

/// Derives the base prefix from an adapter crate name by stripping the
/// `-adapter` segment (and any variant suffix that follows).
///
/// Examples:
/// - `oya-intelligence-account-adapter-inmemory` -> `oya-intelligence-account`
/// - `oya-cloud-billing-adapter-aws` -> `oya-cloud-billing`
/// - `oya-intelligence-claude-account-adapter` -> `oya-intelligence-claude-account`
fn adapter_base(name: &str) -> Option<String> {
    if let Some(prefix) = name.strip_suffix("-adapter") {
        return Some(prefix.to_string());
    }
    // Match the last `-adapter-` occurrence so multi-segment bases stay intact.
    if let Some(index) = name.rfind("-adapter-") {
        return Some(name[..index].to_string());
    }
    None
}

/// Expected importer pattern for a given adapter base.
fn expected_importer_pattern(base: &str) -> String {
    format!("{base}-importer*")
}

/// Returns true if `importer_name` matches the `<base>-importer*` pattern.
fn importer_matches(base: &str, importer_name: &str) -> bool {
    let prefix = format!("{base}-importer");
    importer_name == prefix
        || importer_name
            .strip_prefix(&prefix)
            .is_some_and(|tail| tail.is_empty() || tail.starts_with('-'))
}

pub fn check(workspace: &[WorkspaceCrate]) -> AdapterImporterReport {
    let importer_names: Vec<&str> = workspace
        .iter()
        .map(|krate| krate.name.as_str())
        .filter(|name| is_importer_crate(name))
        .collect();

    let mut violations = Vec::new();
    let mut adapters_checked = 0usize;

    for krate in workspace {
        if !is_adapter_crate(&krate.name) {
            continue;
        }
        adapters_checked += 1;

        let Some(base) = adapter_base(&krate.name) else {
            continue;
        };

        let has_importer = importer_names
            .iter()
            .any(|importer| importer_matches(&base, importer));

        if !has_importer {
            violations.push(Violation {
                adapter_crate: krate.name.clone(),
                expected_importer_pattern: expected_importer_pattern(&base),
                hint: format!(
                    "no `{base}-importer*` consumer in workspace; either ship the importer in the same PR or remove the adapter shell (ADR-0104 ecosystem-expansion rule)"
                ),
            });
        }
    }

    AdapterImporterReport {
        adapters_checked,
        importers_observed: importer_names.len(),
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn krate(name: &str) -> WorkspaceCrate {
        WorkspaceCrate {
            name: name.into(),
            manifest_path: format!("crates/{name}/Cargo.toml"),
        }
    }

    #[test]
    fn accepts_adapter_with_matching_importer() {
        let report = check(&[
            krate("oya-intelligence-claude-account-adapter"),
            krate("oya-intelligence-claude-account-importer-supervisor"),
        ]);
        assert!(report.is_clean(), "expected no violations, got {report:?}");
        assert_eq!(report.adapters_checked, 1);
        assert_eq!(report.importers_observed, 1);
    }

    #[test]
    fn flags_adapter_with_no_importer() {
        let report = check(&[krate("oya-cloud-billing-adapter-aws")]);
        assert!(!report.is_clean());
        assert_eq!(report.violations.len(), 1);
        let violation = &report.violations[0];
        assert_eq!(violation.adapter_crate, "oya-cloud-billing-adapter-aws");
        assert_eq!(
            violation.expected_importer_pattern,
            "oya-cloud-billing-importer*"
        );
        assert!(violation.hint.contains("ADR-0104"));
    }

    #[test]
    fn variant_suffixed_adapter_matches_base_importer() {
        let report = check(&[
            krate("oya-intelligence-account-adapter-inmemory"),
            krate("oya-intelligence-account-importer-supervisor"),
        ]);
        assert!(report.is_clean(), "{report:?}");
    }

    #[test]
    fn unrelated_importer_does_not_satisfy() {
        let report = check(&[
            krate("oya-cloud-billing-adapter-aws"),
            krate("oya-cloud-marketplace-importer-supervisor"),
        ]);
        assert!(!report.is_clean());
        assert_eq!(report.violations.len(), 1);
    }

    #[test]
    fn ignores_adapter_kernel_crates() {
        // `*-adapter-kernel` declares the port and is imported by both adapter
        // and importer; only the concrete adapter crate is checked.
        let report = check(&[krate("oya-intelligence-adapter-anthropic-api-kernel")]);
        assert!(report.is_clean());
        assert_eq!(report.adapters_checked, 0);
    }

    #[test]
    fn ignores_non_adapter_crates() {
        let report = check(&[
            krate("intelligence-supervisor-kernel"),
            krate("intelligence-supervisor-app"),
        ]);
        assert!(report.is_clean());
        assert_eq!(report.adapters_checked, 0);
        assert_eq!(report.importers_observed, 0);
    }

    #[test]
    fn empty_workspace_is_clean() {
        let report = check(&[]);
        assert!(report.is_clean());
        assert_eq!(report.adapters_checked, 0);
    }

    #[test]
    fn detects_violation_against_temp_crate_layout() {
        // Fixture test: build a fake workspace on disk, build WorkspaceCrate
        // records from the directory listing, and assert the kernel catches
        // the orphan adapter. The kernel itself is I/O-free; this exercises
        // the value-object boundary against a real layout.
        let tmp = TempDir::new().expect("tempdir");
        let root = tmp.path().join("crates");
        fs::create_dir_all(&root).expect("create crates root");
        for name in [
            "oya-cloud-billing-adapter-aws",           // orphan: no importer
            "oya-intelligence-claude-account-adapter", // matched by importer below
            "oya-intelligence-claude-account-importer-supervisor",
            "intelligence-supervisor-kernel", // ignored: not an adapter
        ] {
            let dir = root.join(name);
            fs::create_dir(&dir).expect("mkdir crate");
            fs::write(
                dir.join("Cargo.toml"),
                format!("[package]\nname = \"{name}\"\n"),
            )
            .expect("write Cargo.toml");
        }

        let mut workspace: Vec<WorkspaceCrate> = fs::read_dir(&root)
            .expect("read crates")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                let manifest = entry.path().join("Cargo.toml");
                WorkspaceCrate {
                    name,
                    manifest_path: manifest.to_string_lossy().into_owned(),
                }
            })
            .collect();
        workspace.sort_by(|a, b| a.name.cmp(&b.name));

        let report = check(&workspace);
        assert_eq!(report.adapters_checked, 2);
        assert_eq!(report.importers_observed, 1);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0].adapter_crate,
            "oya-cloud-billing-adapter-aws"
        );
    }
}
