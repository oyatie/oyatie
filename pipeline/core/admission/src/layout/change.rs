//! Cross-path completeness checks for new ADR-0719 owners.

use std::collections::BTreeSet;

use crate::GitChangePaths;

use super::{
    ALLOWED_ROOT_DIRS, APP_PRODUCT_DIRS, BUILD_ROOT_DIRS, is_capability_root, layout_violations,
};

const OWNER_LAW_FILES: &[&str] = &["ADR.md", "PRD.md", "SPEC.md", "PLAN.md"];

/// Apply D-8 only to changed paths that remain after the Git diff. A new owner
/// must land as an implemented unit, never as paperwork or an unbuilt source
/// dump. Capability and product owners also land their four D-36 law files.
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
    if owner_is_new_and_touched("base", changes, existing_owner_dirs) {
        require_core_crate("base", "BUILD root", changes, &mut violations);
    }
    for root in ALLOWED_ROOT_DIRS
        .iter()
        .chain(BUILD_ROOT_DIRS)
        .copied()
        .filter(|root| is_capability_root(root))
    {
        if owner_is_new_and_touched(root, changes, existing_owner_dirs) {
            require_core_crate(root, "capability owner", changes, &mut violations);
            require_owner_law(root, changes, &mut violations);
        }
    }
    for product in APP_PRODUCT_DIRS {
        let owner = format!("app/{product}");
        if owner_is_new_and_touched(&owner, changes, existing_owner_dirs) {
            require_core_crate(&owner, "BUILD product", changes, &mut violations);
            require_owner_law(&owner, changes, &mut violations);
        }
    }
    violations
}

/// A conditional owner that had a complete core at the merge base may be
/// removed as a unit, but it cannot survive as paperwork-only scaffolding.
pub fn owner_core_regression_violations(
    changes: &GitChangePaths,
    complete_before: &BTreeSet<String>,
    live_after: &BTreeSet<String>,
    complete_after: &BTreeSet<String>,
) -> Vec<String> {
    complete_before
        .iter()
        .filter(|owner| owner_touched(owner, changes))
        .filter(|owner| live_after.contains(*owner) && !complete_after.contains(*owner))
        .map(|owner| {
            format!(
                "{owner}: deleting the last complete core crate while retaining the owner is forbidden"
            )
        })
        .collect()
}

fn owner_is_new_and_touched(
    owner: &str,
    changes: &GitChangePaths,
    existing_owner_dirs: &BTreeSet<String>,
) -> bool {
    !existing_owner_dirs.contains(owner) && owner_touched(owner, changes)
}

fn owner_touched(owner: &str, changes: &GitChangePaths) -> bool {
    changes
        .occupied
        .iter()
        .any(|path| path == owner || path.starts_with(&format!("{owner}/")))
}

fn require_core_crate(
    owner: &str,
    kind: &str,
    changes: &GitChangePaths,
    violations: &mut Vec<String>,
) {
    let prefix = format!("{owner}/core/");
    let has_crate = changes.layout_candidates.iter().any(|path| {
        let Some(crate_name) = path
            .strip_prefix(&prefix)
            .and_then(|rest| rest.strip_suffix("/src/lib.rs"))
        else {
            return false;
        };
        !crate_name.is_empty()
            && !crate_name.contains('/')
            && changes
                .layout_candidates
                .contains(&format!("{prefix}{crate_name}/Cargo.toml"))
    });
    if !has_crate {
        violations.push(format!(
            "{owner}: new {kind} requires one core crate with `Cargo.toml` and `src/lib.rs` in the same change"
        ));
    }
}

fn require_owner_law(owner: &str, changes: &GitChangePaths, violations: &mut Vec<String>) {
    let missing: Vec<&str> = OWNER_LAW_FILES
        .iter()
        .copied()
        .filter(|name| {
            !changes
                .layout_candidates
                .contains(&format!("{owner}/{name}"))
        })
        .collect();
    if !missing.is_empty() {
        violations.push(format!(
            "{owner}: new owner requires D-36 law files in the same change; missing {}",
            missing.join(", ")
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_owner_cannot_lose_its_last_core_crate() {
        let changes = crate::git_change_paths_from_name_status_z(
            b"D\0policy/core/evaluate/Cargo.toml\0D\0policy/core/evaluate/src/lib.rs\0",
        )
        .unwrap();
        let complete_before = ["policy".to_owned()].into();
        let live_after = ["policy".to_owned()].into();
        assert!(
            !owner_core_regression_violations(
                &changes,
                &complete_before,
                &live_after,
                &BTreeSet::new(),
            )
            .is_empty()
        );
        assert!(
            owner_core_regression_violations(
                &changes,
                &complete_before,
                &BTreeSet::new(),
                &BTreeSet::new(),
            )
            .is_empty()
        );
    }
}
