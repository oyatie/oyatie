//! D-8: a repo-root name exists only if an engine consumes it (or it is
//! OWNERS/README/BUCK). Unknown top-level names are RED. Absence of a listed
//! name is not RED (BUILD vs DONE). Capability and `app/<product>/` children
//! are hexagonal faces.

use std::collections::BTreeSet;

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
];

const META_ROOTS: &[&str] = &[
    "app",
    "build",
    "docs",
    "iac",
    "observability",
    "templates",
    "third-party",
];

fn path_parts(path: &str) -> Vec<&str> {
    path.trim_start_matches("./")
        .split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .collect()
}

fn is_meta_root(root: &str) -> bool {
    META_ROOTS.contains(&root)
}

pub fn is_capability_root(root: &str) -> bool {
    ALLOWED_ROOT_DIRS.contains(&root) && !is_meta_root(root)
}

pub fn cap_root_file_ok(name: &str) -> bool {
    matches!(
        name,
        "OWNERS" | "README.md" | "BUCK" | "PRD.md" | "Cargo.toml" | "Cargo.lock" | "LICENSE"
    )
}

pub fn face_dir_ok(child: &str) -> bool {
    FACES.contains(&child) || CAP_EXTRAS.contains(&child)
}

/// Violations on a path list (PR files or synthetic). Disk walk of existing
/// cap children is a reorg follow-through; this is the pattern engine.
pub fn layout_violations(changed_files: &[String]) -> Vec<String> {
    let allowed_dirs: BTreeSet<&str> = ALLOWED_ROOT_DIRS.iter().copied().collect();
    let allowed_files: BTreeSet<&str> = ALLOWED_ROOT_FILES.iter().copied().collect();
    let forbidden: BTreeSet<&str> = FORBIDDEN_NAMES.iter().copied().collect();
    let mut out = Vec::new();
    for file in changed_files {
        let parts = path_parts(file);
        let Some(root) = parts.first().copied() else {
            continue;
        };
        if root.starts_with('.') || root == "target" {
            continue;
        }
        if forbidden.contains(root) {
            out.push(format!("{file}: forbidden root `{root}`"));
            continue;
        }
        if parts.len() == 1 {
            if !allowed_files.contains(root) && !allowed_dirs.contains(root) {
                out.push(format!("{file}: unknown root file `{root}`"));
            }
            continue;
        }
        if !allowed_dirs.contains(root) {
            out.push(format!("{file}: unknown root `{root}`"));
            continue;
        }
        if root == "app" {
            if parts.len() == 2 {
                if !cap_root_file_ok(parts[1]) {
                    out.push(format!("{file}: app-root file `{}` not allowed", parts[1]));
                }
                continue;
            }
            let child = parts[2];
            if forbidden.contains(child) {
                out.push(format!("{file}: forbidden child `{child}`"));
            } else if parts.len() == 3 {
                if !face_dir_ok(child) && !cap_root_file_ok(child) {
                    out.push(format!("{file}: `app/<product>/{child}` is not a face"));
                }
            } else if !face_dir_ok(child) {
                out.push(format!("{file}: `app/<product>/{child}` is not a face"));
            }
        } else if is_capability_root(root) {
            let child = parts[1];
            if forbidden.contains(child) {
                out.push(format!("{file}: forbidden child `{child}`"));
            } else if parts.len() == 2 {
                if !face_dir_ok(child) && !cap_root_file_ok(child) {
                    out.push(format!("{file}: `{root}/{child}` is not a face"));
                }
            } else if !face_dir_ok(child) {
                out.push(format!("{file}: `{root}/{child}` is not a face"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::Path;

    fn repo_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root")
    }

    #[test]
    fn unknown_root_dir_is_red() {
        let allowed: BTreeSet<&str> = ALLOWED_ROOT_DIRS.iter().copied().collect();
        let mut unknown = Vec::new();
        for entry in std::fs::read_dir(repo_root()).expect("read root") {
            let entry = entry.expect("entry");
            if !entry.file_type().expect("ft").is_dir() {
                continue;
            }
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') || name == "target" {
                continue;
            }
            if !allowed.contains(name.as_ref()) {
                unknown.push(name.into_owned());
            }
        }
        assert!(
            unknown.is_empty(),
            "D-8 unknown root names (not in ALLOWED_ROOT_DIRS): {unknown:?}"
        );
    }

    #[test]
    fn layout_engine_rejects_dump_and_accepts_faces() {
        let v = layout_violations(&[
            "plan/foo.md".into(),
            "libs/x.rs".into(),
            "storage/src/lib.rs".into(),
            "storage/core/journal.rs".into(),
            "app/foundry/ports/blob.rs".into(),
            "docs/decisions/ADR.md".into(),
        ]);
        assert!(v.iter().any(|s| s.contains("plan")));
        assert!(v.iter().any(|s| s.contains("libs")));
        assert!(v.iter().any(|s| s.contains("storage/src")));
        assert!(!v.iter().any(|s| s.contains("storage/core")));
        assert!(!v.iter().any(|s| s.contains("foundry/ports")));
        assert!(!v.iter().any(|s| s.contains("docs/decisions")));
    }
}
