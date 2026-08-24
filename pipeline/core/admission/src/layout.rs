//! ADR-0719 D-8 changed-path layout admission.
//!
//! Unknown top-level names are red. Absence of a listed name is not red
//! (BUILD vs DONE). Capability and `app/<product>/` children are hexagonal
//! faces.

use std::collections::BTreeSet;

use crate::GitChangePaths;

/// Admitted root directories that are present on `dev` and therefore require
/// OWNERS/CODEOWNERS coverage.
pub const ALLOWED_ROOT_DIRS: &[&str] = &[
    "app",
    "audit",
    "billing",
    "build",
    "bus",
    "cell",
    "compliance",
    "compute",
    "data",
    "docs",
    "flags",
    "gateway",
    "iac",
    "iam",
    "intelligence",
    "k8s",
    "marketplace",
    "network",
    "observability",
    "packs",
    "pipeline",
    "secrets",
    "storage",
    "templates",
    "tenancy",
    "third-party",
];

/// ADR-0719 D-19 DO + HAVE NOT roots. They are valid only once a real face is
/// added; absence must not require an empty OWNERS scaffold.
pub const BUILD_ROOT_DIRS: &[&str] = &["notify", "policy", "workflow"];

pub const ALLOWED_ROOT_FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "LICENSE",
    "OWNERS",
    "AGENTS.md",
    "CLAUDE.md",
    "rust-toolchain.toml",
    "rustfmt.toml",
    "deny.toml",
    "reindeer.toml",
];

pub const FACES: &[&str] = &["core", "ports", "adapters", "facade"];

pub const CAP_EXTRAS: &[&str] = &["cedar", "observability", "iac", "docs"];

pub const FORBIDDEN_NAMES: &[&str] = &[
    "plan",
    "tasks",
    "contracts",
    "specs",
    "libs",
    "tools",
    "infra",
    "kernel",
    "os",
    "governance",
    "console",
    "tests",
    "e2e",
];

pub const META_ROOTS: &[&str] = &["app", "build", "docs", "templates", "third-party"];

pub(crate) fn path_parts(path: &str) -> Vec<&str> {
    path.trim_start_matches("./")
        .split(['/', '\\'])
        .filter(|part| !part.is_empty())
        .collect()
}

fn is_meta_root(root: &str) -> bool {
    META_ROOTS.contains(&root)
}

pub fn is_capability_root(root: &str) -> bool {
    (ALLOWED_ROOT_DIRS.contains(&root) || BUILD_ROOT_DIRS.contains(&root)) && !is_meta_root(root)
}

pub fn cap_root_file_ok(name: &str) -> bool {
    matches!(
        name,
        "OWNERS"
            | "README.md"
            | "BUCK"
            | "ADR.md"
            | "PRD.md"
            | "SPEC.md"
            | "PLAN.md"
            | "Cargo.toml"
            | "Cargo.lock"
            | "LICENSE"
    )
}

pub fn face_dir_ok(child: &str) -> bool {
    FACES.contains(&child) || CAP_EXTRAS.contains(&child)
}

/// Report violations on live changed paths. A current-tree walk is separate
/// reorganization follow-through; this is the born-blocking pattern engine.
pub fn layout_violations(changed_files: &[String]) -> Vec<String> {
    let allowed_dirs: BTreeSet<&str> = ALLOWED_ROOT_DIRS
        .iter()
        .chain(BUILD_ROOT_DIRS)
        .copied()
        .collect();
    let allowed_files: BTreeSet<&str> = ALLOWED_ROOT_FILES.iter().copied().collect();
    let forbidden: BTreeSet<&str> = FORBIDDEN_NAMES.iter().copied().collect();
    let mut violations = Vec::new();
    for file in changed_files {
        let parts = path_parts(file);
        let Some(root) = parts.first().copied() else {
            continue;
        };
        if root.starts_with('.') || root == "target" {
            continue;
        }
        if forbidden.contains(root) {
            violations.push(format!("{file}: forbidden root `{root}`"));
            continue;
        }
        if parts.len() == 1 {
            if !allowed_files.contains(root) {
                violations.push(format!("{file}: unknown root file `{root}`"));
            }
            continue;
        }
        if !allowed_dirs.contains(root) {
            violations.push(format!("{file}: unknown root `{root}`"));
            continue;
        }
        if root == "app" {
            validate_app_path(file, &parts, &forbidden, &mut violations);
        } else if is_capability_root(root) {
            validate_capability_path(file, &parts, &forbidden, &mut violations);
        }
    }
    violations
}

fn validate_app_path(
    file: &str,
    parts: &[&str],
    forbidden: &BTreeSet<&str>,
    violations: &mut Vec<String>,
) {
    if parts.len() == 2 {
        if !cap_root_file_ok(parts[1]) {
            violations.push(format!("{file}: app-root file `{}` not allowed", parts[1]));
        }
        return;
    }
    let child = parts[2];
    if forbidden.contains(child) {
        violations.push(format!("{file}: forbidden child `{child}`"));
    } else if parts.len() == 3 {
        if face_dir_ok(child) {
            violations.push(format!(
                "{file}: `app/<product>/{child}` must be a directory"
            ));
        } else if !cap_root_file_ok(child) {
            violations.push(format!("{file}: `app/<product>/{child}` is not a face"));
        }
    } else if !face_dir_ok(child) {
        violations.push(format!("{file}: `app/<product>/{child}` is not a face"));
    }
}

fn validate_capability_path(
    file: &str,
    parts: &[&str],
    forbidden: &BTreeSet<&str>,
    violations: &mut Vec<String>,
) {
    let child = parts[1];
    if forbidden.contains(child) {
        violations.push(format!("{file}: forbidden child `{child}`"));
    } else if parts.len() == 2 {
        if face_dir_ok(child) {
            violations.push(format!(
                "{file}: `{}/{child}` must be a directory",
                parts[0]
            ));
        } else if !cap_root_file_ok(child) {
            violations.push(format!("{file}: `{}/{child}` is not a face", parts[0]));
        }
    } else if !face_dir_ok(child) {
        violations.push(format!("{file}: `{}/{child}` is not a face", parts[0]));
    }
}

/// Apply D-8 only to changed paths that remain after the Git diff.
pub fn changed_layout_violations(changes: &GitChangePaths) -> Vec<String> {
    layout_violations(
        &changes
            .layout_candidates
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
    )
}
