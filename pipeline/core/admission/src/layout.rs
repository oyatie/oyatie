//! ADR-0719 D-8 changed-path layout admission.
//!
//! Unknown top-level names are red. Absence of a listed name is not red
//! (BUILD vs DONE). Capability and `app/<product>/` children are hexagonal
//! faces.

use std::collections::BTreeSet;

use crate::GitChangePaths;

mod inner;
mod manifest;

use inner::validate_owner_path;
pub use manifest::cargo_manifest_violations;

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

/// Conditional roots that are valid only once a real core lands. `base` also
/// requires the ADR-0719 three-capability review; absence never requires an
/// empty OWNERS scaffold.
pub const BUILD_ROOT_DIRS: &[&str] = &["base", "notify", "policy", "workflow"];

/// ADR-0719 D-22's closed v1 product roster. Missing products are BUILD, not
/// membership ghosts, and must arrive with implementation content.
pub const APP_PRODUCT_DIRS: &[&str] = &[
    "accounting",
    "application",
    "calendar",
    "community",
    "drive",
    "foundry",
    "hr",
    "ledger",
    "mail",
    "messenger",
    "payments",
    "payroll",
];

/// Closed dot-directory set. Existing dot-root debt is removal-only rather
/// than a wildcard exemption for new tracked paths.
pub const ALLOWED_DOT_ROOT_DIRS: &[&str] = &[".cargo", ".github"];

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
    ".buckconfig",
    ".buckroot",
    ".gitattributes",
    ".gitignore",
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
    path.split('/').filter(|part| !part.is_empty()).collect()
}

fn is_meta_root(root: &str) -> bool {
    META_ROOTS.contains(&root)
}

pub fn is_capability_root(root: &str) -> bool {
    (ALLOWED_ROOT_DIRS.contains(&root) || BUILD_ROOT_DIRS.contains(&root))
        && root != "base"
        && !is_meta_root(root)
}

pub fn cap_root_file_ok(name: &str) -> bool {
    matches!(
        name,
        "OWNERS" | "README.md" | "BUCK" | "ADR.md" | "PRD.md" | "SPEC.md" | "PLAN.md"
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
        .chain(ALLOWED_DOT_ROOT_DIRS)
        .copied()
        .collect();
    let allowed_files: BTreeSet<&str> = ALLOWED_ROOT_FILES.iter().copied().collect();
    let forbidden: BTreeSet<&str> = FORBIDDEN_NAMES.iter().copied().collect();
    let mut violations = Vec::new();
    for file in changed_files {
        if invalid_git_path(file) {
            violations.push(format!(
                "{file}: invalid Git path spelling; `/` is the only separator"
            ));
            continue;
        }
        let parts = path_parts(file);
        let Some(root) = parts.first().copied() else {
            continue;
        };
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
            validate_app_path(file, &parts, &mut violations);
        } else if root == "base" || is_capability_root(root) {
            validate_owner_path(file, &parts, 1, &mut violations);
        }
    }
    violations
}

fn invalid_git_path(path: &str) -> bool {
    path.is_empty()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
}

fn validate_app_path(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    if parts.len() == 2 {
        if !matches!(parts[1], "OWNERS" | "README.md") {
            violations.push(format!("{file}: app-root file `{}` not allowed", parts[1]));
        }
        return;
    }
    let product = parts[1];
    if !APP_PRODUCT_DIRS.contains(&product) {
        violations.push(format!("{file}: unknown app product `{product}`"));
        return;
    }
    validate_owner_path(file, parts, 2, violations);
}

/// Apply D-8 only to changed paths that remain after the Git diff. BUILD roots
/// absent at the merge base must carry a real core source in the same change.
pub fn changed_layout_violations(
    changes: &GitChangePaths,
    existing_owner_dirs: &BTreeSet<String>,
) -> Vec<String> {
    let mut violations = layout_violations(
        &changes
            .layout_candidates
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
    );
    for root in BUILD_ROOT_DIRS {
        let touches_root = changes.layout_candidates.iter().any(|path| {
            let parts = path_parts(path);
            parts.first() == Some(root)
        });
        let carries_core_source = changes.layout_candidates.iter().any(|path| {
            let parts = path_parts(path);
            parts.len() >= 5
                && parts[0] == *root
                && parts[1] == "core"
                && parts[3] == "src"
                && path.ends_with(".rs")
        });
        if touches_root && !existing_owner_dirs.contains(*root) && !carries_core_source {
            violations.push(format!(
                "{root}: new BUILD root requires a core source in the same change"
            ));
        }
    }
    for product in APP_PRODUCT_DIRS {
        let owner = format!("app/{product}");
        let touches_owner = changes.layout_candidates.iter().any(|path| {
            let parts = path_parts(path);
            parts.first() == Some(&"app") && parts.get(1) == Some(product)
        });
        let carries_core_source = changes.layout_candidates.iter().any(|path| {
            let parts = path_parts(path);
            parts.len() >= 6
                && parts[0] == "app"
                && parts[1] == *product
                && parts[2] == "core"
                && parts[4] == "src"
                && path.ends_with(".rs")
        });
        if touches_owner && !existing_owner_dirs.contains(&owner) && !carries_core_source {
            violations.push(format!(
                "{owner}: new BUILD product requires a core source in the same change"
            ));
        }
    }
    violations
}
