use std::collections::{BTreeSet, HashSet};
use std::path::Path;

use toml::Value;

use crate::CONFIG_PATH;
use crate::report::Finding;

/// Declared PATHS must point at something that exists.
///
/// `managed_file` entries were already existence-checked; single-valued path keys were not, and
/// that gap let `[rust].drift_guard` sit for months naming
/// `cloud/cloud-ci/gates/oya-cloud-ci-freshness-app/src/rust_toolchain_drift.rs` — a file deleted
/// with the whole `cloud/` tree. A config that names a nonexistent guard reads exactly like a
/// config that names a working one, which is the failure mode worth closing: the declaration is
/// the only evidence the guard exists, so an unchecked declaration is no evidence at all.
pub(crate) fn validate_declared_paths(
    root: &Path,
    config: &Value,
    findings: &mut BTreeSet<Finding>,
) {
    // `freshness.kernel` belongs here for the same reason as the rest: it is a path-valued
    // declaration naming an executable freshness artifact, so omitting it left exactly the
    // stale-path failure this validator exists to close open on another file.
    const DECLARED_PATHS: &[(&str, &str)] = &[
        ("rust", "drift_guard"),
        ("supply_chain", "license_policy"),
        ("supply_chain", "stewardship_registry"),
        ("freshness", "mirror"),
        ("freshness", "manifest"),
        ("freshness", "kernel"),
    ];
    for (table, key) in DECLARED_PATHS {
        let location = format!("{CONFIG_PATH}:{table}.{key}");
        // Absence is a FINDING, not a skip. Continuing here meant deleting the key — or giving it
        // a non-string value — silently disabled the check, and no closed-schema or policy rule
        // requires any of these keys, so the gate could report green with nothing declared at all.
        let Some(raw) = config.get(table).and_then(|t| t.get(key)) else {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-KEY",
                location,
                format!("{table}.{key} is required; the declaration is the only evidence the referenced artifact exists"),
            ));
            continue;
        };
        let Some(value) = raw.as_str() else {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-KEY",
                location,
                format!("{table}.{key} must be a string path"),
            ));
            continue;
        };
        // Same containment rule the sibling managed_file validator already applies. Without it,
        // `root.join(value)` on an absolute path discards `root` entirely, so declaring
        // `/etc/passwd` — or a `..` path that happens to exist on the runner — satisfied the
        // existence check using state outside the candidate tree.
        if value.starts_with('/') || value.contains("..") {
            findings.insert(Finding::new(
                "DEP-AUTO-BAD-DECLARED-PATH",
                location,
                format!("declares {value}; declared paths must be repo-relative and must not contain '..'"),
            ));
            continue;
        }
        // `is_file`, not `exists`: a directory is not the executable artifact being declared.
        if !root.join(value).is_file() {
            findings.insert(Finding::new(
                "DEP-AUTO-DECLARED-PATH-MISSING",
                location,
                format!(
                    "declares {value}, which is not a file in this tree; a declaration is the \
                     only evidence the referenced artifact is real"
                ),
            ));
        }
    }
}

pub(crate) fn validate_managed_files(
    root: &Path,
    config: &Value,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(entries) = config.get("managed_file").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "DEP-AUTO-MISSING-KEY",
            format!("{CONFIG_PATH}:managed_file"),
            "at least one managed_file entry is required",
        ));
        return;
    };
    if entries.is_empty() {
        findings.insert(Finding::new(
            "DEP-AUTO-MISSING-KEY",
            format!("{CONFIG_PATH}:managed_file"),
            "managed_file must not be empty",
        ));
        return;
    }

    let mut seen = HashSet::new();
    for (idx, entry) in entries.iter().enumerate() {
        let Some(path) = entry.get("path").and_then(Value::as_str) else {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-KEY",
                format!("{CONFIG_PATH}:managed_file[{idx}].path"),
                "managed_file entries require a path",
            ));
            continue;
        };
        if path.starts_with('/') || path.contains("..") {
            findings.insert(Finding::new(
                "DEP-AUTO-BAD-MANAGED-PATH",
                format!("{CONFIG_PATH}:managed_file[{idx}].path"),
                "managed paths must be repo-relative and must not contain '..'",
            ));
            continue;
        }
        if !seen.insert(path.to_owned()) {
            findings.insert(Finding::new(
                "DEP-AUTO-DUPLICATE-MANAGED-PATH",
                path,
                "managed_file paths must be unique",
            ));
        }
        if !root.join(path).is_file() {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-MANAGED-FILE",
                path,
                "managed_file path does not exist in the candidate tree",
            ));
        }
    }

    for required in [
        "rust-toolchain.toml",
        "Cargo.toml",
        "build/images/Dockerfile.distroless",
    ] {
        if !seen.contains(required) {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-MANAGED-FILE",
                required,
                "required Rust pin surface is not listed in managed_file",
            ));
        }
    }
}
