//! D-8: a repo-root name exists only if an engine consumes it (or it is
//! OWNERS/README/BUCK). Unknown top-level names are RED. Absence of a listed
//! name is not RED (BUILD vs DONE).

/// Names allowed at the repository root. Not a census: new names fail; missing
/// allowed names do not.
pub const ALLOWED_ROOT_DIRS: &[&str] = &[
    "app",
    "audit",
    "billing",
    "build",
    "bus",
    "cell",
    "compliance",
    "compute",
    "contracts",
    "data",
    "docs",
    "flags",
    "gateway",
    "iac",
    "iam",
    "infra",
    "intelligence",
    "k8s",
    "kernel",
    "libs",
    "marketplace",
    "network",
    "observability",
    "os",
    "packs",
    "pipeline",
    "plan",
    "scripts",
    "secrets",
    "storage",
    "tasks",
    "templates",
    "tenancy",
    "third-party",
    "tools",
];

#[cfg(test)]
mod tests {
    use super::ALLOWED_ROOT_DIRS;
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
}
