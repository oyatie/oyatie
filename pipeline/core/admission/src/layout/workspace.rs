//! Closed root-workspace membership policy. Provenance: ADR-0719 D-8/D-41.

use std::collections::BTreeSet;

use workspace_members_kernel::workspace_manifest_entries_from_str;

pub const WORKSPACE_MEMBER_GLOBS: &[&str] = &[
    "*/core/*",
    "*/ports/*/src/..",
    "*/adapters/*/src/..",
    "*/facade/*/src/..",
    "app/*/core/*",
    "*/ports/draft/*/src/..",
    "*/adapters/draft/*/src/..",
    "app/*/ports/*/src/..",
    "app/*/adapters/*/src/..",
    "app/*/ports/draft/*/src/..",
    "app/*/adapters/draft/*/src/..",
    "app/*/facade/*/src/..",
    "app/foundry/pages/crates/*",
    "app/foundry/grid/core/*",
    "build/port-engine/*/*",
];

pub const WORKSPACE_EXCLUDES: &[&str] = &[
    "*/ports/draft/*",
    "*/adapters/draft/*",
    "app/*/ports/draft/*",
    "app/*/adapters/draft/*",
];

const DEPENDENCY_DECLARATIONS_MEMBER: &str = "build/dependency-declarations/*/*/src/..";
const DEPENDENCY_DECLARATIONS_EXCLUDE: &str = "build/dependency-declarations/*/*";

/// Member globs mid-retirement: admitted whether present or absent.
///
/// Admission policy is compiled from the protected tree, so a candidate is
/// graded against the previous revision's list. Without this window an entry
/// could never be removed: dropping it from the manifest and the list in one
/// change still fails the trusted list that requires it.
const RETIRING_MEMBER_GLOBS: &[&str] = &["app/foundry/pages/crates/*", "app/foundry/grid/core/*"];

/// Refuse missing, added, or duplicate workspace membership policy entries.
pub fn workspace_membership_violations(contents: &str) -> Vec<String> {
    let document = match contents.parse::<toml::Value>() {
        Ok(document) => document,
        Err(error) => return vec![format!("Cargo.toml: {error}")],
    };
    let mut violations = Vec::new();
    if document.get("package").is_some() {
        violations.push(
            "Cargo.toml: the repository workspace root must remain virtual; `[package]` is forbidden"
                .to_owned(),
        );
    }
    let entries = match workspace_manifest_entries_from_str(contents) {
        Ok(entries) => entries,
        Err(error) => return vec![format!("Cargo.toml: {error}")],
    };
    let pair_present = entries
        .members
        .iter()
        .any(|entry| entry == DEPENDENCY_DECLARATIONS_MEMBER)
        || entries
            .exclude
            .iter()
            .any(|entry| entry == DEPENDENCY_DECLARATIONS_EXCLUDE);
    // Filter rather than append: a retiring entry that is still present must
    // remain in its canonical position.
    let mut expected_members: Vec<&str> = WORKSPACE_MEMBER_GLOBS
        .iter()
        .copied()
        .filter(|glob| {
            !RETIRING_MEMBER_GLOBS.contains(glob)
                || entries.members.iter().any(|entry| entry == glob)
        })
        .collect();
    let mut expected_excludes = WORKSPACE_EXCLUDES.to_vec();
    if pair_present {
        expected_members.push(DEPENDENCY_DECLARATIONS_MEMBER);
        expected_excludes.push(DEPENDENCY_DECLARATIONS_EXCLUDE);
    }
    compare_closed_entries(
        "member glob",
        &entries.members,
        &expected_members,
        &mut violations,
    );
    compare_closed_entries(
        "exclude",
        &entries.exclude,
        &expected_excludes,
        &mut violations,
    );
    violations
}

fn compare_closed_entries(
    surface: &str,
    actual: &[String],
    expected: &[&str],
    violations: &mut Vec<String>,
) {
    let actual_set: BTreeSet<&str> = actual.iter().map(String::as_str).collect();
    let expected_set: BTreeSet<&str> = expected.iter().copied().collect();
    for missing in expected_set.difference(&actual_set) {
        violations.push(format!(
            "Cargo.toml: required workspace {surface} `{missing}` is missing"
        ));
    }
    for unexpected in actual_set.difference(&expected_set) {
        violations.push(format!(
            "Cargo.toml: unexpected workspace {surface} `{unexpected}`"
        ));
    }
    if actual.len() != actual_set.len() {
        violations.push(format!(
            "Cargo.toml: duplicate workspace {surface} entries are forbidden"
        ));
    } else if actual_set == expected_set
        && !actual
            .iter()
            .map(String::as_str)
            .eq(expected.iter().copied())
    {
        violations.push(format!(
            "Cargo.toml: workspace {surface} entries must retain canonical order"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(members: &[&str], excludes: &[&str]) -> String {
        let members = members
            .iter()
            .map(|entry| format!("  {entry:?},\n"))
            .collect::<String>();
        let excludes = excludes
            .iter()
            .map(|entry| format!("  {entry:?},\n"))
            .collect::<String>();
        format!("[workspace]\nmembers = [\n{members}]\nexclude = [\n{excludes}]\nresolver = '2'\n")
    }

    #[test]
    fn exact_workspace_policy_admits() {
        assert!(
            workspace_membership_violations(&manifest(WORKSPACE_MEMBER_GLOBS, WORKSPACE_EXCLUDES))
                .is_empty()
        );
    }

    #[test]
    fn a_retiring_member_glob_is_admitted_present_or_absent() {
        assert!(
            workspace_membership_violations(&manifest(WORKSPACE_MEMBER_GLOBS, WORKSPACE_EXCLUDES))
                .is_empty()
        );

        let retired: Vec<&str> = WORKSPACE_MEMBER_GLOBS
            .iter()
            .copied()
            .filter(|glob| !RETIRING_MEMBER_GLOBS.contains(glob))
            .collect();
        assert!(
            workspace_membership_violations(&manifest(&retired, WORKSPACE_EXCLUDES)).is_empty()
        );

        // The intermediate state: exactly one retiring glob absent must be
        // admitted too — retirement may land one entry at a time.
        let one_retired: Vec<&str> = WORKSPACE_MEMBER_GLOBS
            .iter()
            .copied()
            .filter(|glob| *glob != RETIRING_MEMBER_GLOBS[0])
            .collect();
        assert!(
            workspace_membership_violations(&manifest(&one_retired, WORKSPACE_EXCLUDES)).is_empty()
        );

        // A duplicated retiring entry is still a duplicate.
        let mut duplicated: Vec<&str> = WORKSPACE_MEMBER_GLOBS.to_vec();
        duplicated.push(RETIRING_MEMBER_GLOBS[0]);
        assert!(
            !workspace_membership_violations(&manifest(&duplicated, WORKSPACE_EXCLUDES)).is_empty()
        );

        // Tolerance is scoped: a required entry is still required.
        let required_dropped: Vec<&str> = WORKSPACE_MEMBER_GLOBS
            .iter()
            .copied()
            .filter(|glob| *glob != "app/*/core/*")
            .collect();
        assert!(
            workspace_membership_violations(&manifest(&required_dropped, WORKSPACE_EXCLUDES))
                .iter()
                .any(|item| item.contains("is missing"))
        );
    }

    #[test]
    fn workspace_policy_rejects_exclusions_and_member_drift() {
        let mut excludes = WORKSPACE_EXCLUDES.to_vec();
        excludes.push("network/core/route");
        let violations =
            workspace_membership_violations(&manifest(&WORKSPACE_MEMBER_GLOBS[1..], &excludes));
        assert!(violations.iter().any(|item| item.contains("is missing")));
        assert!(
            violations
                .iter()
                .any(|item| item.contains("network/core/route"))
        );

        let mut reordered = WORKSPACE_MEMBER_GLOBS.to_vec();
        reordered.swap(0, 1);
        assert!(
            workspace_membership_violations(&manifest(&reordered, WORKSPACE_EXCLUDES))
                .iter()
                .any(|item| item.contains("canonical order"))
        );
    }

    #[test]
    fn workspace_root_cannot_become_an_implicit_member() {
        let contents = format!(
            "{}\n[package]\nname = 'shadow-root'\nversion = '0.1.0'\nedition = '2024'\n[lib]\npath = 'README.md'\n",
            manifest(WORKSPACE_MEMBER_GLOBS, WORKSPACE_EXCLUDES)
        );
        assert!(
            workspace_membership_violations(&contents)
                .iter()
                .any(|violation| violation.contains("must remain virtual"))
        );
    }
}
